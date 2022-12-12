#![allow(clippy::enum_variant_names)]
use anyhow::Result;

use keyvalue::*;
wit_bindgen_rust::import!("../../wit/keyvalue.wit");
wit_error_rs::impl_error!(keyvalue::KeyvalueError);

fn main() -> Result<()> {
    // test get, set, delete
    let kv = Keyvalue::open("rand")?;
    let value = "spiderlightning".as_bytes();
    kv.set("key", value)?;
    println!(
        "Hello, world! the value is: {}",
        std::str::from_utf8(&kv.get("key")?)?
    );
    kv.delete("key")?;
    let value = kv.get("key");
    assert!(value.is_err());

    // test keys
    let kv = Keyvalue::open("rand")?;
    let value = "spiderlightning".as_bytes();
    kv.set("key", value)?;
    kv.set("key2", value)?;
    let keys = kv.keys()?;
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"key".to_string()));
    kv.delete("key")?;
    kv.delete("key2")?;

    // test get_kv() will have a unique allocation in the resource table.
    // so two `get_kv()` with different names will return different allocations.
    let kv1 = Keyvalue::open("random1")?;
    let kv2 = Keyvalue::open("random2")?;
    kv1.set("key1", "value1".as_bytes())?;
    kv2.set("key2", "value2".as_bytes())?;

    assert!(kv1.get("key2").is_err());
    kv1.delete("key1")?;
    kv2.delete("key2")?;

    // test two get_kv() with the same name will return the same allocation.
    // but the resource descriptors are not the same.
    let kv1 = Keyvalue::open("random1")?;
    let kv2 = Keyvalue::open("random1")?;
    kv1.set("key1", "value1".as_bytes())?;
    kv2.set("key2", "value2".as_bytes())?;
    assert!(kv1.get("key2")? == "value2".as_bytes());
    kv1.delete("key1")?;
    kv2.delete("key2")?;

    // test get empty key
    let kv3 = Keyvalue::open("random3")?;
    let value = kv3.get("");
    assert!(value.is_err());

    println!("finished running kv-test");
    Ok(())
}
