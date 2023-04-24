use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;

use http_client::*;
use slight_common::impl_resource;
wit_bindgen_wasmtime::export!({paths: ["../../wit/http-client.wit"], async: *});
wit_error_rs::impl_error!(http_client::HttpError);
wit_error_rs::impl_from!(anyhow::Error, http_client::HttpError::UnexpectedError);

#[derive(Clone, Default)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl http_client::HttpClient for HttpClient {
    async fn request(&mut self, req: Request<'_>) -> Result<Response, HttpError> {
        let mut builder = self.client.request(req.method.into(), req.uri);
        for header in req.headers {
            builder = builder.header(header.0, header.1);
        }
        builder = builder.body(if let Some(body) = req.body {
            String::from_utf8(body.to_vec()).unwrap()
        } else {
            "".to_string()
        });
        let res = builder.send().await?;

        let status = res.status().as_u16();
        let mut headers = vec![];
        for (name, value) in res.headers().iter() {
            headers.push((
                name.as_str().to_string(),
                value.to_str().unwrap().to_string(),
            ));
        }
        let body = Some(res.bytes().await?.to_vec());
        Ok(Response {
            status,
            headers: Some(headers),
            body,
        })
    }
}

impl_resource!(
    HttpClient,
    http_client::add_to_linker,
    "http-client".to_string()
);

impl From<http_client::Method> for reqwest::Method {
    fn from(method: http_client::Method) -> Self {
        match method {
            http_client::Method::Get => reqwest::Method::GET,
            http_client::Method::Post => reqwest::Method::POST,
            http_client::Method::Put => reqwest::Method::PUT,
            http_client::Method::Delete => reqwest::Method::DELETE,
            http_client::Method::Head => reqwest::Method::HEAD,
            http_client::Method::Options => reqwest::Method::OPTIONS,
            http_client::Method::Patch => reqwest::Method::PATCH,
        }
    }
}

impl From<reqwest::Error> for http_client::HttpError {
    fn from(e: reqwest::Error) -> Self {
        Self::UnexpectedError(e.to_string())
    }
}
