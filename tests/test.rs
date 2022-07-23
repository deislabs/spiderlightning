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

#[cfg(test)]
mod statestore_test {
    use crate::{run, SLIGHT};
    use anyhow::Result;

    const STATESTORE_TEST_MODULE: &str =
        "./tests/state_store-test/target/wasm32-wasi/debug/state_store-test.wasm";

    #[test]
    fn statestore_filesystem_test() -> Result<()> {
        let slightfile = "./tests/state_store-test/slightfile-filesystem.toml";
        run(
            SLIGHT,
            vec!["-c", slightfile, "run", "-m", STATESTORE_TEST_MODULE],
        );
        Ok(())
    }

    #[test]
    fn statestore_azblob_test() -> Result<()> {
        let slightfile = "./tests/state_store-test/slightfile-azblob.toml";
        run(
            SLIGHT,
            vec!["-c", slightfile, "run", "-m", STATESTORE_TEST_MODULE],
        );
        Ok(())
    }
}

// TODO: We need to mq_test, etcdlockd_test, and ckpubsub_test modules
