#![allow(clippy::enum_variant_names)]
use anyhow::Result;

use keyvalue::*;
wit_bindgen_rust::import!("../../wit/keyvalue.wit");
wit_error_rs::impl_error!(keyvalue::KeyvalueError);

fn main() -> Result<()> {
    // test get, set, delete
    let keyvalue = Keyvalue::open("slight-keyvalue-test-4")?;
    let value = "spiderlightning".as_bytes();
    keyvalue.set("key", value)?;
    println!(
        "Hello, world! the value is: {}",
        std::str::from_utf8(&keyvalue.get("key")?)?
    );
    keyvalue.delete("key")?;
    let value = keyvalue.get("key");
    assert!(value.is_err());

    // test keys
    let keyvalue = Keyvalue::open("slight-keyvalue-test-4")?;
    let value = "spiderlightning".as_bytes();
    keyvalue.set("key", value)?;
    keyvalue.set("key2", value)?;
    let keys = keyvalue.keys()?;
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"key".to_string()));
    keyvalue.delete("key")?;
    keyvalue.delete("key2")?;

    let keyvalue1 = Keyvalue::open("slight-keyvalue-test-1")?;
    let keyvalue2 = Keyvalue::open("slight-keyvalue-test-2")?;
    keyvalue1.set("key1", "value1".as_bytes())?;
    keyvalue2.set("key2", "value2".as_bytes())?;

    assert!(keyvalue1.get("key2").is_err());
    keyvalue1.delete("key1")?;
    keyvalue2.delete("key2")?;

    let keyvalue1 = Keyvalue::open("slight-keyvalue-test-1")?;
    let keyvalue2 = Keyvalue::open("slight-keyvalue-test-1")?;
    keyvalue1.set("key1", "value1".as_bytes())?;
    keyvalue2.set("key2", "value2".as_bytes())?;
    assert!(keyvalue1.get("key2")? == "value2".as_bytes());
    keyvalue1.delete("key1")?;
    keyvalue2.delete("key2")?;

    // test get empty key
    let keyvalue3 = Keyvalue::open("slight-keyvalue-test-3")?;
    let value = keyvalue3.get("");
    assert!(value.is_err());

    println!("finished running keyvalue-test");
    Ok(())
}
