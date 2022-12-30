use anyhow::Result;

use http::*;
use slight_http_handler_macro::register_handler;

wit_bindgen_rust::import!("../../wit/http.wit");
wit_bindgen_rust::import!("../../wit/http-outbound.wit");
wit_error_rs::impl_error!(http::HttpError);
wit_error_rs::impl_error!(http_outbound::HttpError);

fn main() -> Result<()> {
    let router = Router::new()?;
    let router_with_route = router
        .get("/hello", "handle_hello")?;
    
    let _ = Server::serve("0.0.0.0:3000", &router_with_route)?;
    Ok(())
}

#[register_handler]
fn handle_hello(_req: Request) -> Result<Response, HttpError> {
    let req = crate::http_outbound::Request {
        method: crate::http_outbound::Method::Get,
        uri: "https://some-random-api.ml/facts/dog",
        headers: &[],
        body: None,
        params: &[],
    };
    let res = crate::http_outbound::request(req).unwrap();
    println!("{:?}", res);

    let res = Response {
        status: res.status,
        headers: res.headers,
        body: res.body,
    };
    Ok(res)
}