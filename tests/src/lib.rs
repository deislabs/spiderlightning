use std::{
    io::{stderr, stdout, Write},
    process::Command,
};

#[allow(dead_code)]
fn slight_path() -> String {
    format!("{}/../target/release/slight", env!("CARGO_MANIFEST_DIR"))
}

pub fn run(executable: &str, args: Vec<&str>, current_dir: Option<&str>) {
    println!("Running {executable} with args: {args:?}");
    let mut cmd = Command::new(executable);
    for arg in args {
        cmd.arg(arg);
    }
    if let Some(dir) = current_dir {
        cmd.current_dir(dir);
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
        use std::path::PathBuf;

        use crate::{run, slight_path};
        use anyhow::Result;

        #[test]
        fn filesystem_access_test() -> Result<()> {
            let out_dir = PathBuf::from(format!("{}/target/wasms", env!("CARGO_MANIFEST_DIR")));
            let out_dir = out_dir.join("wasm32-wasi/debug/filesystem-access-test.wasm");
            let file_config = &format!(
                "{}/filesystem-access-test/slightfile.toml",
                env!("CARGO_MANIFEST_DIR")
            );
            run(
                &slight_path(),
                vec!["-c", file_config, "run", out_dir.to_str().unwrap()],
                None,
            );
            Ok(())
        }

        #[test]
        fn envvars_test() -> Result<()> {
            let out_dir = PathBuf::from(format!("{}/target/wasms", env!("CARGO_MANIFEST_DIR")));
            let out_dir = out_dir.join("wasm32-wasi/debug/configs-test.wasm");
            let file_config = &format!(
                "{}/configs-test/azapp_slightfile.toml",
                env!("CARGO_MANIFEST_DIR")
            );
            run(
                &slight_path(),
                vec!["-c", file_config, "run", out_dir.to_str().unwrap()],
                None,
            );
            Ok(())
        }

        #[test]
        fn usersecrets_test() -> Result<()> {
            let out_dir = PathBuf::from(format!("{}/target/wasms", env!("CARGO_MANIFEST_DIR")));
            let out_dir = out_dir.join("wasm32-wasi/debug/configs-test.wasm");
            let file_config = &format!(
                "{}/configs-test/us_slightfile.toml",
                env!("CARGO_MANIFEST_DIR")
            );
            run(
                &slight_path(),
                vec!["-c", file_config, "run", out_dir.to_str().unwrap()],
                None,
            );
            Ok(())
        }

        #[test]
        fn azapp_test() -> Result<()> {
            let out_dir = PathBuf::from(format!("{}/target/wasms", env!("CARGO_MANIFEST_DIR")));
            let out_dir = out_dir.join("wasm32-wasi/debug/configs-test.wasm");
            let file_config = &format!(
                "{}/configs-test/azapp_slightfile.toml",
                env!("CARGO_MANIFEST_DIR")
            );
            run(
                &slight_path(),
                vec!["-c", file_config, "run", out_dir.to_str().unwrap()],
                None,
            );
            Ok(())
        }
    }

    #[cfg(test)]
    mod keyvalue_tests {
        use std::path::PathBuf;
        #[cfg(unix)]
        use std::{
            env,
            net::{Ipv4Addr, SocketAddrV4, TcpListener},
            process::Command,
        };

        use crate::{run, slight_path};
        use anyhow::Result;

        #[test]
        fn filesystem_test() -> Result<()> {
            let out_dir = PathBuf::from(format!("{}/target/wasms", env!("CARGO_MANIFEST_DIR")));
            let out_dir = out_dir.join("wasm32-wasi/debug/keyvalue-test.wasm");
            let file_config = &format!(
                "{}/keyvalue-test/keyvalue_filesystem_slightfile.toml",
                env!("CARGO_MANIFEST_DIR")
            );
            run(
                &slight_path(),
                vec!["-c", file_config, "run", out_dir.to_str().unwrap()],
                None,
            );
            Ok(())
        }

        #[test]
        fn azblob_test() -> Result<()> {
            let out_dir = PathBuf::from(format!("{}/target/wasms", env!("CARGO_MANIFEST_DIR")));
            let out_dir = out_dir.join("wasm32-wasi/debug/keyvalue-test.wasm");
            let file_config = &format!(
                "{}/keyvalue-test/keyvalue_azblob_slightfile.toml",
                env!("CARGO_MANIFEST_DIR")
            );
            run(
                &slight_path(),
                vec!["-c", file_config, "run", out_dir.to_str().unwrap()],
                None,
            );
            Ok(())
        }

        #[test]
        fn aws_dynamodb_test() -> Result<()> {
            let out_dir = PathBuf::from(format!("{}/target/wasms", env!("CARGO_MANIFEST_DIR")));
            let out_dir = out_dir.join("wasm32-wasi/debug/keyvalue-test.wasm");
            let file_config = "./keyvalue-test/keyvalue_awsdynamodb_slightfile.toml";
            run(
                &slight_path(),
                vec!["-c", file_config, "run", out_dir.to_str().unwrap()],
                None,
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

            let out_dir = PathBuf::from(format!("{}/target/wasms", env!("CARGO_MANIFEST_DIR")));
            let out_dir = out_dir.join("wasm32-wasi/debug/keyvalue-test.wasm");
            let file_config = &format!(
                "{}/keyvalue-test/keyvalue_redis_slightfile.toml",
                env!("CARGO_MANIFEST_DIR")
            );
            env::set_var("REDIS_ADDRESS", format!("redis://127.0.0.1:{port}"));
            run(
                &slight_path(),
                vec!["-c", file_config, "run", out_dir.to_str().unwrap()],
                None,
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
    #[cfg(test)]
    mod http_tests_unix {
        use crate::slight_path;

        use std::{path::PathBuf, process::Command};

        use anyhow::Result;
        use hyper::{body, client::HttpConnector, Body, Client, Method, Request, StatusCode};

        use signal_child::Signalable;
        use tokio::{
            join,
            time::{sleep, Duration},
        };

        #[tokio::test]
        async fn http_test() -> Result<()> {
            let out_dir = PathBuf::from(format!("{}/target/wasms", env!("CARGO_MANIFEST_DIR")));
            let out_dir = out_dir.join("wasm32-wasi/debug/http_test.wasm");
            println!(
                "out_dir: {}",
                out_dir.to_owned().as_os_str().to_str().unwrap()
            );
            let config = &format!("{}/http-test/slightfile.toml", env!("CARGO_MANIFEST_DIR"));
            let mut child = Command::new(slight_path())
                .args(["-c", config, "run", out_dir.to_str().unwrap()])
                .spawn()?;
            sleep(Duration::from_secs(2)).await;

            let client = hyper::Client::new();

            let (res1, res2, res3, res4, res5, res6) = join!(
                handle_get_request(&client),
                handle_get_params(&client),
                handle_put_request(&client),
                handle_post_request(&client),
                handle_delete_request(&client),
                handle_request(&client)
            );

            child.interrupt().expect("Error interrupting child");
            child.wait().ok();

            assert!(res1.is_ok());
            assert!(res2.is_ok());
            assert!(res3.is_ok());
            assert!(res4.is_ok());
            assert!(res5.is_ok());
            assert!(res6.is_ok());

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

        async fn handle_request(client: &Client<HttpConnector>) -> Result<()> {
            let req = Request::builder()
                .method(Method::GET)
                .uri("http://0.0.0.0:3000/request")
                .body(Body::empty())
                .expect("request builder");

            let res = client.request(req).await?;
            assert!(res.status().is_success());
            Ok(())
        }
    }

    #[cfg(test)]
    #[cfg(unix)]
    mod messaging_tests {
        use std::{path::PathBuf, time::Duration};

        use crate::{slight_path, spawn};
        use anyhow::Result;
        use hyper::{Body, Method, Request};
        use mosquitto_rs::{Client, QoS};
        use tokio::{process::Command, time::sleep};

        #[tokio::test]
        async fn messaging_test() -> Result<()> {
            let out_dir = PathBuf::from(format!("{}/target/wasms", env!("CARGO_MANIFEST_DIR")));
            let http_service_dir = out_dir.join("wasm32-wasi/debug/messaging_test.wasm");
            let service_a_dir = out_dir.join("wasm32-wasi/debug/consumer_a.wasm");
            let service_b_dir = out_dir.join("wasm32-wasi/debug/consumer_b.wasm");
            let file_config = &format!(
                "{}/messaging-test/messaging.slightfile.toml",
                env!("CARGO_MANIFEST_DIR")
            );
            let consumer_config = &format!(
                "{}/messaging-test/consumer.toml",
                env!("CARGO_MANIFEST_DIR")
            );

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

            let http_child = spawn(
                &slight_path(),
                vec!["-c", file_config, "run", http_service_dir.to_str().unwrap()],
            )?;
            let service_a_child = spawn(
                &slight_path(),
                vec![
                    "-c",
                    consumer_config,
                    "run",
                    service_a_dir.to_str().unwrap(),
                ],
            )?;
            let service_b_child = spawn(
                &slight_path(),
                vec![
                    "-c",
                    consumer_config,
                    "run",
                    service_b_dir.to_str().unwrap(),
                ],
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

    #[cfg(test)]
    mod cli_tests {
        use std::process::Command;

        use crate::slight_path;

        #[test]
        fn slight_new_rust() -> anyhow::Result<()> {
            let tmpdir = tempfile::tempdir()?;
            let mut child = Command::new(slight_path())
                .args(["new", "--name-at-release", "my-demo@v0.4.0", "rust"])
                .current_dir(&tmpdir)
                .spawn()?;
            child.wait().ok();

            // compile the my-demo at target wasm32-wasi
            let p = tmpdir.path().to_owned();
            let mut child = Command::new("cargo")
                .args(["build", "--target", "wasm32-wasi"])
                .current_dir(p.join("my-demo"))
                .spawn()?;
            child.wait().ok();

            // run the my-demo
            let output = Command::new(slight_path())
                .args([
                    "-c",
                    "slightfile.toml",
                    "run",
                    "./target/wasm32-wasi/debug/my-demo.wasm",
                ])
                .current_dir(p.join("my-demo"))
                .output()?;

            // examine the output
            assert!(output.status.success());
            assert!(String::from_utf8(output.stdout)?.contains("Hello, SpiderLightning!"));
            Ok(())
        }

        #[test]
        fn slight_add_tests() -> anyhow::Result<()> {
            let capabilities = vec![
                "keyvalue",
                "configs",
                "http-server",
                "http-client",
                "distributed-locking",
                "messaging",
                "sql",
            ];
            let version = "v0.4.0";

            let tmpdir = tempfile::tempdir()?;
            for cap in capabilities {
                let output = Command::new(slight_path())
                    .args(["add", &format!("{cap}@{version}")])
                    .current_dir(&tmpdir)
                    .output()?;
                assert!(output.status.success());
            }

            let ill_version = "v0.5763.2355";
            let output = Command::new(slight_path())
                .args(["add", &format!("keyvalue@{ill_version}")])
                .current_dir(&tmpdir)
                .output()?;
            assert!(!output.status.success());
            Ok(())
        }

        #[test]
        fn slight_add_http_server_tests() -> anyhow::Result<()> {
            let mut wits = vec![
                "http-server-export.wit",
                "http-server.wit",
                "http-types.wit",
                "http-handler.wit",
            ];
            wits.sort();
            let version = "v0.4.0";

            let tmpdir = tempfile::tempdir()?;

            let output = Command::new(slight_path())
                .args(["add", &format!("{cap}@{version}", cap = "http-server")])
                .current_dir(&tmpdir)
                .output()?;
            assert!(output.status.success());

            // check file names in the http-server folder
            let p = tmpdir.path().to_owned();
            let mut files = std::fs::read_dir(p.join(format!(
                "http-server_{version}",
                version = version.strip_prefix('v').expect("version format")
            )))?
            .map(|res| res.map(|e| e.file_name().into_string().unwrap()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;

            files.sort();
            assert_eq!(files, wits);
            Ok(())
        }
    }

    #[cfg(test)]
    mod blob_store_tests {
        #[cfg(unix)]
        use std::env;
        use std::path::PathBuf;

        use crate::{run, slight_path};
        use anyhow::Result;

        #[test]
        fn s3_test() -> Result<()> {
            let out_dir = PathBuf::from(format!("{}/target/wasms", env!("CARGO_MANIFEST_DIR")));
            let out_dir = out_dir.join("wasm32-wasi/debug/blob-store-test.wasm");
            let file_config = &format!(
                "{}/blob-store-test/blob_s3.toml",
                env!("CARGO_MANIFEST_DIR")
            );
            run(
                &slight_path(),
                vec!["-c", file_config, "run", out_dir.to_str().unwrap()],
                Some(&format!("{}/blob-store-test/", env!("CARGO_MANIFEST_DIR"))),
            );
            Ok(())
        }

        #[test]
        fn az_blob_test() -> Result<()> {
            let out_dir = PathBuf::from(format!("{}/target/wasms", env!("CARGO_MANIFEST_DIR")));
            let out_dir = out_dir.join("wasm32-wasi/debug/blob-store-test.wasm");
            let file_config = &format!(
                "{}/blob-store-test/az_blob.toml",
                env!("CARGO_MANIFEST_DIR")
            );
            run(
                &slight_path(),
                vec!["-c", file_config, "run", out_dir.to_str().unwrap()],
                Some(&format!("{}/blob-store-test/", env!("CARGO_MANIFEST_DIR"))),
            );
            Ok(())
        }
    }

    #[cfg(test)]
    #[cfg(unix)]
    mod wildcard_tests {
        use crate::{slight_path, spawn};

        use hyper::{Body, Method, Request};

        use std::{path::PathBuf, time::Duration};
        use tokio::{process::Command, time::sleep};

        #[tokio::test]
        async fn slight_add_wildcard() -> anyhow::Result<()> {
            let out_dir = PathBuf::from(format!("{}/target/wasms", env!("CARGO_MANIFEST_DIR")));
            let http_service_dir = out_dir.join("wasm32-wasi/debug/wildcard_test.wasm");
            let file_config = &format!(
                "{}/wildcard-test/slightfile.toml",
                env!("CARGO_MANIFEST_DIR")
            );
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

            let http_child = spawn(
                &slight_path(),
                vec!["-c", file_config, "run", http_service_dir.to_str().unwrap()],
            )?;
            sleep(Duration::from_secs(2)).await;

            let http_client = hyper::Client::new();
            let req = Request::builder()
                .method(Method::PUT)
                .uri("http://0.0.0.0:3002/register")
                .body(Body::empty())
                .expect("request builder");

            // curl -X PUT http://0.0.0.0:3002/register
            let res = http_client.request(req).await?;
            assert!(res.status().is_success());
            let token_a = String::from_utf8(
                hyper::body::to_bytes(res.into_body())
                    .await?
                    .into_iter()
                    .collect(),
            )?;

            let req = Request::builder()
                .method(Method::PUT)
                .uri("http://0.0.0.0:3002/register")
                .body(Body::empty())
                .expect("request builder");
            // curl -X PUT http://0.0.0.0:3002/register
            let res = http_client.request(req).await?;
            assert!(res.status().is_success());
            let token_b = String::from_utf8(
                hyper::body::to_bytes(res.into_body())
                    .await?
                    .into_iter()
                    .collect(),
            )?;

            let req = Request::builder()
                .method(Method::PUT)
                .uri("http://0.0.0.0:3002/send/sender".to_string())
                .body(Body::from("a message!"))
                .expect("request builder");
            // curl -X PUT http://0.0.0.0:3002/send/sender -x "a message!"
            let res = http_client.request(req).await?;
            assert!(res.status().is_success());

            let req = Request::builder()
                .method(Method::GET)
                .uri(format!("http://0.0.0.0:3002/get/{token_a}"))
                .body(Body::empty())
                .expect("request builder");
            // curl -X GET http://0.0.0.0:3002/get/{token_a}
            let res = http_client.request(req).await?;
            assert!(res.status().is_success());
            let body = res.into_body();
            assert_eq!(
                hyper::body::to_bytes(body).await?,
                "a message!".to_string().into_bytes()
            );

            let req = Request::builder()
                .method(Method::GET)
                .uri(format!("http://0.0.0.0:3002/get/{token_b}"))
                .body(Body::empty())
                .expect("request builder");
            // curl -X GET http://0.0.0.0:3002/get/{token_b}
            let res = http_client.request(req).await?;
            assert!(res.status().is_success());
            let body = res.into_body();
            assert_eq!(
                hyper::body::to_bytes(body).await?,
                "a message!".to_string().into_bytes()
            );

            sleep(Duration::from_secs(2)).await;
            http_child();
            mosquitto_child();

            Ok(())
        }
    }
}
