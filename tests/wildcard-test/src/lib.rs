use anyhow::Result;
use rand::{distributions::Alphanumeric, prelude::*};

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
        .put("/register", "handle_register")?
        .put("/send/:id", "handle_send")?
        .get("/get/:id", "handle_get")?;

    println!("guest starting server");
    let _ = Server::serve("0.0.0.0:3002", &router_with_route)?;
    // server.stop().unwrap();
    println!("guest moving on");

    Ok(())
}

fn generate_random_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect::<String>()
}

#[register_handler]
fn handle_register(req: Request) -> Result<Response, HttpError> {
    let id = generate_random_id();
    let sub = messaging::Sub::open(&id).unwrap();
    let token = sub.subscribe("room").unwrap();

    Ok(Response {
        headers: Some(req.headers),
        body: Some(token.as_bytes().to_vec()),
        status: 200,
    })
}

#[register_handler]
fn handle_send(req: Request) -> Result<Response, HttpError> {
    let id = req
        .params
        .into_iter()
        .find(|x| x.0 == "id")
        .unwrap_or(("".into(), "".into()))
        .1;
    let publisher = messaging::Pub::open(&id).unwrap();
    publisher.publish(&req.body.unwrap(), "room").unwrap();
    Ok(Response {
        headers: Some(req.headers),
        body: None,
        status: 200,
    })
}

#[register_handler]
fn handle_get(request: Request) -> Result<Response, HttpError> {
    let id = request
        .params
        .into_iter()
        .find(|x| x.0 == "id")
        .unwrap_or(("".into(), "".into()))
        .1;
    let sub = messaging::Sub::open(&id).unwrap();
    let msg = sub.receive(&id).unwrap();
    Ok(Response {
        headers: Some(request.headers),
        body: Some(msg),
        status: 200,
    })
}
