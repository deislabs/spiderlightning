use anyhow::Result;

use http_api::*;
use http_handler_macro::register_handler;

wit_bindgen_rust::import!("../../wit/http-api.wit");

fn main() -> Result<()> {
    let router = Router::new().unwrap();
    let router_with_route = router.get("/hello", "handle_hello").unwrap();
    let router_with_route = router_with_route.get("/foo", "handle_foo").unwrap();
    println!("guest starting server");
    let _ = Server::serve("0.0.0.0:3000", &router_with_route).unwrap();
    // server.stop().unwrap();
    println!("guest moving on");

    Ok(())
}

#[register_handler]
fn handle_hello(req: Request) -> Result<Response, Error> {
    Ok(Response {
        headers: Some(req.headers),
        body: req.body,
        status: 200,
    })
}

#[register_handler]
fn handle_foo(request: Request) -> Result<Response, Error> {
    Ok(Response {
        headers: Some(request.headers),
        body: request.body,
        status: 500,
    })
}
