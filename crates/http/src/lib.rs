use std::iter::zip;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::{convert::Infallible, net::SocketAddr};

use anyhow::Result;
use crossbeam_utils::thread;
use futures::executor::block_on;
pub use http::add_to_linker;
use http::*;
use hyper::{Body, Request, Response, Server, StatusCode};
use routerify::ext::RequestExt;
use routerify::{Router, RouterBuilder, RouterService};
use runtime::{
    impl_resource,
    resource::{Ctx, ResourceMap},
};
use std::future::Future;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::log;
use url::Url;

use http_api::{
    Error as HttpError, HttpHandler, Method, Request as HttpRequest, Response as HttpResponse,
};
use wasmtime::{Instance, Store};

wit_bindgen_wasmtime::export!("../../wit/http.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);

const SCHEME_NAME: &str = "http";

#[derive(Clone, Debug, Default)]
enum Methods {
    #[default]
    GET,
}

#[derive(Clone, Debug)]
struct Route {
    method: Methods,
    route: String,
    handler: String,
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

    fn get(&mut self, route: String, handler: String) -> Result<Self, Error> {
        let route = Route {
            method: Methods::GET,
            route,
            handler,
        };
        self.routes.push(route);
        Ok(self.clone())
    }
}

#[derive(Clone, Debug)]
pub struct ServerProxy {
    closer: Arc<Mutex<UnboundedSender<()>>>,
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
        })
        .unwrap();
        Ok(())
    }
}

/// Http capability
#[derive(Default)]
pub struct Http {
    host_state: HttpState,
}
impl Http {
    pub fn update_state(
        &mut self,
        store: Arc<Mutex<Store<Ctx>>>,
        instance: Arc<Mutex<Instance>>,
    ) -> Result<()> {
        self.host_state.store = Some(store);
        self.host_state.instance = Some(instance);
        Ok(())
    }
}

#[derive(Default)]
pub struct HttpState {
    resource_map: ResourceMap,
    store: Option<Arc<Mutex<Store<Ctx>>>>,
    instance: Option<Arc<Mutex<Instance>>>,
    closer: Option<Arc<Mutex<UnboundedSender<()>>>>,
}

impl HttpState {
    pub fn new(resource_map: ResourceMap) -> Self {
        Self {
            resource_map,
            ..Default::default()
        }
    }
}

impl_resource!(
    Http,
    http::HttpTables<Http>,
    HttpState,
    SCHEME_NAME.to_string()
);

impl Http {
    pub fn update_store(mut self, store: Arc<Mutex<Store<Ctx>>>) {
        self.host_state.store = Some(store);
    }

    pub fn close(&mut self) {
        if let Some(c) = self.host_state.closer.clone() {
            // server was started, so send the termination message
            let _ = c.lock().unwrap().send(());
        }
    }
}

impl http::Http for Http {
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
        let store = self.host_state.store.as_mut().unwrap().clone();
        let instance = self.host_state.instance.as_mut().unwrap().clone();
        let mut outer_builder: RouterBuilder<Body, anyhow::Error> =
            Router::builder().data(store).data(instance);
        let mut built_routes = vec![];
        for route in router.routes.iter() {
            let mut builder: RouterBuilder<Body, anyhow::Error> = Router::builder();
            match route.method {
                Methods::GET => {
                    builder = builder.data(route.clone()); // hello, foo
                    builder = builder.get("/", |request| async move {
                        log::debug!("received request: {:?}", request);
                        let route = request.data::<Route>().unwrap();
                        let mut store = request
                            .data::<Arc<Mutex<Store<Ctx>>>>()
                            .unwrap()
                            .lock()
                            .unwrap();
                        let instance = request
                            .data::<Arc<Mutex<Instance>>>()
                            .unwrap()
                            .lock()
                            .unwrap();

                        let method = Method::from(request.method());
                        let url = &request.uri().to_string();
                        // FIXME: this is a hack to get the headers
                        let headers = [("Content-Type", "application/json")];
                        let params = [("name", "joe")];
                        let body = None;

                        let req = HttpRequest {
                            method: method,
                            uri: url,
                            headers: &headers,
                            params: &params,
                            body: body,
                        };

                        let mut handler =
                            HttpHandler::new(store.deref_mut(), instance.deref(), |ctx| {
                                &mut ctx.http_state
                            })
                            .unwrap();

                        log::debug!("Invoking guest handler {}", &route.handler);
                        handler.handle_http = instance
                            .get_typed_func::<(i32,i32,i32,i32,i32,i32,i32,i32,i32,i32,), (i32,), _>(
                                store.deref_mut(),
                                &func_name_to_abi_name(&route.handler),
                            )
                            .unwrap();
                        let res = handler.handle_http(store.deref_mut(), req)??;
                        log::debug!("response: {:?}", res);
                        Ok(res.into())
                    });
                }
            }
            built_routes.push(builder.build().unwrap());
        }

        for (route, built) in zip(router.routes.clone(), built_routes) {
            outer_builder = outer_builder.scope(&route.route, built);
        }
        let built = outer_builder.build().unwrap();
        let service = RouterService::new(built).unwrap();
        let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
        let server = Server::bind(&addr).serve(service);
        let (tx, rx) = unbounded_channel();
        let graceful = server.with_graceful_shutdown(shutdown_signal(rx));
        tokio::task::spawn(graceful);

        let arc_tx = Arc::new(Mutex::new(tx));
        self.host_state.closer = Some(arc_tx.clone());

        Ok(ServerProxy { closer: arc_tx })
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

// async fn handler(req,) -> res {
//  // get access to store?
// }
// routes.get("/foo/:id", "getFoo");
// fn getFoo(req) -> res {...}

// routes.put("/bar/:id", "putBar");
// fn putBar(req) -> res {...}
fn func_name_to_abi_name(name: &str) -> String {
    name.replace('_', "-")
}
