#![allow(clippy::upper_case_acronyms)]

use std::iter::zip;
use std::net::{SocketAddr, ToSocketAddrs};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use crossbeam_utils::thread;
use futures::executor::block_on;
pub use http::add_to_linker;
use http::*;
use hyper::{Body, Server};
use routerify::ext::RequestExt;
use routerify::{Router, RouterBuilder, RouterService};
use runtime::{
    impl_resource,
    resource::{Ctx, ResourceMap},
};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::log;

use http_api::{HttpBody, HttpHandler, HttpHeader, Method, Request};
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
    _base_uri: String,
    routes: Vec<Route>,
}

impl RouterProxy {
    fn new(uri: &str) -> Self {
        Self {
            _base_uri: uri.to_string(),
            ..Default::default()
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
        let closer = self.closer.lock().unwrap();
        thread::scope(|s| {
            s.spawn(|_| {
                block_on(async {
                    log::info!("shutting down the http server");
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
    _resource_map: ResourceMap,
    store: Option<Arc<Mutex<Store<Ctx>>>>,
    instance: Option<Arc<Mutex<Instance>>>,
    closer: Option<Arc<Mutex<UnboundedSender<()>>>>,
}

impl HttpState {
    pub fn new(_resource_map: ResourceMap) -> Self {
        Self {
            _resource_map,
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
        Ok(RouterProxy::new(base))
    }

    fn router_get(
        &mut self,
        router: &Self::Router,
        route: &str,
        handler: &str,
    ) -> Result<Self::Router, Error> {
        // Router is a reference to the router proxy, so we need to clone it to get a
        // mutable reference to the router.
        let mut rclone = router.clone();
        rclone.get(route.to_string(), handler.to_string())
    }

    fn server_serve(
        &mut self,
        address: &str,
        router: &Self::Router,
    ) -> Result<Self::Server, Error> {
        // Shared states for all routes
        let store = self.host_state.store.as_mut().unwrap().clone();
        let instance = self.host_state.instance.as_mut().unwrap().clone();

        // The outer builder is used to define the route paths, while creating a scope
        // for the inner builder which passes states to the route handler.
        let mut outer_builder: RouterBuilder<Body, anyhow::Error> =
            Router::builder().data(store).data(instance);

        // There is a one-to-one mapping between the outer router's scope and inner router builder.
        let mut inner_routes = vec![];
        for route in router.routes.iter() {
            let mut inner_builder: RouterBuilder<Body, anyhow::Error> = Router::builder();
            match route.method {
                Methods::GET => {
                    // per route state
                    inner_builder = inner_builder.data(route.clone());
                    inner_builder = inner_builder.get("/", handler);
                }
            }
            inner_routes.push(inner_builder.build().unwrap());
        }

        // Create a scope for each inner route.
        for (route, built) in zip(router.routes.clone(), inner_routes) {
            outer_builder = outer_builder.scope(&route.route, built);
        }
        let built = outer_builder.build().unwrap();

        // Defines the server
        let service = RouterService::new(built).unwrap();
        let addr = str_to_socket_address(address)?;
        let server = Server::bind(&addr).serve(service);
        // Create a channel to send the termination message
        let (tx, rx) = unbounded_channel();
        let graceful = server.with_graceful_shutdown(shutdown_signal(rx));
        // Start the server in a separate thread
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

async fn handler(request: hyper::Request<Body>) -> Result<hyper::Response<Body>> {
    log::debug!("received request: {:?}", &request);
    let (parts, body) = request.into_parts();

    // Fetch states from the request, including Store, Instance and the route name.
    let route = parts.data::<Route>().unwrap();
    let mut store = parts
        .data::<Arc<Mutex<Store<Ctx>>>>()
        .unwrap()
        .lock()
        .unwrap();
    let instance = parts
        .data::<Arc<Mutex<Instance>>>()
        .unwrap()
        .lock()
        .unwrap();

    // Perform conversion from the `hyper::Request` to `handle_http::Request`.
    let params = parts.params();
    let params: Vec<(&str, &str)> = params
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let methods: Method = (&parts.method).into();
    let headers: HttpHeader = (&parts.headers).into();

    // FIXME: `HttpBody::from_body` returns a future here. The reason that `block_on` is used is
    // because the `store` and `instance` are holding a mutex, which means that the async runtime
    // cannot switch to another thread.
    let bytes = block_on(HttpBody::from_body(body))?.inner();
    let uri = &(&parts.uri).to_string();
    let req = Request {
        method: methods,
        uri,
        headers: &headers.inner(),
        body: Some(&bytes),
        params: &params,
    };

    // Construct http handler
    let mut handler = HttpHandler::new(store.deref_mut(), instance.deref(), |ctx| {
        &mut ctx.http_state
    })
    .unwrap();

    // Perform the http request
    log::debug!("Invoking guest handler {}", &route.handler,);
    handler.handle_http = instance
        .get_typed_func::<(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32), (i32,), _>(
            store.deref_mut(),
            &route.handler.replace('_', "-"),
        )
        .unwrap();
    let res = handler.handle_http(store.deref_mut(), req)??;
    log::debug!("response: {:?}", res);

    // Perform the conversion from `handle_http::Response` to `hyper::Response`.
    Ok(res.into())
}

async fn shutdown_signal(mut rx: UnboundedReceiver<()>) {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = rx.recv() => {},
    }
    log::info!("shutting down the server")
}

fn str_to_socket_address(s: &str) -> Result<SocketAddr> {
    match s.to_socket_addrs().map(|mut iter| iter.next().unwrap()) {
        Ok(addr) => Ok(addr),
        Err(e) => bail!("could not parse address: {} due to {}", s, e),
    }
}

#[cfg(test)]
mod unittests {
    use super::str_to_socket_address;
    use anyhow::Result;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

    #[test]
    fn test_str_to_socket_address() -> Result<()> {
        assert_eq!(
            str_to_socket_address("0.0.0.0:3000")?,
            SocketAddr::new([0, 0, 0, 0].into(), 3000)
        );

        assert_eq!(
            str_to_socket_address("localhost:8080")?,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
        );

        assert_eq!(
            str_to_socket_address("[::1]:8080")?,
            SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 8080)
        );

        assert_eq!(
            str_to_socket_address("127.0.0.1:55555")?,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 55555)
        );
        Ok(())
    }
}
