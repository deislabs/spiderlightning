use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");
wit_bindgen_rust::export!("../../wit/config.wit");

/// A Filesystem configuration
pub struct Config {}

impl config::Config for Config {
    /// the Filesystem configuration will have a {path: String} field.
    fn get_capability() -> Result<config::Map, config::Error> {
        let mut map = config::Map::new();
        map.push(("path".to_string(), ".".to_string()));
        Ok(map)
    }
}

fn main() -> Result<()> {
    let resource_descriptor = get_kv()?;
    let value = "wasi-cloud".as_bytes();
    set(&resource_descriptor, "key", value)?;
    println!(
        "Hello, world! the value is: {}",
        std::str::from_utf8(&get(&resource_descriptor, "key")?)?
    );
    delete(&resource_descriptor, "key")?;
    let value = get(&resource_descriptor, "key");
    assert_eq!(value.is_err(), true);
    drop(resource_descriptor);
    Ok(())
}

impl From<kv::Error> for anyhow::Error {
    fn from(_: kv::Error) -> Self {
        anyhow::anyhow!("kv error")
    }
}
