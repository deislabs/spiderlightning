use anyhow::Result;

use keyvalue::*;
wit_bindgen_rust::import!("wit/keyvalue_{{release}}/keyvalue.wit");
wit_error_rs::impl_error!(keyvalue::KeyvalueError);

fn main() -> Result<()> {
    let my_keyvalue = Keyvalue::open("placeholder-name")?;
    my_keyvalue.set("hello-spiderlightning", b"Hello, SpiderLightning!")?;
    println!(
        "{}",
        std::str::from_utf8(&my_keyvalue.get("hello-spiderlightning")?)?,
    );

    Ok(())
}
