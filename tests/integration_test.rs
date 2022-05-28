mod common;

#[cfg(test)]
mod kv_test {
    use anyhow::Result;
    
    
    use temp_dir::TempDir;
    
    use crate::common::run;

    const WASI_CLOUD_BINARY: &str = "./target/debug/wasi-cloud";
    const KV_TEST_MODULE: &str = "tests/kv-test/target/wasm32-wasi/debug/kv-test.wasm";

    #[test]
    fn test_kv_filesystem() -> Result<()> {
        let d = TempDir::new().unwrap();
        let url = format!("file://{}", d.path().to_string_lossy());
        run(WASI_CLOUD_BINARY, vec!["-m", KV_TEST_MODULE, "-c", &url]);
        Ok(())
    }
}
