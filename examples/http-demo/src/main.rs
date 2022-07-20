use anyhow::Result;

use http_api::*;
wit_bindgen_rust::import!("../../wit/http-api.wit");
wit_error_rs::impl_error!(Error);

fn main() -> Result<()> {
    let router = Router::new().unwrap();
    let router_with_route = router.get("/hello", "handler").unwrap();
    println!("guest starting server");
    let _ = Server::serve("0.0.0.0:3000", &router_with_route).unwrap();
    // server.stop().unwrap();
    println!("guest moving on");

    Ok(())
}
