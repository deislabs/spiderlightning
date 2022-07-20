use anyhow::Result;
use crossbeam_utils::thread;
use futures::executor::block_on;
use hyper::{Body, Request, Response, Server};
use std::{convert::Infallible, net::SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel, UnboundedReceiver};
use routerify::{Router, RouterService};
use runtime::{
    impl_resource,
    resource::{
        get_table, Ctx, DataT, Linker, Resource, ResourceMap, ResourceTables, RuntimeResource,
    },
};
use wasmtime::Store;

use http_api::*;

wit_bindgen_wasmtime::export!("../../wit/http-api.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);

const SCHEME_NAME: &str = "http-api";

#[derive(Clone, Debug, Default)]
enum Methods {
    #[default]
    GET,
}

#[derive(Clone, Debug)]
struct Route {
    method: Methods,
    route: String,
    // handler: String,
}

/// A Router implementation for the HTTP interface
#[derive(Default, Clone, Debug)]
pub struct RouterProxy {
    /// The root directory of the filesystem
    base_uri: Option<String>,

    routes: Vec<Route>,
}

impl RouterProxy {
    fn new() -> Self {
        Self::new_with_base(None)
    }

    fn new_with_base(uri: Option<String>) -> Self {
        Self {
            base_uri: uri,
            routes: Vec::new(),
        }
    }

    fn get(&mut self, route: String, _handler: String) -> Result<Self, Error> {
        let route = Route {
            method: Methods::GET,
            route,
            // handler,
        };
        self.routes.push(route);
        Ok(self.clone())
    }
}

#[derive(Clone, Debug)]
pub struct ServerProxy {
    closer: Arc<Mutex<UnboundedSender<()>>>
}

impl ServerProxy {
    fn close(self) -> Result<(), Error> {
        let clone = self.closer.clone();
        let closer = clone.lock().unwrap();
        thread::scope(|s| {
            s.spawn(|_| {
                block_on(async {
                    println!("shutting down the http server");
                    closer.send(())
                })
            });
        }).unwrap();
        Ok(())
    }
}

/// HttpApi capability
#[derive(Default)]
pub struct HttpApi {
    host_state: Option<ResourceMap>,
    closer: Option<Arc<Mutex<UnboundedSender<()>>>>
}

impl_resource!(
    HttpApi,
    http_api::HttpApiTables<HttpApi>,
    ResourceMap,
    SCHEME_NAME.to_string()
);

impl HttpApi {
    pub fn close(&mut self) {
        match self.closer.clone() {
            Some(c) => {
                // server was started, so send the termination message
                let _ = c.lock().unwrap().send(());
            },
            // nothing to do b/c server wasn't started
            _ => (),
        }
    }
}

impl Resource for HttpApi {
    fn get_inner(&self) -> &dyn std::any::Any {
        unimplemented!("events will not be dynamically dispatched to a specific resource")
    }

    fn watch(
        &mut self,
        _data: &str,
        _rd: &str,
        _key: &str,
        _sender: Arc<Mutex<crossbeam_channel::Sender<events_api::Event>>>,
    ) -> Result<()> {
        unimplemented!("events will not be listened to")
    }
}

impl http_api::HttpApi for HttpApi {
    type Router = RouterProxy;
    type Server = ServerProxy;

    fn router_new(&mut self) -> Result<Self::Router, Error> {
        Ok(RouterProxy::default())
    }

    fn router_new_with_base(&mut self, base: &str) -> Result<Self::Router, Error> {
        Ok(RouterProxy::new_with_base(Some(base.to_string())))
    }

    fn router_get(
        &mut self,
        router: &Self::Router,
        route: &str,
        handler: &str,
    ) -> Result<Self::Router, Error> {
        let mut rclone = router.clone();
        rclone.get(route.to_string(), handler.to_string())
    }

    fn server_serve(
        &mut self,
        _address: &str,
        router: &Self::Router,
    ) -> Result<Self::Server, Error> {
        let mut builder = Router::builder();
        for route in router.routes.iter() {
            match route.method {
                Methods::GET => {
                    builder = builder.get(&route.route, handler)
                },
            }
        }

        let built = builder.build().unwrap();
        let service = RouterService::new(built).unwrap();
        let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
        let server = Server::bind(&addr).serve(service);
        let (tx, rx) = unbounded_channel();
        let graceful = server.with_graceful_shutdown(shutdown_signal(rx));
        tokio::task::spawn(graceful);

        let arc_tx = Arc::new(Mutex::new(tx));
        self.closer = Some(arc_tx.clone());

        Ok(ServerProxy{
            closer: arc_tx,
        })
    }

    fn server_stop(&mut self, server: &Self::Server) -> Result<(), Error> {
        let clone = server.clone();
        clone.close()
    }
}

async fn shutdown_signal(mut rx: UnboundedReceiver<()>) {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = rx.recv() => {},
    }
    println!("shutting down the server")
}

async fn handler(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from(format!(
        "Hello method: {:?}, uri: {:?}",
        request.method(),
        request.uri(),
    ))))
}

// routes.get("/foo/:id", "getFoo");
// fn getFoo(req) -> res {...}

// routes.put("/bar/:id", "putBar");
// fn putBar(req) -> res {...}
