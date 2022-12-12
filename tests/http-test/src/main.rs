use anyhow::Result;

use http::*;
use slight_http_handler_macro::register_handler;

wit_bindgen_rust::import!("../../wit/http.wit");
wit_error_rs::impl_error!(http::HttpError);

fn main() -> Result<()> {
    let router = Router::new()?;
    let router_with_route = router
        .get("/hello", "handle_hello")?
        .get("/person/:name", "handle_person_with_name")?
        .get("/foo", "handle_foo")?
        .put("/bar", "handle_bar")?
        .post("/upload", "upload")?
        .delete("/delete-file", "delete_file_handler")?;

    println!("guest starting server");
    let _ = Server::serve("0.0.0.0:3000", &router_with_route)?;
    // server.stop().unwrap();
    println!("guest moving on");

    Ok(())
}

#[register_handler]
fn handle_hello(req: Request) -> Result<Response, HttpError> {
    Ok(Response {
        headers: Some(req.headers),
        body: Some("hello".as_bytes().to_vec()),
        status: 200,
    })
}

#[register_handler]
fn handle_person_with_name(req: Request) -> Result<Response, HttpError> {
    let id = req.params.into_iter().find(|x| x.0 == "name").unwrap_or(("".into(), "".into()));
    Ok(Response {
        headers: Some(req.headers),
        body: Some(format!("hello: {}", id.1).as_bytes().to_vec()),
        status: 200,
    })
}

#[register_handler]
fn handle_foo(request: Request) -> Result<Response, HttpError> {
    Ok(Response {
        headers: Some(request.headers),
        body: request.body,
        status: 500,
    })
}

#[register_handler]
fn handle_bar(request: Request) -> Result<Response, HttpError> {
    assert_eq!(request.method, Method::Put);
    Ok(Response {
        headers: Some(request.headers),
        body: request.body,
        status: 200,
    })
}

#[register_handler]
fn delete_file_handler(request: Request) -> Result<Response, HttpError> {
    assert_eq!(request.method, Method::Delete);
    Ok(Response {
        headers: Some(request.headers),
        body: request.body,
        status: 200,
    })
}


#[register_handler]
fn upload(request: Request) -> Result<Response, HttpError> {
    assert_eq!(request.method, Method::Post);
    Ok(Response {
        headers: Some(request.headers),
        body: request.body,
        status: 200,
    })
}

/// FIXME: We should come up with a solution that removes the need to register a 
/// handle_http function. The reason this is needed because the host is 
/// parsing the wit htto-handler file which has the handle_http function.        
#[register_handler]
fn handle_http(_req: Request) -> Result<Response, HttpError> {
    Err(HttpError::UnexpectedError(
        "this is a dummy handler".to_string(),
    ))
}