use anyhow::Result;

use configs::*;
wit_bindgen_rust::import!("../../wit/configs.wit");
wit_error_rs::impl_error!(configs::ConfigsError);

fn main() -> Result<()> {
    let configs = Configs::open("my-secret-store").expect("failed to open configs capability");
    configs.set("key", "value".as_bytes())?;
    configs.get("key")?;
    Ok(())
}
