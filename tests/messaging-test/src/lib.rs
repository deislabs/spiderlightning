use anyhow::Result;

use http_server::*;
use slight_http_handler_macro::register_handler;
use slight_http_server_macro::on_server_init;

wit_bindgen_rust::import!("../../wit/http-server.wit");
wit_bindgen_rust::export!("../../wit/http-server-export.wit");
wit_bindgen_rust::import!("../../wit/messaging.wit");
wit_error_rs::impl_error!(http_server::HttpRouterError);
wit_error_rs::impl_error!(messaging::MessagingError);

#[on_server_init]
fn main() -> Result<()> {
    let router = Router::new()?;
    let router_with_route = router
        .put("/send", "handle_send")?;
    
    Server::serve("0.0.0.0:3001", &router_with_route)?;
    Ok(())
}

#[register_handler]
fn handle_send(req: Request) -> Result<Response, HttpError> {
    assert_eq!(req.method, Method::Put);
    let ps = crate::messaging::Pub::open("my-messaging").unwrap();
    let message = req.body.unwrap();
    ps.publish(&message, "room").unwrap();

    Ok(Response {
        headers: Some(req.headers),
        body: None,
        status: 200,
    })
}
