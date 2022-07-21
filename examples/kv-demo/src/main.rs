use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");
wit_error_rs::impl_error!(Error);

fn main() -> Result<()> {
    let kv1 = Kv::open("my-container")?;
    let kv2 = Kv::open("my-container2")?;
    let value = b"spiderlightning";
    kv1.set("key", value)?;
    kv2.set("key", value)?;
    println!(
        "Hello, world! the value for kv1 is: {}, kv2 is {}",
        std::str::from_utf8(&kv1.get("key")?)?,
        std::str::from_utf8(&kv2.get("key")?)?,
    );
    kv1.delete("key")?;
    kv2.delete("key")?;
    let value = kv1.get("key");
    assert!(value.is_err());
    Ok(())
}
