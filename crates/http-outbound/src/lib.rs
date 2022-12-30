use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;

use http_outbound::*;
wit_bindgen_wasmtime::export!({paths: ["../../wit/http-outbound.wit"], async: *});
wit_error_rs::impl_error!(http_outbound::HttpError);
wit_error_rs::impl_from!(anyhow::Error, http_outbound::HttpError::UnexpectedError);

#[derive(Clone, Default)]
pub struct HttpOutbound {
    client: Client,
}

impl HttpOutbound {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl http_outbound::HttpOutbound for HttpOutbound {
    async fn request(&mut self, req: Request<'_>) -> Result<Response, HttpError> {
        let mut builder = self.client.request(req.method.into(), req.uri);
        for header in req.headers {
            builder = builder.header(header.0, header.1);
        }
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

impl slight_common::Capability for HttpOutbound {}
impl slight_common::CapabilityBuilder for HttpOutbound {
    fn build(self) -> Result<slight_common::HostState> {
        Ok((Box::new(self), Some(Box::new(()))))
    }
}
impl slight_common::WasmtimeLinkable for HttpOutbound {
    fn add_to_linker<Ctx: slight_common::Ctx + Send + Sync + 'static>(
        linker: &mut slight_common::Linker<Ctx>,
    ) -> anyhow::Result<()> {
        http_outbound::add_to_linker(linker, |ctx| {
            let res = Ctx::get_host_state::<HttpOutbound, ()>(ctx, "http-outbound".to_string());
            res.0
        })
    }
}

impl From<http_outbound::Method> for reqwest::Method {
    fn from(method: http_outbound::Method) -> Self {
        match method {
            http_outbound::Method::Get => reqwest::Method::GET,
            http_outbound::Method::Post => reqwest::Method::POST,
            http_outbound::Method::Put => reqwest::Method::PUT,
            http_outbound::Method::Delete => reqwest::Method::DELETE,
            http_outbound::Method::Head => reqwest::Method::HEAD,
            http_outbound::Method::Options => reqwest::Method::OPTIONS,
            http_outbound::Method::Patch => reqwest::Method::PATCH,
        }
    }
}

impl From<reqwest::Error> for http_outbound::HttpError {
    fn from(e: reqwest::Error) -> Self {
        Self::UnexpectedError(e.to_string())
    }
}
