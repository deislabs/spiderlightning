use anyhow::Result;

use http_server::*;
use slight_http_handler_macro::register_handler;

wit_bindgen_rust::import!("../../wit/http-server.wit");
wit_bindgen_rust::import!("../../wit/http-client.wit");
wit_error_rs::impl_error!(http_server::HttpError);
wit_error_rs::impl_error!(http_client::HttpError);

fn main() -> Result<()> {
    let router = Router::new()?;
    let router_with_route = router
        .get("/hello", "handle_hello")?;
    
    let _ = Server::serve("0.0.0.0:3000", &router_with_route)?;
    Ok(())
}

#[register_handler]
fn handle_hello(_req: Request) -> Result<Response, HttpError> {
    let req = crate::http_client::Request {
        method: crate::http_client::Method::Get,
        uri: "https://some-random-api.ml/facts/dog",
        headers: &[],
        body: None,
        params: &[],
    };
    let res = crate::http_client::request(req).unwrap();
    println!("{:?}", res);

    let res = Response {
        status: res.status,
        headers: res.headers,
        body: res.body,
    };
    Ok(res)
}