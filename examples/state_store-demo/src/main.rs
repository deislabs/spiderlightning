use anyhow::Result;

use state_store::*;
wit_bindgen_rust::import!("../../wit/state_store.wit");
wit_error_rs::impl_error!(state_store::Error);

fn main() -> Result<()> {
    let ss1 = StateStore::open("my-container")?;
    let ss2 = StateStore::open("my-container2")?;
    let value = b"spiderlightning";
    ss1.set("key", value)?;
    ss2.set("key", value)?;
    println!(
        "The for ss1 is: {}, ss2 is {}",
        std::str::from_utf8(&ss1.get("key")?)?,
        std::str::from_utf8(&ss2.get("key")?)?,
    );
    ss1.delete("key")?;
    ss2.delete("key")?;
    let value = ss1.get("key");
    assert!(value.is_err());
    Ok(())
}
