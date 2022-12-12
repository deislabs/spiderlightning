use anyhow::Result;

use keyvalue::*;
wit_bindgen_rust::import!("../../wit/keyvalue.wit");
wit_error_rs::impl_error!(KeyvalueError);

fn main() -> Result<()> {
    let keyvalue1 = Keyvalue::open("my-container")?;
    let keyvalue2 = Keyvalue::open("my-container2")?;
    let value = b"spiderlightning";
    keyvalue1.set("key", value)?;
    keyvalue2.set("key", value)?;

    let keys = keyvalue1.keys()?;
    assert_eq!(keys.len(), 1);
    
    println!(
        "Hello, world! the value for keyvalue1 is: {}, keyvalue2 is {}",
        std::str::from_utf8(&keyvalue1.get("key")?)?,
        std::str::from_utf8(&keyvalue2.get("key")?)?,
    );
    keyvalue1.delete("key")?;
    keyvalue2.delete("key")?;
    let value = keyvalue1.get("key");
    assert!(value.is_err());
    Ok(())
}
