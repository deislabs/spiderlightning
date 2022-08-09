use anyhow::{bail, Result};
pub use http_handler::{Error, HttpHandler, HttpHandlerData, Method, Request, Response};
use hyper::{
    body::HttpBody as HyperHttpBody,
    header::{HeaderName, HeaderValue},
    Body, HeaderMap, StatusCode,
};

wit_bindgen_wasmtime::import!("../../wit/http-handler.wit");
wit_error_rs::impl_error!(Error);

/// A owned http_handler::HeadersParam
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

/// A owned http_handler::BodyParam
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
            None => bail!("HTTP Body too large"),
        };

        if content_length < MAX_ALLOWED_SIZE {
            let body_bytes = hyper::body::to_bytes(body).await?;
            let owned_body = Self(body_bytes.to_vec());
            return Ok(owned_body);
        }

        bail!("HTTP body too large")
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
            _ => panic!("unsupported method"),
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
        let uri = &(&parts.uri).to_string();
        let method: Method = (&parts.method).into();
        let bytes: HttpBody = HttpBody::from_body(body).await?;
        let params = [];
        let req = Request {
            method,
            uri,
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
