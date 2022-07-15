use std::{
    io::{stderr, stdout, Write},
    process::Command,
};

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

#[cfg(test)]
mod filekv_test {
    use crate::run;
    use anyhow::Result;

    const SLIGHT: &str = "./target/release/slight";
    const KV_TEST_MODULE: &str = "./target/wasm32-wasi/release/kv-demo.wasm";

    #[test]
    fn test_kv_filesystem() -> Result<()> {
        let file_config = "./examples/kv-demo/filekv.toml";
        run(SLIGHT, vec!["-c", file_config, "run", "-m", KV_TEST_MODULE]);
        Ok(())
    }
}

// TODO: We need to add azblobkv_test, filemq_test, azsbusmq_test, etcdlockd_test, and ckpubsub_test modules
