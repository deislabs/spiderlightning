use anyhow::{bail, Result};
pub use http_handler::{HttpError, HttpHandlerData, Method, Request, Response};
pub use http_server_export::HttpServerExportData;
use hyper::{
    body::HttpBody as HyperHttpBody,
    header::{HeaderName, HeaderValue},
    Body, HeaderMap, StatusCode,
};

wit_bindgen_wasmtime::import!({paths: ["../../wit/http-handler.wit"], async: *});
wit_bindgen_wasmtime::import!({paths: ["../../wit/http-server-export.wit"], async: *});
wit_error_rs::impl_error!(http_handler::HttpError);

/// An exported HTTP server init function from the wasm module
///
/// This is a wrapper implementation of the WIT generated `HttpServerExport`
pub struct HttpServerInit<T> {
    inner: http_server_export::HttpServerExport<T>,
}

impl<T> AsRef<http_server_export::HttpServerExport<T>> for HttpServerInit<T> {
    fn as_ref(&self) -> &http_server_export::HttpServerExport<T> {
        &self.inner
    }
}

impl<T> AsMut<http_server_export::HttpServerExport<T>> for HttpServerInit<T> {
    fn as_mut(&mut self) -> &mut http_server_export::HttpServerExport<T> {
        &mut self.inner
    }
}

impl<T: Send> HttpServerInit<T> {
    pub fn new(
        store: impl wasmtime::AsContextMut<Data = T>,
        instance: &wasmtime::Instance,
        get_state: impl Fn(&mut T) -> &mut HttpServerExportData + Send + Sync + Copy + 'static,
    ) -> Result<Self> {
        http_server_export::HttpServerExport::new(store, instance, get_state)
            .map(|inner| Self { inner })
    }

    pub async fn on_server_init(
        &self,
        caller: impl wasmtime::AsContextMut<Data = T>,
    ) -> Result<Result<(), String>, anyhow::Error> {
        self.inner.on_server_init(caller).await
    }
}

/// A HTTP Handler that finds the handler function from the wasm module
/// and calls it with the HTTP request.
///
/// The purpose of this wrapper is to enable any http handler name being
/// registered in the host-side. The `new` constructor function will take
/// `handler_name` as a parameter, and then use it to find the handler function.
/// The handler_name may different from the function name defined in the wit
/// file.
///
/// For example, the handler_name may be "handle_hello" while the function
/// name in the wit file is "handle_http".
///
/// The handler_name must be defined in the wasm module and use the
/// `register_handler` macro to register the handler function.
pub struct HttpHandler<T> {
    inner: http_handler::HttpHandler<T>,
}

impl<T> AsRef<http_handler::HttpHandler<T>> for HttpHandler<T> {
    fn as_ref(&self) -> &http_handler::HttpHandler<T> {
        &self.inner
    }
}

impl<T> AsMut<http_handler::HttpHandler<T>> for HttpHandler<T> {
    fn as_mut(&mut self) -> &mut http_handler::HttpHandler<T> {
        &mut self.inner
    }
}

impl<T: Send> HttpHandler<T> {
    /// Create a new HTTP Handler.
    pub fn new(
        mut store: impl wasmtime::AsContextMut<Data = T>,
        instance: &wasmtime::Instance,
        handler_name: &str,
        get_state: impl Fn(&mut T) -> &mut HttpHandlerData + Send + Sync + Copy + 'static,
    ) -> anyhow::Result<Self> {
        let mut store = store.as_context_mut();
        let canonical_abi_free =
            instance.get_typed_func::<(i32, i32, i32), ()>(&mut store, "canonical_abi_free")?;
        let canonical_abi_realloc = instance
            .get_typed_func::<(i32, i32, i32, i32), i32>(&mut store, "canonical_abi_realloc")?;
        let handle_http = instance
            .get_typed_func::<(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32), (i32,)>(
                &mut store,
                handler_name,
            )?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("`memory` export not a memory"))?;
        let get_state = Box::new(get_state);
        Ok(Self {
            inner: http_handler::HttpHandler {
                get_state,
                canonical_abi_free,
                canonical_abi_realloc,
                handle_http,
                memory,
            },
        })
    }

    pub async fn handle_http(
        &self,
        caller: impl wasmtime::AsContextMut<Data = T>,
        req: Request<'_>,
    ) -> Result<Result<Response, http_handler::HttpError>, anyhow::Error> {
        self.inner.handle_http(caller, req).await
    }
}

/// An owned http_handler::HeadersParam
///
/// It can be directly transformed from `hyper::HeaderMap`
/// ```rust
/// let (parts, _) = request.into_parts();
/// let header_map: HttpHeader = (&parts.headers).into()
/// let req = http_handler::Request {
///     headers: &headers.inner(),
///     ...
/// }
/// ```
pub struct HttpHeader<'a>(Vec<(&'a str, &'a str)>);

impl<'a> HttpHeader<'a> {
    pub fn inner(self) -> Vec<(&'a str, &'a str)> {
        self.0
    }
}

impl<'a> From<&'a hyper::HeaderMap> for HttpHeader<'a> {
    fn from(headers: &'a hyper::HeaderMap) -> Self {
        Self(
            headers
                .iter()
                .map(|(name, value)| (name.as_str(), value.to_str().unwrap()))
                .collect(),
        )
    }
}

/// An owned http_handler::BodyParam
///
/// It can be directly transformed from `hyper::Body`
/// ```rust
/// let (parts, body) = request.into_parts();
/// let bytes = HttpBody::from_body(body).await?.inner();
/// let req = http_handler::Request {
///     body: Some(&bytes),
///     ...
/// }
/// ```
pub struct HttpBody(Vec<u8>);

impl HttpBody {
    pub async fn from_body(body: hyper::Body) -> Result<Self> {
        const MAX_ALLOWED_SIZE: u64 = u64::MAX;
        let content_length = match body.size_hint().upper() {
            Some(v) => v,
            None => bail!("failed to read HTTP body size"),
        };

        if content_length < MAX_ALLOWED_SIZE {
            let body_bytes = hyper::body::to_bytes(body).await?;
            let owned_body = Self(body_bytes.to_vec());
            return Ok(owned_body);
        }

        bail!(
            "failed due to HTTP body being too large (size: {}, allowed size: {})",
            content_length,
            MAX_ALLOWED_SIZE
        );
    }

    pub fn inner(self) -> Vec<u8> {
        self.0
    }
}

impl From<&hyper::Method> for Method {
    fn from(method: &hyper::Method) -> Self {
        match *method {
            hyper::Method::GET => Method::Get,
            hyper::Method::POST => Method::Post,
            hyper::Method::PUT => Method::Put,
            hyper::Method::DELETE => Method::Delete,
            hyper::Method::PATCH => Method::Patch,
            hyper::Method::HEAD => Method::Head,
            hyper::Method::OPTIONS => Method::Options,
            _ => panic!("failed due to unsupported method, currently supported methods are: GET, POST, PUT, DELETE, PATCH, HEAD, and OPTIONS"),
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

#[cfg(test)]
mod unittests {

    use crate::{HttpBody, HttpHeader};

    use super::{Body, HeaderValue, Method, Request, Response, StatusCode};
    use anyhow::Result;

    #[tokio::test]
    async fn test_request_conversion() -> Result<()> {
        let req = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri("http://localhost:8080/")
            .header("Content-Type", "application/json")
            .body(hyper::Body::from("{\"name\": \"John\"}"))?;

        let (parts, body) = req.into_parts();
        let headers: HttpHeader = (&parts.headers).into();
        let uri = parts.uri.to_string();
        let method: Method = (&parts.method).into();
        let bytes: HttpBody = HttpBody::from_body(body).await?;
        let params = [];
        let req = Request {
            method,
            uri: &uri,
            headers: &headers.0,
            params: &params,
            body: Some(&bytes.0),
        };

        assert_eq!(req.method, Method::Get);
        assert_eq!(req.headers[0].1, "application/json");
        assert_eq!(req.headers.len(), 1);
        assert_eq!(req.body, Some(b"{\"name\": \"John\"}".to_vec().as_ref()));
        assert_eq!(req.uri, "http://localhost:8080/");
        Ok(())
    }

    #[test]
    fn test_response_conversion() {
        let res = Response {
            status: 200,
            headers: Some(vec![("Content-Type".into(), "text/plain".into())]),
            body: Some("Hello World".into()),
        };
        let hyper_res: hyper::Response<Body> = res.into();
        assert_eq!(hyper_res.status(), StatusCode::OK);
        assert_eq!(
            hyper_res.headers().get("Content-Type"),
            Some(&HeaderValue::from_static("text/plain"))
        );
    }
}
