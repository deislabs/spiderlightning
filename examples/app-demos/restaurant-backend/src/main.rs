use anyhow::Result;

use slight_http_handler_macro::register_handler;

wit_bindgen_rust::import!("../../../wit/http.wit");
wit_error_rs::impl_error!(http::Error);

wit_bindgen_rust::import!("../../../wit/mq.wit");
wit_error_rs::impl_error!(mq::Error);

fn main() -> Result<()> {
    let router = http::Router::new()?;
    let router_with_route = router
        .post("/orders/", "handle_make_order")?
        .get("/orders/next/", "handle_get_next_order")?;
    let _ = http::Server::serve("0.0.0.0:3000", &router_with_route)?;
    Ok(())
}

#[register_handler]
fn handle_make_order(req: Request) -> Result<Response, Error> {
    let mq = crate::mq::Mq::open("slight-restaurant").expect("failed to open message queue");
    mq.send(&req.body.as_ref().unwrap())
        .expect("failed to make order");
    Ok(Response {
        headers: Some(req.headers),
        body: None,
        status: 200,
    })
}

#[register_handler]
fn handle_get_next_order(req: Request) -> Result<Response, Error> {
    let mq = crate::mq::Mq::open("slight-restaurant").expect("failed to open message queue");
    Ok(Response {
        headers: Some(req.headers),
        body: Some(mq.receive().expect("failed to get next order")),
        status: 200,
    })
}

#[register_handler]
fn handle_http(_req: Request) -> Result<Response, Error> {
    Err(Error::ErrorWithDescription(
        "this is a dummy handler".to_string(),
    ))
}
