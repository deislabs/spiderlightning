#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
pub use http_handler::{Error, HttpHandler, HttpHandlerData, Method, Request, Response};
use hyper::{
    header::{HeaderName, HeaderValue},
    Body, HeaderMap, StatusCode,
};
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
impl From<&hyper::Method> for Method {
    fn from(method: &hyper::Method) -> Self {
        match method {
            &hyper::Method::GET => Method::Get,
            &hyper::Method::POST => Method::Post,
            &hyper::Method::PUT => Method::Put,
            &hyper::Method::DELETE => Method::Delete,
            &hyper::Method::PATCH => Method::Patch,
            &hyper::Method::HEAD => Method::Head,
            &hyper::Method::OPTIONS => Method::Options,
            _ => ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                &["unsupported method"],
                &[],
            )),
        }
    }
}
impl From<Response> for hyper::Response<Body> {
    fn from(res: Response) -> Self {
        let mut response = if let Some(body) = res.body {
            hyper::Response::new(Body::from(body))
        } else {
            hyper::Response::new(Body::empty())
        };
        *response.status_mut() = StatusCode::from_u16(res.status).unwrap();
        if let Some(headers) = res.headers {
            let headers = HeaderMap::from_iter(headers.iter().map(|(key, value)| {
                (
                    HeaderName::from_bytes(key.as_bytes()).unwrap(),
                    HeaderValue::from_str(value).unwrap(),
                )
            }));
            *response.headers_mut() = headers;
        }
        response
    }
}
pub mod http_handler {
    #[allow(unused_imports)]
    use wit_bindgen_wasmtime::{wasmtime, anyhow};
    #[repr(u8)]
    pub enum Method {
        Get,
        Post,
        Put,
        Delete,
        Patch,
        Head,
        Options,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Method {
        #[inline]
        fn clone(&self) -> Method {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Method {}
    impl ::core::marker::StructuralPartialEq for Method {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Method {
        #[inline]
        fn eq(&self, other: &Method) -> bool {
            {
                let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                if __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for Method {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Method {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    impl std::fmt::Debug for Method {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Method::Get => f.debug_tuple("Method::Get").finish(),
                Method::Post => f.debug_tuple("Method::Post").finish(),
                Method::Put => f.debug_tuple("Method::Put").finish(),
                Method::Delete => f.debug_tuple("Method::Delete").finish(),
                Method::Patch => f.debug_tuple("Method::Patch").finish(),
                Method::Head => f.debug_tuple("Method::Head").finish(),
                Method::Options => f.debug_tuple("Method::Options").finish(),
            }
        }
    }
    pub type Uri<'a> = &'a str;
    pub type HeadersParam<'a> = &'a [(&'a str, &'a str)];
    pub type HeadersResult = Vec<(String, String)>;
    pub type Params<'a> = &'a [(&'a str, &'a str)];
    pub type BodyParam<'a> = &'a [u8];
    pub type BodyResult = Vec<u8>;
    pub struct Request<'a> {
        pub method: Method,
        pub uri: Uri<'a>,
        pub headers: HeadersParam<'a>,
        pub params: Params<'a>,
        pub body: Option<BodyParam<'a>>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<'a> ::core::clone::Clone for Request<'a> {
        #[inline]
        fn clone(&self) -> Request<'a> {
            match *self {
                Self {
                    method: ref __self_0_0,
                    uri: ref __self_0_1,
                    headers: ref __self_0_2,
                    params: ref __self_0_3,
                    body: ref __self_0_4,
                } => Request {
                    method: ::core::clone::Clone::clone(&(*__self_0_0)),
                    uri: ::core::clone::Clone::clone(&(*__self_0_1)),
                    headers: ::core::clone::Clone::clone(&(*__self_0_2)),
                    params: ::core::clone::Clone::clone(&(*__self_0_3)),
                    body: ::core::clone::Clone::clone(&(*__self_0_4)),
                },
            }
        }
    }
    impl<'a> std::fmt::Debug for Request<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Request")
                .field("method", &self.method)
                .field("uri", &self.uri)
                .field("headers", &self.headers)
                .field("params", &self.params)
                .field("body", &self.body)
                .finish()
        }
    }
    pub type HttpStatus = u16;
    pub struct Response {
        pub status: HttpStatus,
        pub headers: Option<HeadersResult>,
        pub body: Option<BodyResult>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Response {
        #[inline]
        fn clone(&self) -> Response {
            match *self {
                Self {
                    status: ref __self_0_0,
                    headers: ref __self_0_1,
                    body: ref __self_0_2,
                } => Response {
                    status: ::core::clone::Clone::clone(&(*__self_0_0)),
                    headers: ::core::clone::Clone::clone(&(*__self_0_1)),
                    body: ::core::clone::Clone::clone(&(*__self_0_2)),
                },
            }
        }
    }
    impl std::fmt::Debug for Response {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Response")
                .field("status", &self.status)
                .field("headers", &self.headers)
                .field("body", &self.body)
                .finish()
        }
    }
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
    /// Auxiliary data associated with the wasm exports.
    ///
    /// This is required to be stored within the data of a
    /// `Store<T>` itself so lifting/lowering state can be managed
    /// when translating between the host and wasm.
    pub struct HttpHandlerData {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::default::Default for HttpHandlerData {
        #[inline]
        fn default() -> HttpHandlerData {
            HttpHandlerData {}
        }
    }
    pub struct HttpHandler<T> {
        get_state: Box<dyn Fn(&mut T) -> &mut HttpHandlerData + Send + Sync>,
        canonical_abi_free: wasmtime::TypedFunc<(i32, i32, i32), ()>,
        canonical_abi_realloc: wasmtime::TypedFunc<(i32, i32, i32, i32), i32>,
        handle_http:
            wasmtime::TypedFunc<(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32), (i32,)>,
        memory: wasmtime::Memory,
    }
    impl<T> HttpHandler<T> {
        #[allow(unused_variables)]
        /// Adds any intrinsics, if necessary for this exported wasm
        /// functionality to the `linker` provided.
        ///
        /// The `get_state` closure is required to access the
        /// auxiliary data necessary for these wasm exports from
        /// the general store's state.
        pub fn add_to_linker(
            linker: &mut wasmtime::Linker<T>,
            get_state: impl Fn(&mut T) -> &mut HttpHandlerData + Send + Sync + Copy + 'static,
        ) -> anyhow::Result<()> {
            Ok(())
        }
        /// Instantiates the provided `module` using the specified
        /// parameters, wrapping up the result in a structure that
        /// translates between wasm and the host.
        ///
        /// The `linker` provided will have intrinsics added to it
        /// automatically, so it's not necessary to call
        /// `add_to_linker` beforehand. This function will
        /// instantiate the `module` otherwise using `linker`, and
        /// both an instance of this structure and the underlying
        /// `wasmtime::Instance` will be returned.
        ///
        /// The `get_state` parameter is used to access the
        /// auxiliary state necessary for these wasm exports from
        /// the general store state `T`.
        pub fn instantiate(
            mut store: impl wasmtime::AsContextMut<Data = T>,
            module: &wasmtime::Module,
            linker: &mut wasmtime::Linker<T>,
            get_state: impl Fn(&mut T) -> &mut HttpHandlerData + Send + Sync + Copy + 'static,
        ) -> anyhow::Result<(Self, wasmtime::Instance)> {
            Self::add_to_linker(linker, get_state)?;
            let instance = linker.instantiate(&mut store, module)?;
            Ok((Self::new(store, &instance, get_state)?, instance))
        }
        /// Low-level creation wrapper for wrapping up the exports
        /// of the `instance` provided in this structure of wasm
        /// exports.
        ///
        /// This function will extract exports from the `instance`
        /// defined within `store` and wrap them all up in the
        /// returned structure which can be used to interact with
        /// the wasm module.
        pub fn new(
            mut store: impl wasmtime::AsContextMut<Data = T>,
            instance: &wasmtime::Instance,
            get_state: impl Fn(&mut T) -> &mut HttpHandlerData + Send + Sync + Copy + 'static,
        ) -> anyhow::Result<Self> {
            let mut store = store.as_context_mut();
            let canonical_abi_free = instance
                .get_typed_func::<(i32, i32, i32), (), _>(&mut store, "canonical_abi_free")?;
            let canonical_abi_realloc = instance.get_typed_func::<(i32, i32, i32, i32), i32, _>(
                &mut store,
                "canonical_abi_realloc",
            )?;
            let handle_http = instance
                .get_typed_func::<(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32), (i32,), _>(
                    &mut store,
                    "handle-http",
                )?;
            let memory = instance.get_memory(&mut store, "memory").ok_or_else(|| {
                ::anyhow::private::must_use({
                    let error = ::anyhow::private::format_err(::core::fmt::Arguments::new_v1(
                        &["`memory` export not a memory"],
                        &[],
                    ));
                    error
                })
            })?;
            Ok(HttpHandler {
                canonical_abi_free,
                canonical_abi_realloc,
                handle_http,
                memory,
                get_state: Box::new(get_state),
            })
        }
        pub fn handle_http(
            &self,
            mut caller: impl wasmtime::AsContextMut<Data = T>,
            req: Request<'_>,
        ) -> Result<Result<Response, Error>, wasmtime::Trap> {
            let func_canonical_abi_realloc = &self.canonical_abi_realloc;
            let func_canonical_abi_free = &self.canonical_abi_free;
            let memory = &self.memory;
            let Request {
                method: method0,
                uri: uri0,
                headers: headers0,
                params: params0,
                body: body0,
            } = req;
            let vec1 = uri0;
            let ptr1 =
                func_canonical_abi_realloc.call(&mut caller, (0, 0, 1, vec1.len() as i32))?;
            memory
                .data_mut(&mut caller)
                .store_many(ptr1, vec1.as_bytes())?;
            let vec5 = headers0;
            let len5 = vec5.len() as i32;
            let result5 = func_canonical_abi_realloc.call(&mut caller, (0, 0, 4, len5 * 16))?;
            for (i, e) in vec5.into_iter().enumerate() {
                let base = result5 + (i as i32) * 16;
                {
                    let (t2_0, t2_1) = e;
                    let vec3 = t2_0;
                    let ptr3 = func_canonical_abi_realloc
                        .call(&mut caller, (0, 0, 1, vec3.len() as i32))?;
                    memory
                        .data_mut(&mut caller)
                        .store_many(ptr3, vec3.as_bytes())?;
                    memory.data_mut(&mut caller).store(
                        base + 4,
                        wit_bindgen_wasmtime::rt::as_i32(vec3.len() as i32),
                    )?;
                    memory
                        .data_mut(&mut caller)
                        .store(base + 0, wit_bindgen_wasmtime::rt::as_i32(ptr3))?;
                    let vec4 = t2_1;
                    let ptr4 = func_canonical_abi_realloc
                        .call(&mut caller, (0, 0, 1, vec4.len() as i32))?;
                    memory
                        .data_mut(&mut caller)
                        .store_many(ptr4, vec4.as_bytes())?;
                    memory.data_mut(&mut caller).store(
                        base + 12,
                        wit_bindgen_wasmtime::rt::as_i32(vec4.len() as i32),
                    )?;
                    memory
                        .data_mut(&mut caller)
                        .store(base + 8, wit_bindgen_wasmtime::rt::as_i32(ptr4))?;
                }
            }
            let vec9 = params0;
            let len9 = vec9.len() as i32;
            let result9 = func_canonical_abi_realloc.call(&mut caller, (0, 0, 4, len9 * 16))?;
            for (i, e) in vec9.into_iter().enumerate() {
                let base = result9 + (i as i32) * 16;
                {
                    let (t6_0, t6_1) = e;
                    let vec7 = t6_0;
                    let ptr7 = func_canonical_abi_realloc
                        .call(&mut caller, (0, 0, 1, vec7.len() as i32))?;
                    memory
                        .data_mut(&mut caller)
                        .store_many(ptr7, vec7.as_bytes())?;
                    memory.data_mut(&mut caller).store(
                        base + 4,
                        wit_bindgen_wasmtime::rt::as_i32(vec7.len() as i32),
                    )?;
                    memory
                        .data_mut(&mut caller)
                        .store(base + 0, wit_bindgen_wasmtime::rt::as_i32(ptr7))?;
                    let vec8 = t6_1;
                    let ptr8 = func_canonical_abi_realloc
                        .call(&mut caller, (0, 0, 1, vec8.len() as i32))?;
                    memory
                        .data_mut(&mut caller)
                        .store_many(ptr8, vec8.as_bytes())?;
                    memory.data_mut(&mut caller).store(
                        base + 12,
                        wit_bindgen_wasmtime::rt::as_i32(vec8.len() as i32),
                    )?;
                    memory
                        .data_mut(&mut caller)
                        .store(base + 8, wit_bindgen_wasmtime::rt::as_i32(ptr8))?;
                }
            }
            let (result11_0, result11_1, result11_2) = match body0 {
                Some(e) => {
                    let vec10 = e;
                    let ptr10 = func_canonical_abi_realloc
                        .call(&mut caller, (0, 0, 1, (vec10.len() as i32) * 1))?;
                    memory.data_mut(&mut caller).store_many(ptr10, &vec10)?;
                    (1i32, ptr10, vec10.len() as i32)
                }
                None => {
                    let e = ();
                    {
                        let () = e;
                        (0i32, 0i32, 0i32)
                    }
                }
            };
            let (result12_0,) = self.handle_http.call(
                &mut caller,
                (
                    method0 as i32,
                    ptr1,
                    vec1.len() as i32,
                    result5,
                    len5,
                    result9,
                    len9,
                    result11_0,
                    result11_1,
                    result11_2,
                ),
            )?;
            let load13 = memory.data_mut(&mut caller).load::<u8>(result12_0 + 0)?;
            Ok(match i32::from(load13) {
                0 => Ok({
                    let load14 = memory.data_mut(&mut caller).load::<u16>(result12_0 + 4)?;
                    let load15 = memory.data_mut(&mut caller).load::<u8>(result12_0 + 8)?;
                    let load25 = memory.data_mut(&mut caller).load::<u8>(result12_0 + 20)?;
                    Response {
                        status: u16::try_from(i32::from(load14)).map_err(bad_int)?,
                        headers: match i32::from(load15) {
                            0 => None,
                            1 => Some({
                                let load16 =
                                    memory.data_mut(&mut caller).load::<i32>(result12_0 + 12)?;
                                let load17 =
                                    memory.data_mut(&mut caller).load::<i32>(result12_0 + 16)?;
                                let len24 = load17;
                                let base24 = load16;
                                let mut result24 = Vec::with_capacity(len24 as usize);
                                for i in 0..len24 {
                                    let base = base24 + i * 16;
                                    result24.push({
                                        let load18 =
                                            memory.data_mut(&mut caller).load::<i32>(base + 0)?;
                                        let load19 =
                                            memory.data_mut(&mut caller).load::<i32>(base + 4)?;
                                        let ptr20 = load18;
                                        let len20 = load19;
                                        let data20 =
                                            copy_slice(&mut caller, memory, ptr20, len20, 1)?;
                                        func_canonical_abi_free
                                            .call(&mut caller, (ptr20, len20, 1))?;
                                        let load21 =
                                            memory.data_mut(&mut caller).load::<i32>(base + 8)?;
                                        let load22 =
                                            memory.data_mut(&mut caller).load::<i32>(base + 12)?;
                                        let ptr23 = load21;
                                        let len23 = load22;
                                        let data23 =
                                            copy_slice(&mut caller, memory, ptr23, len23, 1)?;
                                        func_canonical_abi_free
                                            .call(&mut caller, (ptr23, len23, 1))?;
                                        (
                                            String::from_utf8(data20).map_err(|_| {
                                                wasmtime::Trap::new("invalid utf-8")
                                            })?,
                                            String::from_utf8(data23).map_err(|_| {
                                                wasmtime::Trap::new("invalid utf-8")
                                            })?,
                                        )
                                    });
                                }
                                func_canonical_abi_free
                                    .call(&mut caller, (base24, len24 * 16, 4))?;
                                result24
                            }),
                            _ => return Err(invalid_variant("option")),
                        },
                        body: match i32::from(load25) {
                            0 => None,
                            1 => Some({
                                let load26 =
                                    memory.data_mut(&mut caller).load::<i32>(result12_0 + 24)?;
                                let load27 =
                                    memory.data_mut(&mut caller).load::<i32>(result12_0 + 28)?;
                                let ptr28 = load26;
                                let len28 = load27;
                                let data28 = copy_slice(&mut caller, memory, ptr28, len28, 1)?;
                                func_canonical_abi_free.call(&mut caller, (ptr28, len28 * 1, 1))?;
                                data28
                            }),
                            _ => return Err(invalid_variant("option")),
                        },
                    }
                }),
                1 => Err({
                    let load29 = memory.data_mut(&mut caller).load::<u8>(result12_0 + 4)?;
                    match i32::from(load29) {
                        0 => Error::ErrorWithDescription({
                            let load30 =
                                memory.data_mut(&mut caller).load::<i32>(result12_0 + 8)?;
                            let load31 =
                                memory.data_mut(&mut caller).load::<i32>(result12_0 + 12)?;
                            let ptr32 = load30;
                            let len32 = load31;
                            let data32 = copy_slice(&mut caller, memory, ptr32, len32, 1)?;
                            func_canonical_abi_free.call(&mut caller, (ptr32, len32, 1))?;
                            String::from_utf8(data32)
                                .map_err(|_| wasmtime::Trap::new("invalid utf-8"))?
                        }),
                        _ => return Err(invalid_variant("Error")),
                    }
                }),
                _ => return Err(invalid_variant("expected")),
            })
        }
    }
    use wit_bindgen_wasmtime::rt::RawMem;
    use wit_bindgen_wasmtime::rt::invalid_variant;
    use core::convert::TryFrom;
    use wit_bindgen_wasmtime::rt::bad_int;
    use wit_bindgen_wasmtime::rt::copy_slice;
}
const _ : & str = "use { request, response } from http-types\nuse { error } from types\n\nhandle-http: function(req: request) -> expected<response, error>\n" ;
