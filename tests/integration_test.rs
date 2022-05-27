mod commons;

#[cfg(test)]
mod kv_test {
    use std::{process::Command, io::{Write, stdout, stderr}};

    use anyhow::Result;
    use kv_filesystem::KvFilesystem;
    use runtime::Builder;
    use url::Url;
    use temp_dir::TempDir;

    const WASI_CLOUD_BINARY: &str = "./target/debug/wasi-cloud";
    const KV_TEST_MODULE: &str = "tests/kv-test/target/wasm32-wasi/debug/kv-test.wasm";

    #[test]
    fn test_kv_filesystem() -> Result<()> {
        let d = TempDir::new().unwrap();
        let url = format!("file://{}", d.path().to_string_lossy().to_string());

        // let mut cmd = Command::new(WASI_CLOUD_BINARY);
        // let output = cmd.arg("-m")
        //     .arg(KV_TEST_MODULE)
        //     .arg("-c")
        //     .arg(url)
        //     .stdout(std::process::Stdio::piped())
        //     .stderr(std::process::Stdio::piped())
        //     .output()
        //     .expect("failed to execute process");
        
        // let code = output.status.code().expect("should have status code");
        // stdout().write_all(&output.stdout).unwrap();
        // if code != 0 {
        //     stderr().write_all(&output.stderr).unwrap();
        //     panic!("failed to run wasi-cloud");
        // }
        commons::run(
            WASI_CLOUD_BINARY, 
            vec!["-m", KV_TEST_MODULE, "-c", url]
        );
        Ok(())
    }
}