mod common;

#[cfg(test)]
mod kv_test {
    use crate::common::run;
    use anyhow::Result;

    const WASI_CLOUD_BINARY: &str = "./target/debug/wasi-cloud";
    const KV_TEST_MODULE: &str = "tests/kv-test/target/wasm32-wasi/debug/kv-test.wasm";

    #[test]
    fn test_kv_filesystem() -> Result<()> {
        let config = format!("./tests/wc.toml");
        run(WASI_CLOUD_BINARY, vec!["-m", KV_TEST_MODULE, "-c", &config]);
        Ok(())
    }
}
