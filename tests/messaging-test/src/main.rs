use anyhow::Result;

use http::*;
use slight_http_handler_macro::register_handler;

wit_bindgen_rust::import!("../../wit/http.wit");
wit_bindgen_rust::import!("../../wit/messaging.wit");
wit_error_rs::impl_error!(http::HttpRouterError);
wit_error_rs::impl_error!(messaging::MessagingError);

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
