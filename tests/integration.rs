use std::{
    io::{stderr, stdout, Write},
    process::Command,
};

pub const SLIGHT: &str = "./target/release/slight";

pub fn run(executable: &str, args: Vec<&str>) {
    let mut cmd = Command::new(executable);
    for arg in args {
        cmd.arg(arg);
    }
    let output = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("failed to execute process");

    let code = output.status.code().expect("should have status code");
    stdout().write_all(&output.stdout).unwrap();
    if code != 0 {
        stderr().write_all(&output.stderr).unwrap();
        panic!("failed to run spiderlightning");
    }
}

mod integration_tests {
    #[cfg(test)]
    mod kv_test {
        use crate::{run, SLIGHT};
        use anyhow::Result;

        const KV_TEST_MODULE: &str = "./tests/kv-test/target/wasm32-wasi/debug/kv-test.wasm";

        #[test]
        fn filekv_test() -> Result<()> {
            let file_config = "./tests/kv-test/kvfilesystem_slightfile.toml";
            run(SLIGHT, vec!["-c", file_config, "run", "-m", KV_TEST_MODULE]);
            Ok(())
        }

        #[test]
        fn azblobkv_test() -> Result<()> {
            let file_config = "./tests/kv-test/kvazblob_slightfile.toml";
            run(SLIGHT, vec!["-c", file_config, "run", "-m", KV_TEST_MODULE]);
            Ok(())
        }

        #[test]
        fn aws_dynamodb_test() -> Result<()> {
            let file_config = "./tests/kv-test/kvawsdynamodb_slightfile.toml";
            run(SLIGHT, vec!["-c", file_config, "run", "-m", KV_TEST_MODULE]);
            Ok(())
        }
    }

    #[cfg(unix)]
    mod http_tests_unix {

        use std::process::Command;

        use crate::SLIGHT;
        use anyhow::Result;
        use hyper::{body, client::HttpConnector, Body, Client, Method, Request, StatusCode};
        use signal_child::Signalable;

        use tokio::time::{sleep, Duration};
        // use futures::future::{FutureExt};

        const HTTP_TEST_MODULE: &str = "./tests/http-test/target/wasm32-wasi/debug/http-test.wasm";

        #[tokio::test]
        async fn http_test() -> Result<()> {
            let config = "./tests/http-test/slightfile.toml";
            let mut child = Command::new(SLIGHT)
                .args(&["-c", config, "run", "-m", HTTP_TEST_MODULE])
                .spawn()?;
            sleep(Duration::from_secs(2)).await;

            let client = hyper::Client::new();
            // can handle get requests
            handle_get_request(&client).await?;

            // can handle get params
            handle_get_params(&client).await?;

            // can handle put requests
            handle_put_request(&client).await?;

            // can handle post requests
            handle_post_request(&client).await?;

            // can handle delete requests
            handle_delete_request(&client).await?;

            child.interrupt().expect("Error interrupting child");
            child.wait().ok();

            Ok(())
        }

        async fn handle_get_request(client: &Client<HttpConnector>) -> Result<()> {
            let res = client.get("http://0.0.0.0:3000/hello".parse()?).await?;
            assert!(res.status().is_success());

            // curl -X GET http://0.0.0.0:3000/foo
            let res = client.get("http://0.0.0.0:3000/foo".parse()?).await?;
            assert_ne!(res.status().is_success(), true);
            assert!(res.status().is_server_error());

            // curl -X GET http://0.0.0.0:3000/should_return_404
            let res = client
                .get("http://0.0.0.0:3000/should_return_404".parse()?)
                .await?;
            assert_eq!(StatusCode::NOT_FOUND, res.status());
            Ok(())
        }

        async fn handle_get_params(client: &Client<HttpConnector>) -> Result<()> {
            // curl -X GET http://0.0.0.0:3000/hello/:name
            let res = client.get("http://0.0.0.0:3000/person/x".parse()?).await?;
            assert!(res.status().is_success());
            let body = res.into_body();
            let bytes = body::to_bytes(body).await?;
            assert_eq!(bytes, "hello: x".to_string());

            let res = client
                .get("http://0.0.0.0:3000/person/yager".parse()?)
                .await?;
            assert!(res.status().is_success());
            let body = res.into_body();
            let bytes = body::to_bytes(body).await?;
            assert_eq!(bytes, "hello: yager".to_string());

            // FIXME: there is a exiting issue in Routerify https://github.com/routerify/routerify/issues/118 that
            //       prevents the following test from working.

            // let mut res = client.get("http://0.0.0.0:3000/person/yager".parse()?).await?;
            // assert!(res.status().is_success());
            // let body = res.into_body();
            // let bytes = body::to_bytes(body).await?;
            // assert_eq!(bytes, "hello: yager".to_string());
            Ok(())
        }

        async fn handle_put_request(client: &Client<HttpConnector>) -> Result<()> {
            let req = Request::builder()
                .method(Method::PUT)
                .uri("http://0.0.0.0:3000/bar")
                .body(Body::from("Hallo!"))
                .expect("request builder");

            // curl -X PUT http://0.0.0.0:3000/bar
            let res = client.request(req).await?;
            assert!(res.status().is_success());
            Ok(())
        }

        async fn handle_post_request(client: &Client<HttpConnector>) -> Result<()> {
            let req = Request::builder()
                .method(Method::POST)
                .uri("http://0.0.0.0:3000/upload")
                .body(Body::from("Hallo!"))
                .expect("request builder");

            // curl -X POST http://0.0.0.0:3000/upload
            let res = client.request(req).await?;
            assert!(res.status().is_success());
            Ok(())
        }

        async fn handle_delete_request(client: &Client<HttpConnector>) -> Result<()> {
            let req = Request::builder()
                .method(Method::DELETE)
                .uri("http://0.0.0.0:3000/delete-file")
                .body(Body::from("Hallo!"))
                .expect("request builder");

            // curl -X DELETE http://0.0.0.0:3000/upload
            let res = client.request(req).await?;
            assert!(res.status().is_success());
            Ok(())
        }
    }
    // TODO: We need to mq_test, lockd_test, and pubsub_test modules
}
