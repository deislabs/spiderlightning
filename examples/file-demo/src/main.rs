
use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");

fn main() -> Result<()> {
    let resource_descriptor = get_kv()?;
    let value = "wasi-cloud".as_bytes();
    set("key", value, &resource_descriptor)?;
    let value = get("key", &resource_descriptor)?;
    println!("Hello, world! the value is: {}", std::str::from_utf8(&value)?);
    Ok(())
}


impl From<kv::Error> for anyhow::Error {
    fn from(_: kv::Error) -> Self {
        anyhow::format_err!("kv::Error")
    }
}