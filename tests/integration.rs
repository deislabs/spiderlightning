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
mod integration_tests {
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

    // TODO: We need to mq_test, lockd_test, and pubsub_test modules
}
