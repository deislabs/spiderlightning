#![allow(clippy::enum_variant_names)]
use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");
wit_error_rs::impl_error!(Error);

fn main() -> Result<()> {
    // test get, set, delete
    let rd = get_kv("rand")?; // TODO: this should be a random name
    let value = "wasi-cloud".as_bytes();
    set(&rd, "key", value)?;
    println!(
        "Hello, world! the value is: {}",
        std::str::from_utf8(&get(&rd, "key")?)?
    );
    delete(&rd, "key")?;
    let value = get(&rd, "key");
    assert!(value.is_err());

    // test get_kv() will refer to the same underlying resource
    let rd1 = get_kv("random1")?;
    let rd2 = get_kv("random2")?;
    set(&rd1, "key1", "value1".as_bytes())?;
    set(&rd2, "key2", "value2".as_bytes())?;

    let value1 = get(&rd1, "key2")?;
    let value2 = get(&rd2, "key1")?;
    assert_eq!(std::str::from_utf8(&value1)?, "value2");
    assert_eq!(std::str::from_utf8(&value2)?, "value1");

    // test get empty key
    let rd = get_kv("random3")?;
    let value = get(&rd, "");
    assert!(value.is_err());

    // test delete empty key
    let rd = get_kv("random4")?;
    let ret = delete(&rd, "key");
    assert!(ret.is_err());
    Ok(())
}