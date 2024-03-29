use std::{
    env,
    fs::{self, File},
    process::Command,
};
use wizer::Wizer;

use anyhow::Result;

pub fn handle_buildjs(engine_path: &str, js_path: &str, output_file: &str) -> Result<()> {
    if env::var("JS_COMPILED").eq(&Ok("1".into())) {
        env::remove_var("JS_COMPILED");

        let wasm = fs::read(engine_path)?;

        let wasm = Wizer::new()
            .dir(".")
            .allow_wasi(true)?
            .inherit_stdio(true)
            .wasm_bulk_memory(true)
            .run(&wasm)?;

        fs::write(output_file, wasm)?;

        return Ok(());
    }

    env::set_var("JS_COMPILED", "1");

    let script = File::open(js_path)?;

    let self_cmd = std::env::current_exe()?;
    let status = Command::new(self_cmd)
        .arg("buildjs")
        .arg("--engine")
        .arg(engine_path)
        .arg("--output")
        .arg(output_file)
        .arg(js_path)
        .stdin(script)
        .status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("failed to compile"));
    }

    Ok(())
}
