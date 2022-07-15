mod common;

#[cfg(test)]
mod kv_test {
    use crate::common::run;
    use anyhow::Result;

    const SLIGHT: &str = "../target/release/slight";
    const KV_TEST_MODULE: &str = "../target/wasm32-wasi/release/kv-demo.wasm";

    #[test]
    fn test_kv_filesystem() -> Result<()> {
        let file_config = "../examples/kv-demo/filekv-wc.toml";
        run(SLIGHT, vec!["-m", KV_TEST_MODULE, "-c", file_config]);
        Ok(())
    }
}
