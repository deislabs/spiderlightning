#![allow(clippy::enum_variant_names)]
use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");
wit_error_rs::impl_error!(Error);

fn main() -> Result<()> {
    // test get, set, delete
    let kv = Kv::open("rand")?;
    let value = "spiderlightning".as_bytes();
    kv.set("key", value)?;
    println!(
        "Hello, world! the value is: {}",
        std::str::from_utf8(&kv.get("key")?)?
    );
    kv.delete("key")?;
    let value = kv.get("key");
    assert!(value.is_err());

    // test get_kv() will have a unique allocation in the resource table.
    // so two `get_kv()` with different names will return different allocations.
    let kv1 = Kv::open("random1")?;
    let kv2 = Kv::open("random2")?;
    kv1.set("key1", "value1".as_bytes())?;
    kv2.set("key2", "value2".as_bytes())?;

    assert!(kv1.get("key2").is_err());
    kv1.delete("key1")?;
    kv2.delete("key2")?;

    // test two get_kv() with the same name will return the same allocation.
    // but the resource descriptors are not the same.
    let kv1 = Kv::open("random1")?;
    let kv2 = Kv::open("random1")?;
    kv1.set("key1", "value1".as_bytes())?;
    kv2.set("key2", "value2".as_bytes())?;
    assert!(kv1.get("key2")? == "value2".as_bytes());
    kv1.delete("key1")?;
    kv2.delete("key2")?;

    // test get empty key
    let kv3 = Kv::open("random3")?;
    let value = kv3.get("");
    assert!(value.is_err());

    // test delete empty key
    let kv4 = Kv::open("random4")?;
    let ret = kv4.delete("key");
    assert!(ret.is_err());

    println!("finished running kv-test");
    Ok(())
}
