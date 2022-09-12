#![allow(clippy::upper_case_acronyms)]

use std::iter::zip;
use std::net::{SocketAddr, ToSocketAddrs};

use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use crossbeam_utils::thread;
use futures::executor::block_on;
pub use http::add_to_linker;
use http::*;
use hyper::{Body, Server};
use routerify::ext::RequestExt;
use routerify::{Router, RouterBuilder, RouterService};
use slight_common::{impl_resource, Buildable, Builder, Ctx, HostState};
use slight_events_api::ResourceMap;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::log;

use slight_http_api::{HttpBody, HttpHandler, HttpHeader, Method, Request};

wit_bindgen_wasmtime::export!("../../wit/http.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);

#[derive(Clone, Debug, Default)]
enum Methods {
    #[default]
    GET,
    PUT,
    POST,
    DELETE,
}

#[derive(Clone, Debug)]
struct Route {
    method: Methods,
    route: String,
    handler: String,
}

/// A Router implementation for the HTTP interface
#[derive(Default, Clone, Debug)]
pub struct RouterInner {
    /// The root directory of the filesystem
    _base_uri: String,
    routes: Vec<Route>,
}

impl RouterInner {
    fn new(uri: &str) -> Self {
        Self {
            _base_uri: uri.to_string(),
            ..Default::default()
        }
    }

    /// Adds a new route with `GET` method and the handler's name.
    fn get(&mut self, route: String, handler: String) -> Result<Self, Error> {
        self.add(route, handler, Methods::GET)
    }

    /// Adds a new route with `PUT` method and the handler's name.
    fn put(&mut self, route: String, handler: String) -> Result<Self, Error> {
        self.add(route, handler, Methods::PUT)
    }

    /// Adds a new route with `POST` method and the handler's name.
    fn post(&mut self, route: String, handler: String) -> Result<Self, Error> {
        self.add(route, handler, Methods::POST)
    }

    /// Adds a new route with `DELETE` method and the handler's name.
    fn delete(&mut self, route: String, handler: String) -> Result<Self, Error> {
        self.add(route, handler, Methods::DELETE)
    }

    /// Adds a new route with the given method and the handler's name.
    fn add(&mut self, route: String, handler: String, method: Methods) -> Result<Self, Error> {
        let route = Route {
            method,
            route,
            handler,
        };
        self.routes.push(route);
        Ok(self.clone())
    }
}

#[derive(Clone, Debug)]
pub struct ServerInner {
    closer: Arc<Mutex<UnboundedSender<()>>>,
}

impl ServerInner {
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
pub struct Http<T: Buildable> {
    host_state: HttpState<T>,
}

impl<T: Buildable> Http<T> {
    pub fn from_state(host_state: HttpState<T>) -> Self {
        Self { host_state }
    }
}

impl<T: Buildable + Send + Sync + 'static> Http<T> {
    pub fn update_state(&mut self, builder: Builder<T>) -> Result<()> {
        self.host_state.builder = Some(builder);
        Ok(())
    }

    pub fn close(&mut self) {
        if let Some(c) = self.host_state.closer.clone() {
            // server was started, so send the termination message
            let _ = c.lock().unwrap().send(());
        }
    }

    pub fn build_state(state: HttpState<T>) -> HostState {
        let http = Self { host_state: state };
        (
            Box::new(http),
            Some(Box::new(http::HttpTables::<Http<T>>::default())),
        )
    }
}

#[derive(Clone)]
pub struct HttpState<T: Buildable> {
    _resource_map: ResourceMap,
    builder: Option<Builder<T>>,
    closer: Option<Arc<Mutex<UnboundedSender<()>>>>,
}

impl<T: Buildable> HttpState<T> {
    pub fn new(_resource_map: ResourceMap) -> Self {
        Self {
            _resource_map,
            builder: None,
            closer: None,
        }
    }
}

impl<T: Buildable + Send + Sync + 'static> http::Http for Http<T> {
    type Router = RouterInner;
    type Server = ServerInner;

    fn router_new(&mut self) -> Result<Self::Router, Error> {
        Ok(RouterInner::default())
    }

    fn router_new_with_base(&mut self, base: &str) -> Result<Self::Router, Error> {
        Ok(RouterInner::new(base))
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

    fn router_put(
        &mut self,
        router: &Self::Router,
        route: &str,
        handler: &str,
    ) -> Result<Self::Router, Error> {
        // Router is a reference to the router proxy, so we need to clone it to get a
        // mutable reference to the router.
        let mut rclone = router.clone();
        rclone.put(route.to_string(), handler.to_string())
    }

    fn router_post(
        &mut self,
        router: &Self::Router,
        route: &str,
        handler: &str,
    ) -> Result<Self::Router, Error> {
        // Router is a reference to the router proxy, so we need to clone it to get a
        // mutable reference to the router.
        let mut rclone = router.clone();
        rclone.post(route.to_string(), handler.to_string())
    }

    fn router_delete(
        &mut self,
        router: &Self::Router,
        route: &str,
        handler: &str,
    ) -> Result<Self::Router, Error> {
        // Router is a reference to the router proxy, so we need to clone it to get a
        // mutable reference to the router.
        let mut rclone = router.clone();
        rclone.delete(route.to_string(), handler.to_string())
    }

    fn server_serve(
        &mut self,
        address: &str,
        router: &Self::Router,
    ) -> Result<Self::Server, Error> {
        // Shared states for all routes
        let instance_builder = self.host_state.builder.as_ref().unwrap().clone();

        // The outer builder is used to define the route paths, while creating a scope
        // for the inner builder which passes states to the route handler.
        let mut outer_builder: RouterBuilder<Body, anyhow::Error> =
            Router::builder().data(instance_builder);

        // There is a one-to-one mapping between the outer router's scope and inner router builder.
        let mut inner_routes = vec![];
        for route in router.routes.iter() {
            // per route state
            let mut inner_builder: RouterBuilder<Body, anyhow::Error> = Router::builder();
            inner_builder = inner_builder.data(route.clone());
            match route.method {
                Methods::GET => {
                    inner_builder = inner_builder.get("/", handler::<T>);
                }
                Methods::PUT => {
                    inner_builder = inner_builder.put("/", handler::<T>);
                }
                Methods::POST => {
                    inner_builder = inner_builder.post("/", handler::<T>);
                }
                Methods::DELETE => {
                    inner_builder = inner_builder.delete("/", handler::<T>);
                }
            }
            inner_routes.push(inner_builder.build().unwrap());
        }

        // Create a scope for each inner route.
        for (route, built) in zip(router.routes.clone(), inner_routes) {
            outer_builder = outer_builder.scope(&route.route, built);
        }
        let built = outer_builder.build().unwrap();

        // Log the routes for debugging purposes.
        log::debug!("{:#?}", built);

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
        Ok(ServerInner { closer: arc_tx })
    }

    fn server_stop(&mut self, server: &Self::Server) -> Result<(), Error> {
        // clone is needed here because we have a reference to `ServerInner`,
        // but we need ownership of `ServerInner` to stop it.
        let clone = server.clone();
        clone.close()
    }
}

async fn handler<T: Buildable + Send + Sync + 'static>(
    request: hyper::Request<Body>,
) -> Result<hyper::Response<Body>> {
    log::debug!("received request: {:?}", &request);
    let (parts, body) = request.into_parts();

    // Fetch states from the request, including the route name and builder.
    let route = parts.data::<Route>().unwrap();

    let instance_builder = parts.data::<Builder<T>>().unwrap();
    let (mut store, instance) = instance_builder.inner().build();

    // Perform conversion from the `hyper::Request` to `handle_http::Request`.
    let params = parts.params();
    let params: Vec<(&str, &str)> = params
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let method: Method = (&parts.method).into();
    let headers: HttpHeader = (&parts.headers).into();

    // FIXME: `HttpBody::from_body` returns a future here. The reason that `block_on` is used is
    // because the `store` and `instance` are holding a mutex, which means that the async runtime
    // cannot switch to another thread.
    let bytes = block_on(HttpBody::from_body(body))?.inner();
    let uri = &(&parts.uri).to_string();
    let req = Request {
        method,
        uri,
        headers: &headers.inner(),
        body: Some(&bytes),
        params: &params,
    };

    // Construct http handler
    let handler = HttpHandler::new(&mut store, &instance, &route.handler, |ctx| {
        ctx.get_http_state_mut()
    })?;

    // Invoke the handler with http request
    let res = handler.handle_http(&mut store, req)??;

    // Perform the conversion from `handle_http::Response` to `hyper::Response`.
    log::debug!("response: {:?}", res);
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

impl_resource!(Http<T>, HttpTables<Http<T>>, HttpState<T>, T);

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

        let address = str_to_socket_address("localhost:8080")?;
        if address.is_ipv4() {
            assert_eq!(address.ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        } else {
            assert_eq!(
                address.ip(),
                IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
            );
        }

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
