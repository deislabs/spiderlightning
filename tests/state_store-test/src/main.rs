#![allow(clippy::enum_variant_names)]
use anyhow::Result;

use state_store::*;
wit_bindgen_rust::import!("../../wit/state_store.wit");
wit_error_rs::impl_error!(Error);

fn main() -> Result<()> {
    // test get, set, delete
    let ss = StateStore::open("rand")?;
    let value = "spiderlightning".as_bytes();
    ss.set("key", value)?;
    println!(
        "Hello, world! the value is: {}",
        std::str::from_utf8(&ss.get("key")?)?
    );
    ss.delete("key")?;
    let value = ss.get("key");
    assert!(value.is_err());

    let ss1 = StateStore::open("random1")?;
    let ss2 = StateStore::open("random2")?;
    ss1.set("key1", "value1".as_bytes())?;
    ss2.set("key2", "value2".as_bytes())?;

    assert!(ss1.get("key2").is_err());
    ss1.delete("key1")?;
    ss2.delete("key2")?;

    let ss1 = StateStore::open("random1")?;
    let ss2 = StateStore::open("random1")?;
    ss1.set("key1", "value1".as_bytes())?;
    ss2.set("key2", "value2".as_bytes())?;
    assert!(ss1.get("key2")? == "value2".as_bytes());
    ss1.delete("key1")?;
    ss2.delete("key2")?;

    // test get empty key
    let ss3 = StateStore::open("random3")?;
    let value = ss3.get("");
    assert!(value.is_err());

    // test delete empty key
    let ss4 = StateStore::open("random4")?;
    let ret = ss4.delete("key");
    assert!(ret.is_err());

    println!("finished running state_store-test");
    Ok(())
}
