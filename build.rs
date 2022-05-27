use std::{
    io::{stderr, stdout, Write},
    process::Command,
};

const WIT_DIRECTORY: &str = "wit/*";
const KV_TEST_PATH: &str = "tests/kv-test";
const KV_EXAMPLE_DEMO: &str = "examples/kv-demo";
const MQ_EXAMPLE_FILESYSTEM_SENDER: &str = "examples/mq-filesystem-sender-demo";
const MQ_EXAMPLE_FILESYSTEM_RECEIVER: &str = "examples/mq-filesystem-receiver-demo";

fn main() {
    println!("cargo:rerun-if-changed={}", WIT_DIRECTORY);
    println!("cargo:rerun-if-changed={}/src/main.rs", KV_TEST_PATH);

    cargo_wasi_build(KV_TEST_PATH);
}

fn cargo_wasi_build(path: &str) {
    let mut cmd = Command::new("cargo");
    let output = cmd
        .arg("build")
        .arg("--target=wasm32-wasi")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(path)
        .output()
        .expect("failed to execute process");
    let code = output.status.code().expect("should have status code");
    if code != 0 {
        stdout().write_all(&output.stdout).unwrap();
        stderr().write_all(&output.stderr).unwrap();
        panic!("failed to build wasi-cloud with command cargo build --target=wasm32-wasi");
    }
}
