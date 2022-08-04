["../../wit/http.wit"]
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
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
pub mod http {
    #[allow(unused_imports)]
    use wit_bindgen_wasmtime::{wasmtime, anyhow};
    pub type Uri<'a> = &'a str;
    pub enum Error {
        ErrorWithDescription(String),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Error {
        #[inline]
        fn clone(&self) -> Error {
            match (&*self,) {
                (&Error::ErrorWithDescription(ref __self_0),) => {
                    Error::ErrorWithDescription(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    impl std::fmt::Debug for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Error::ErrorWithDescription(e) => f
                    .debug_tuple("Error::ErrorWithDescription")
                    .field(e)
                    .finish(),
            }
        }
    }
    pub trait Http: Sized {
        type Router: std::fmt::Debug;
        type Server: std::fmt::Debug;
        fn router_new(&mut self) -> Result<Self::Router, Error>;
        fn router_new_with_base(&mut self, base: Uri<'_>) -> Result<Self::Router, Error>;
        fn router_get(
            &mut self,
            self_: &Self::Router,
            route: &str,
            handler: &str,
        ) -> Result<Self::Router, Error>;
        fn server_serve(
            &mut self,
            address: &str,
            router: &Self::Router,
        ) -> Result<Self::Server, Error>;
        fn server_stop(&mut self, self_: &Self::Server) -> Result<(), Error>;
        fn drop_router(&mut self, state: Self::Router) {
            drop(state);
        }
        fn drop_server(&mut self, state: Self::Server) {
            drop(state);
        }
    }
    pub struct HttpTables<T: Http> {
        pub(crate) router_table: wit_bindgen_wasmtime::Table<T::Router>,
        pub(crate) server_table: wit_bindgen_wasmtime::Table<T::Server>,
    }
    impl<T: Http> Default for HttpTables<T> {
        fn default() -> Self {
            Self {
                router_table: Default::default(),
                server_table: Default::default(),
            }
        }
    }
    pub fn add_to_linker<T, U>(
        linker: &mut wasmtime::Linker<T>,
        get: impl Fn(&mut T) -> (&mut U, &mut HttpTables<U>) + Send + Sync + Copy + 'static,
    ) -> anyhow::Result<()>
    where
        U: Http,
    {
        use wit_bindgen_wasmtime::rt::get_memory;
        use wit_bindgen_wasmtime::rt::get_func;
        linker.func_wrap(
            "http",
            "router::new",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let host = get(caller.data_mut());
                let (host, _tables) = host;
                let result = host.router_new();
                match result {
                    Ok(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                        caller_memory.store(
                            arg0 + 4,
                            wit_bindgen_wasmtime::rt::as_i32(_tables.router_table.insert(e) as i32),
                        )?;
                    }
                    Err(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                        match e {
                            Error::ErrorWithDescription(e) => {
                                caller_memory.store(
                                    arg0 + 4,
                                    wit_bindgen_wasmtime::rt::as_i32(0i32) as u8,
                                )?;
                                let vec0 = e;
                                let ptr0 = func_canonical_abi_realloc
                                    .call(&mut caller, (0, 0, 1, vec0.len() as i32))?;
                                let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                                let (_, _tables) = get(data);
                                caller_memory.store_many(ptr0, vec0.as_bytes())?;
                                caller_memory.store(
                                    arg0 + 12,
                                    wit_bindgen_wasmtime::rt::as_i32(vec0.len() as i32),
                                )?;
                                caller_memory
                                    .store(arg0 + 8, wit_bindgen_wasmtime::rt::as_i32(ptr0))?;
                            }
                        };
                    }
                };
                Ok(())
            },
        )?;
        linker.func_wrap(
            "http",
            "router::new-with-base",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let (mem, data) = memory.data_and_store_mut(&mut caller);
                let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                let host = get(data);
                let (host, _tables) = host;
                let ptr0 = arg0;
                let len0 = arg1;
                let param0 = _bc.slice_str(ptr0, len0)?;
                let result = host.router_new_with_base(param0);
                match result {
                    Ok(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                        caller_memory.store(
                            arg2 + 4,
                            wit_bindgen_wasmtime::rt::as_i32(_tables.router_table.insert(e) as i32),
                        )?;
                    }
                    Err(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                        match e {
                            Error::ErrorWithDescription(e) => {
                                caller_memory.store(
                                    arg2 + 4,
                                    wit_bindgen_wasmtime::rt::as_i32(0i32) as u8,
                                )?;
                                let vec1 = e;
                                let ptr1 = func_canonical_abi_realloc
                                    .call(&mut caller, (0, 0, 1, vec1.len() as i32))?;
                                let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                                let (_, _tables) = get(data);
                                caller_memory.store_many(ptr1, vec1.as_bytes())?;
                                caller_memory.store(
                                    arg2 + 12,
                                    wit_bindgen_wasmtime::rt::as_i32(vec1.len() as i32),
                                )?;
                                caller_memory
                                    .store(arg2 + 8, wit_bindgen_wasmtime::rt::as_i32(ptr1))?;
                            }
                        };
                    }
                };
                Ok(())
            },
        )?;
        linker.func_wrap(
            "http",
            "router::get",
            move |mut caller: wasmtime::Caller<'_, T>,
                  arg0: i32,
                  arg1: i32,
                  arg2: i32,
                  arg3: i32,
                  arg4: i32,
                  arg5: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let (mem, data) = memory.data_and_store_mut(&mut caller);
                let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                let host = get(data);
                let (host, _tables) = host;
                let ptr0 = arg1;
                let len0 = arg2;
                let ptr1 = arg3;
                let len1 = arg4;
                let param0 = _tables
                    .router_table
                    .get((arg0) as u32)
                    .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                let param1 = _bc.slice_str(ptr0, len0)?;
                let param2 = _bc.slice_str(ptr1, len1)?;
                let result = host.router_get(param0, param1, param2);
                match result {
                    Ok(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg5 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                        caller_memory.store(
                            arg5 + 4,
                            wit_bindgen_wasmtime::rt::as_i32(_tables.router_table.insert(e) as i32),
                        )?;
                    }
                    Err(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg5 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                        match e {
                            Error::ErrorWithDescription(e) => {
                                caller_memory.store(
                                    arg5 + 4,
                                    wit_bindgen_wasmtime::rt::as_i32(0i32) as u8,
                                )?;
                                let vec2 = e;
                                let ptr2 = func_canonical_abi_realloc
                                    .call(&mut caller, (0, 0, 1, vec2.len() as i32))?;
                                let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                                let (_, _tables) = get(data);
                                caller_memory.store_many(ptr2, vec2.as_bytes())?;
                                caller_memory.store(
                                    arg5 + 12,
                                    wit_bindgen_wasmtime::rt::as_i32(vec2.len() as i32),
                                )?;
                                caller_memory
                                    .store(arg5 + 8, wit_bindgen_wasmtime::rt::as_i32(ptr2))?;
                            }
                        };
                    }
                };
                Ok(())
            },
        )?;
        linker.func_wrap(
            "http",
            "server::serve",
            move |mut caller: wasmtime::Caller<'_, T>,
                  arg0: i32,
                  arg1: i32,
                  arg2: i32,
                  arg3: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let (mem, data) = memory.data_and_store_mut(&mut caller);
                let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                let host = get(data);
                let (host, _tables) = host;
                let ptr0 = arg0;
                let len0 = arg1;
                let param0 = _bc.slice_str(ptr0, len0)?;
                let param1 = _tables
                    .router_table
                    .get((arg2) as u32)
                    .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                let result = host.server_serve(param0, param1);
                match result {
                    Ok(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg3 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                        caller_memory.store(
                            arg3 + 4,
                            wit_bindgen_wasmtime::rt::as_i32(_tables.server_table.insert(e) as i32),
                        )?;
                    }
                    Err(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg3 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                        match e {
                            Error::ErrorWithDescription(e) => {
                                caller_memory.store(
                                    arg3 + 4,
                                    wit_bindgen_wasmtime::rt::as_i32(0i32) as u8,
                                )?;
                                let vec1 = e;
                                let ptr1 = func_canonical_abi_realloc
                                    .call(&mut caller, (0, 0, 1, vec1.len() as i32))?;
                                let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                                let (_, _tables) = get(data);
                                caller_memory.store_many(ptr1, vec1.as_bytes())?;
                                caller_memory.store(
                                    arg3 + 12,
                                    wit_bindgen_wasmtime::rt::as_i32(vec1.len() as i32),
                                )?;
                                caller_memory
                                    .store(arg3 + 8, wit_bindgen_wasmtime::rt::as_i32(ptr1))?;
                            }
                        };
                    }
                };
                Ok(())
            },
        )?;
        linker.func_wrap(
            "http",
            "server::stop",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let host = get(caller.data_mut());
                let (host, _tables) = host;
                let param0 = _tables
                    .server_table
                    .get((arg0) as u32)
                    .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                let result = host.server_stop(param0);
                match result {
                    Ok(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg1 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                        let () = e;
                    }
                    Err(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg1 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                        match e {
                            Error::ErrorWithDescription(e) => {
                                caller_memory.store(
                                    arg1 + 4,
                                    wit_bindgen_wasmtime::rt::as_i32(0i32) as u8,
                                )?;
                                let vec0 = e;
                                let ptr0 = func_canonical_abi_realloc
                                    .call(&mut caller, (0, 0, 1, vec0.len() as i32))?;
                                let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                                let (_, _tables) = get(data);
                                caller_memory.store_many(ptr0, vec0.as_bytes())?;
                                caller_memory.store(
                                    arg1 + 12,
                                    wit_bindgen_wasmtime::rt::as_i32(vec0.len() as i32),
                                )?;
                                caller_memory
                                    .store(arg1 + 8, wit_bindgen_wasmtime::rt::as_i32(ptr0))?;
                            }
                        };
                    }
                };
                Ok(())
            },
        )?;
        linker.func_wrap(
            "canonical_abi",
            "resource_drop_router",
            move |mut caller: wasmtime::Caller<'_, T>, handle: u32| {
                let (host, tables) = get(caller.data_mut());
                let handle = tables.router_table.remove(handle).map_err(|e| {
                    wasmtime::Trap::new({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["failed to remove handle: "],
                            &[::core::fmt::ArgumentV1::new_display(&e)],
                        ));
                        res
                    })
                })?;
                host.drop_router(handle);
                Ok(())
            },
        )?;
        linker.func_wrap(
            "canonical_abi",
            "resource_drop_server",
            move |mut caller: wasmtime::Caller<'_, T>, handle: u32| {
                let (host, tables) = get(caller.data_mut());
                let handle = tables.server_table.remove(handle).map_err(|e| {
                    wasmtime::Trap::new({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["failed to remove handle: "],
                            &[::core::fmt::ArgumentV1::new_display(&e)],
                        ));
                        res
                    })
                })?;
                host.drop_server(handle);
                Ok(())
            },
        )?;
        Ok(())
    }
    use wit_bindgen_wasmtime::rt::RawMem;
}
const _ : & str = "use { uri } from http-types\nuse { error } from types\n\nresource router {\n\t// create a new HTTP router\n\tstatic new: function() -> expected<router, error>\n\n    // create a new HTTP router\n\tstatic new-with-base: function(base: uri) -> expected<router, error>\n\n\t// register a HTTP GET route\n\tget: function(route: string, handler: string) -> expected<router, error>\n\n\t// put:\n\t// post:\n\t// delete:\n}\n\nresource server {\n    static serve: function(address: string, router: router) -> expected<server, error> // non-blocking\n    stop: function() -> expected<unit, error>\n}" ;
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        {
            let result = f.write_fmt(::core::fmt::Arguments::new_v1(
                &[""],
                &[::core::fmt::ArgumentV1::new_debug(&&self)],
            ));
            result
        }
    }
}
impl std::error::Error for Error {}
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::ErrorWithDescription(err.to_string())
    }
}
const SCHEME_NAME: &str = "http";
enum Methods {
    #[default]
    GET,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::clone::Clone for Methods {
    #[inline]
    fn clone(&self) -> Methods {
        match (&*self,) {
            (&Methods::GET,) => Methods::GET,
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for Methods {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match (&*self,) {
            (&Methods::GET,) => ::core::fmt::Formatter::write_str(f, "GET"),
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::default::Default for Methods {
    #[inline]
    fn default() -> Methods {
        Self::GET
    }
}
struct Route {
    method: Methods,
    route: String,
    handler: String,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::clone::Clone for Route {
    #[inline]
    fn clone(&self) -> Route {
        match *self {
            Self {
                method: ref __self_0_0,
                route: ref __self_0_1,
                handler: ref __self_0_2,
            } => Route {
                method: ::core::clone::Clone::clone(&(*__self_0_0)),
                route: ::core::clone::Clone::clone(&(*__self_0_1)),
                handler: ::core::clone::Clone::clone(&(*__self_0_2)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for Route {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            Self {
                method: ref __self_0_0,
                route: ref __self_0_1,
                handler: ref __self_0_2,
            } => ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "Route",
                "method",
                &&(*__self_0_0),
                "route",
                &&(*__self_0_1),
                "handler",
                &&(*__self_0_2),
            ),
        }
    }
}
/// A Router implementation for the HTTP interface
pub struct RouterProxy {
    /// The root directory of the filesystem
    base_uri: Option<String>,
    routes: Vec<Route>,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::default::Default for RouterProxy {
    #[inline]
    fn default() -> RouterProxy {
        RouterProxy {
            base_uri: ::core::default::Default::default(),
            routes: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::clone::Clone for RouterProxy {
    #[inline]
    fn clone(&self) -> RouterProxy {
        match *self {
            Self {
                base_uri: ref __self_0_0,
                routes: ref __self_0_1,
            } => RouterProxy {
                base_uri: ::core::clone::Clone::clone(&(*__self_0_0)),
                routes: ::core::clone::Clone::clone(&(*__self_0_1)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for RouterProxy {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            Self {
                base_uri: ref __self_0_0,
                routes: ref __self_0_1,
            } => ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "RouterProxy",
                "base_uri",
                &&(*__self_0_0),
                "routes",
                &&(*__self_0_1),
            ),
        }
    }
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
pub struct ServerProxy {
    closer: Arc<Mutex<UnboundedSender<()>>>,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::clone::Clone for ServerProxy {
    #[inline]
    fn clone(&self) -> ServerProxy {
        match *self {
            Self {
                closer: ref __self_0_0,
            } => ServerProxy {
                closer: ::core::clone::Clone::clone(&(*__self_0_0)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for ServerProxy {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            Self {
                closer: ref __self_0_0,
            } => ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "ServerProxy",
                "closer",
                &&(*__self_0_0),
            ),
        }
    }
}
impl ServerProxy {
    fn close(self) -> Result<(), Error> {
        let clone = self.closer.clone();
        let closer = clone.lock().unwrap();
        thread::scope(|s| {
            s.spawn(|_| {
                block_on(async {
                    {
                        ::std::io::_print(::core::fmt::Arguments::new_v1(
                            &["shutting down the http server\n"],
                            &[],
                        ));
                    };
                    closer.send(())
                })
            });
        })
        .unwrap();
        Ok(())
    }
}
/// Http capability
pub struct Http {
    host_state: HttpState,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::default::Default for Http {
    #[inline]
    fn default() -> Http {
        Http {
            host_state: ::core::default::Default::default(),
        }
    }
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
pub struct HttpState {
    resource_map: ResourceMap,
    store: Option<Arc<Mutex<Store<Ctx>>>>,
    instance: Option<Arc<Mutex<Instance>>>,
    closer: Option<Arc<Mutex<UnboundedSender<()>>>>,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::default::Default for HttpState {
    #[inline]
    fn default() -> HttpState {
        HttpState {
            resource_map: ::core::default::Default::default(),
            store: ::core::default::Default::default(),
            instance: ::core::default::Default::default(),
            closer: ::core::default::Default::default(),
        }
    }
}
impl HttpState {
    pub fn new(resource_map: ResourceMap) -> Self {
        Self {
            resource_map,
            ..Default::default()
        }
    }
}
impl runtime::resource::Resource for Http {}
impl runtime::resource::ResourceTables<dyn runtime::resource::Resource> for http::HttpTables<Http> {}
impl runtime::resource::ResourceBuilder for Http {
    type State = HttpState;
    fn add_to_linker(
        linker: &mut runtime::resource::Linker<runtime::resource::Ctx>,
    ) -> anyhow::Result<()> {
        crate::add_to_linker(linker, |cx| {
            runtime::resource::get_table::<Self, http::HttpTables<Http>>(
                cx,
                SCHEME_NAME.to_string(),
            )
        })
    }
    fn build_data(state: Self::State) -> anyhow::Result<runtime::resource::HostState> {
        let mut resource = Self { host_state: state };
        Ok((
            Box::new(resource),
            Some(Box::new(<http::HttpTables<Http>>::default())),
        ))
    }
}
impl Http {
    pub fn update_store(mut self, store: Arc<Mutex<Store<Ctx>>>) {
        self.host_state.store = Some(store);
    }
    pub fn close(&mut self) {
        if let Some(c) = self.host_state.closer.clone() {
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
        let mut builder: RouterBuilder<Body, anyhow::Error> =
            Router::builder().data(store).data(instance);
        for route in router.routes.iter() {
            let store = self.host_state.store.as_ref().unwrap().clone();
            match route.method {
                Methods::GET => {
                    builder = builder . get (& route . route , | request | async move { { let lvl = :: log :: Level :: Debug ; if lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () { :: log :: __private_api_log (:: core :: fmt :: Arguments :: new_v1 (& ["received request: "] , & [:: core :: fmt :: ArgumentV1 :: new_debug (& request)]) , lvl , & ("http" , "http" , "crates/http/src/lib.rs" , 189u32) , :: log :: __private_api :: Option :: None) ; } } ; let mut store = request . data :: < Arc < Mutex < Store < Ctx > > > > () . unwrap () . lock () . unwrap () ; let instance = request . data :: < Arc < Mutex < Instance > > > () . unwrap () . lock () . unwrap () ; let method = Method :: from (request . method ()) ; let url = & request . uri () . to_string () ; let headers = [("Content-Type" , "application/json")] ; let params = [("name" , "joe")] ; let body = None ; let req = HttpRequest { method : method , uri : url , headers : & headers , params : & params , body : body , } ; let mut handler = HttpHandler :: new (store . deref_mut () , instance . deref () , | ctx | { & mut ctx . http_state }) . unwrap () ; { let lvl = :: log :: Level :: Debug ; if lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () { :: log :: __private_api_log (:: core :: fmt :: Arguments :: new_v1 (& ["Invoking guest handler, hard coded as handle_hello for now"] , & []) , lvl , & ("http" , "http" , "crates/http/src/lib.rs" , 222u32) , :: log :: __private_api :: Option :: None) ; } } ; handler . handle_http = instance . get_typed_func :: < (i32 , i32 , i32 , i32 , i32 , i32 , i32 , i32 , i32 , i32) , (i32 ,) , _ > (store . deref_mut () , & func_name_to_abi_name (& route . handler)) . unwrap () ; match handler . handle_http (store . deref_mut () , req) { Ok (_) => { } Err (e) => { { :: std :: io :: _print (:: core :: fmt :: Arguments :: new_v1 (& ["" , "\n"] , & [:: core :: fmt :: ArgumentV1 :: new_display (& e)])) ; } ; :: core :: panicking :: panic_display (& e) ; } } let res = handler . handle_http (store . deref_mut () , req) ? ? ; { let lvl = :: log :: Level :: Debug ; if lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () { :: log :: __private_api_log (:: core :: fmt :: Arguments :: new_v1 (& ["response: "] , & [:: core :: fmt :: ArgumentV1 :: new_debug (& res)]) , lvl , & ("http" , "http" , "crates/http/src/lib.rs" , 238u32) , :: log :: __private_api :: Option :: None) ; } } ; Ok (res . into ()) }) ;
                }
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
        self.host_state.closer = Some(arc_tx.clone());
        Ok(ServerProxy { closer: arc_tx })
    }
    fn server_stop(&mut self, server: &Self::Server) -> Result<(), Error> {
        let clone = server.clone();
        clone.close()
    }
}
async fn shutdown_signal(mut rx: UnboundedReceiver<()>) {
    {
        #[doc(hidden)]
        mod __tokio_select_util {
            pub(super) enum Out<_0, _1> {
                _0(_0),
                _1(_1),
                Disabled,
            }
            pub(super) type Mask = u8;
        }
        use ::tokio::macros::support::Future;
        use ::tokio::macros::support::Pin;
        use ::tokio::macros::support::Poll::{Ready, Pending};
        const BRANCHES: u32 = 2;
        let mut disabled: __tokio_select_util::Mask = Default::default();
        if !true {
            let mask: __tokio_select_util::Mask = 1 << 0;
            disabled |= mask;
        }
        if !true {
            let mask: __tokio_select_util::Mask = 1 << 1;
            disabled |= mask;
        }
        let mut output = {
            let mut futures = (tokio::signal::ctrl_c(), rx.recv());
            ::tokio::macros::support::poll_fn(|cx| {
                let mut is_pending = false;
                let start = { ::tokio::macros::support::thread_rng_n(BRANCHES) };
                for i in 0..BRANCHES {
                    let branch;
                    #[allow(clippy::modulo_one)]
                    {
                        branch = (start + i) % BRANCHES;
                    }
                    match branch {
                        #[allow(unreachable_code)]
                        0 => {
                            let mask = 1 << branch;
                            if disabled & mask == mask {
                                continue;
                            }
                            let (fut, ..) = &mut futures;
                            let mut fut = unsafe { Pin::new_unchecked(fut) };
                            let out = match Future::poll(fut, cx) {
                                Ready(out) => out,
                                Pending => {
                                    is_pending = true;
                                    continue;
                                }
                            };
                            disabled |= mask;
                            #[allow(unused_variables)]
                            #[allow(unused_mut)]
                            match &out {
                                _ => {}
                                _ => continue,
                            }
                            return Ready(__tokio_select_util::Out::_0(out));
                        }
                        #[allow(unreachable_code)]
                        1 => {
                            let mask = 1 << branch;
                            if disabled & mask == mask {
                                continue;
                            }
                            let (_, fut, ..) = &mut futures;
                            let mut fut = unsafe { Pin::new_unchecked(fut) };
                            let out = match Future::poll(fut, cx) {
                                Ready(out) => out,
                                Pending => {
                                    is_pending = true;
                                    continue;
                                }
                            };
                            disabled |= mask;
                            #[allow(unused_variables)]
                            #[allow(unused_mut)]
                            match &out {
                                _ => {}
                                _ => continue,
                            }
                            return Ready(__tokio_select_util::Out::_1(out));
                        }
                        _ => ::core::panicking::unreachable_display(
                            &"reaching this means there probably is an off by one bug",
                        ),
                    }
                }
                if is_pending {
                    Pending
                } else {
                    Ready(__tokio_select_util::Out::Disabled)
                }
            })
            .await
        };
        match output {
            __tokio_select_util::Out::_0(_) => {}
            __tokio_select_util::Out::_1(_) => {}
            __tokio_select_util::Out::Disabled => {
                ::std::rt::begin_panic("all branches are disabled and there is no else branch")
            }
            _ => ::core::panicking::unreachable_display(&"failed to match bind"),
        }
    }
    {
        ::std::io::_print(::core::fmt::Arguments::new_v1(
            &["shutting down the server\n"],
            &[],
        ));
    }
}
fn func_name_to_abi_name(name: &str) -> String {
    name.replace('_', "-")
}
