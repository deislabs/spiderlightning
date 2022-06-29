#![allow(clippy::enum_variant_names)]
use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");

fn main() {
    // test get, set, delete
    let rd = get_kv("rand").unwrap(); // TODO: this should be a random name
    let value = "wasi-cloud".as_bytes();
    set(&rd, "key", value).unwrap();
    println!(
        "Hello, world! the value is: {}",
        std::str::from_utf8(&get(&rd, "key").unwrap()).unwrap()
    );
    delete(&rd, "key").unwrap();
    let value = get(&rd, "key");
    assert!(value.is_err());

    // test get_kv() will have a unique allocation in the resource table.
    // so two `get_kv()` with different names will return different allocations.
    let rd1 = get_kv("random1").unwrap();
    let rd2 = get_kv("random2").unwrap();
    set(&rd1, "key1", "value1".as_bytes()).unwrap();
    set(&rd2, "key2", "value2".as_bytes()).unwrap();

    assert!(get(&rd1, "key2").is_err());
    delete(&rd1, "key1").unwrap();
    delete(&rd2, "key2").unwrap();

    // test two get_kv() with the same name will return the same allocation.
    // but the resource descriptors are not the same.
    let rd1 = get_kv("random1").unwrap();
    let rd2 = get_kv("random1").unwrap();
    assert!(rd1 != rd2);
    set(&rd1, "key1", "value1".as_bytes()).unwrap();
    set(&rd2, "key2", "value2".as_bytes()).unwrap();
    assert!(get(&rd1, "key2").unwrap() == "value2".as_bytes());
    delete(&rd1, "key1").unwrap();
    delete(&rd2, "key2").unwrap();

    // test get empty key
    let rd = get_kv("random3").unwrap();
    let value = get(&rd, "");
    assert!(value.is_err());

    // test delete empty key
    let rd = get_kv("random4").unwrap();
    let ret = delete(&rd, "key");
    assert!(ret.is_err());

    println!("finished running kv-test");
}
