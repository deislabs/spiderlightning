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

pub fn spawn(executable: &str, args: Vec<&str>) -> anyhow::Result<Box<dyn FnOnce()>> {
    let mut cmd = Command::new(executable);
    for arg in args {
        cmd.arg(arg);
    }
    let mut child = cmd.spawn().expect("Error spawning the process");

    let f = move || {
        child.kill().expect("command wasn't running");
        child.wait().ok();
    };
    Ok(Box::new(f))
}

mod integration_tests {
    #[cfg(test)]
    mod configs_tests {
        use crate::{run, SLIGHT};
        use anyhow::Result;

        const CONFIGS_TEST_MODULE: &str =
            "./tests/configs-test/target/wasm32-wasi/debug/configs-test.wasm";

        #[test]
        fn envvars_test() -> Result<()> {
            let file_config = "./tests/configs-test/azapp_slightfile.toml";
            run(
                SLIGHT,
                vec!["-c", file_config, "run", "-m", CONFIGS_TEST_MODULE],
            );
            Ok(())
        }

        #[test]
        fn usersecrets_test() -> Result<()> {
            let file_config = "./tests/configs-test/us_slightfile.toml";
            run(
                SLIGHT,
                vec!["-c", file_config, "run", "-m", CONFIGS_TEST_MODULE],
            );
            Ok(())
        }

        #[test]
        fn azapp_test() -> Result<()> {
            let file_config = "./tests/configs-test/azapp_slightfile.toml";
            run(
                SLIGHT,
                vec!["-c", file_config, "run", "-m", CONFIGS_TEST_MODULE],
            );
            Ok(())
        }
    }

    #[cfg(test)]
    mod keyvalue_tests {
        #[cfg(unix)]
        use std::{
            env,
            net::{Ipv4Addr, SocketAddrV4, TcpListener},
            process::Command,
        };

        use crate::{run, SLIGHT};
        use anyhow::Result;

        const KEYVALUE_TEST_MODULE: &str =
            "./tests/keyvalue-test/target/wasm32-wasi/debug/keyvalue-test.wasm";

        #[test]
        fn filesystem_test() -> Result<()> {
            let file_config = "./tests/keyvalue-test/keyvalue_filesystem_slightfile.toml";
            run(
                SLIGHT,
                vec!["-c", file_config, "run", "-m", KEYVALUE_TEST_MODULE],
            );
            Ok(())
        }

        #[test]
        fn azblob_test() -> Result<()> {
            let file_config = "./tests/keyvalue-test/keyvalue_azblob_slightfile.toml";
            run(
                SLIGHT,
                vec!["-c", file_config, "run", "-m", KEYVALUE_TEST_MODULE],
            );
            Ok(())
        }

        #[test]
        fn aws_dynamodb_test() -> Result<()> {
            let file_config = "./tests/keyvalue-test/keyvalue_awsdynamodb_slightfile.toml";
            run(
                SLIGHT,
                vec!["-c", file_config, "run", "-m", KEYVALUE_TEST_MODULE],
            );
            Ok(())
        }

        #[test]
        #[cfg(unix)] // TODO: Add Windows support
        fn redis_test() -> Result<()> {
            // make sure redis server is running
            let port = get_random_port();

            // make sure redis-server is running
            let mut binary_path = "redis-server";
            let output = Command::new("which")
                .arg(binary_path)
                .output()
                .expect("failed to execute process");

            if !output.status.success() {
                binary_path = "/home/linuxbrew/.linuxbrew/opt/redis/bin/redis-server";
                let output = Command::new("which")
                    .arg(binary_path)
                    .output()
                    .expect("failed to execute process");
                if !output.status.success() {
                    panic!("redis-server not found");
                }
            }

            let mut cmd = Command::new(binary_path)
                .args(["--port", port.to_string().as_str()])
                .spawn()?;

            // sleep 5 seconds waiting for redis server to start
            std::thread::sleep(std::time::Duration::from_secs(5));

            let file_config = "./tests/keyvalue-test/keyvalue_redis_slightfile.toml";
            env::set_var("REDIS_ADDRESS", format!("redis://127.0.0.1:{}", port));
            run(
                SLIGHT,
                vec!["-c", file_config, "run", "-m", KEYVALUE_TEST_MODULE],
            );

            // kill the server
            cmd.kill()?;
            Ok(())
        }

        #[cfg(unix)]
        fn get_random_port() -> u16 {
            TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
                .expect("Unable to bind to check for port")
                .local_addr()
                .unwrap()
                .port()
        }
    }

    #[cfg(unix)]
    mod http_tests_unix {
        use crate::{spawn, SLIGHT};
        use anyhow::Result;
        use hyper::{body, client::HttpConnector, Body, Client, Method, Request, StatusCode};

        use tokio::{
            join,
            time::{sleep, Duration},
        };
        // use futures::future::{FutureExt};

        const HTTP_TEST_MODULE: &str = "./tests/http-test/target/wasm32-wasi/debug/http-test.wasm";

        #[tokio::test]
        async fn http_test() -> Result<()> {
            let config = "./tests/http-test/slightfile.toml";
            let child = spawn(SLIGHT, vec!["-c", config, "run", "-m", HTTP_TEST_MODULE])?;
            sleep(Duration::from_secs(2)).await;

            let client = hyper::Client::new();

            let (res1, res2, res3, res4, res5) = join!(
                handle_get_request(&client),
                handle_get_params(&client),
                handle_put_request(&client),
                handle_post_request(&client),
                handle_delete_request(&client),
            );

            child();

            assert!(res1.is_ok());
            assert!(res2.is_ok());
            assert!(res3.is_ok());
            assert!(res4.is_ok());
            assert!(res5.is_ok());

            Ok(())
        }

        async fn handle_get_request(client: &Client<HttpConnector>) -> Result<()> {
            let res = client.get("http://0.0.0.0:3000/hello".parse()?).await?;
            assert!(res.status().is_success());

            // curl -X GET http://0.0.0.0:3000/foo
            let res = client.get("http://0.0.0.0:3000/foo".parse()?).await?;
            assert!(!res.status().is_success());
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

    #[cfg(test)]
    #[cfg(unix)]
    mod messaging_tests {
        use std::time::Duration;

        use crate::{spawn, SLIGHT};
        use anyhow::Result;
        use hyper::{Body, Method, Request};
        use mosquitto_rs::{Client, QoS};
        use tokio::{process::Command, time::sleep};

        const MESSAGING_HTTP_SERVICE_MODULE: &str =
            "./tests/messaging-test/target/wasm32-wasi/debug/messaging-test.wasm";
        const MESSAGING_SERVICE_A_MODULE: &str =
            "./tests/messaging-test/target/wasm32-wasi/debug/consumer_a.wasm";
        const MESSAGING_SERVICE_B_MODULE: &str =
            "./tests/messaging-test/target/wasm32-wasi/debug/consumer_b.wasm";

        #[tokio::test]
        async fn messaging_test() -> Result<()> {
            let mut mosquitto_binary_path = "mosquitto";
            let output = Command::new("which")
                .arg(mosquitto_binary_path)
                .output()
                .await
                .expect("failed to execute process");

            if !output.status.success() {
                mosquitto_binary_path = "/usr/local/sbin/mosquitto";
            }
            let mosquitto_child = spawn(mosquitto_binary_path, vec![])?;

            let file_config = "./tests/messaging-test/messaging.slightfile.toml";
            let http_child = spawn(
                SLIGHT,
                vec![
                    "-c",
                    file_config,
                    "run",
                    "-m",
                    MESSAGING_HTTP_SERVICE_MODULE,
                ],
            )?;
            let service_a_child = spawn(
                SLIGHT,
                vec!["-c", file_config, "run", "-m", MESSAGING_SERVICE_A_MODULE],
            )?;
            let service_b_child = spawn(
                SLIGHT,
                vec!["-c", file_config, "run", "-m", MESSAGING_SERVICE_B_MODULE],
            )?;

            sleep(Duration::from_secs(2)).await;
            // read streams "service-a-channel-out" and "service-b-channel-out"
            let mut client = Client::with_auto_id().unwrap();

            client
                .connect("localhost", 1883, std::time::Duration::from_secs(5), None)
                .await
                .unwrap();

            client
                .subscribe("service-a-channel-out", QoS::AtLeastOnce)
                .await
                .unwrap();
            client
                .subscribe("service-b-channel-out", QoS::AtLeastOnce)
                .await
                .unwrap();

            let http_client = hyper::Client::new();
            let req = Request::builder()
                .method(Method::PUT)
                .uri("http://0.0.0.0:3001/send")
                .body(Body::from("a message!"))
                .expect("request builder");

            // curl -X PUT http://0.0.0.0:3001/send -d "a message!"
            let res = http_client.request(req).await?;
            assert!(res.status().is_success());

            sleep(Duration::from_secs(2)).await;
            let subscriber = client.subscriber().unwrap();
            let msg1 = subscriber.try_recv()?;
            let msg2 = subscriber.try_recv()?;

            http_child();
            service_a_child();
            service_b_child();
            mosquitto_child();

            assert!(msg1.payload == "a message!".as_bytes());
            assert!(msg2.payload == "a message!".as_bytes());
            Ok(())
        }
    }
    // TODO: We need to add distributed_locking modules
}
