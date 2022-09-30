use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("wit/kv_{{release}}/kv.wit");
wit_error_rs::impl_error!(kv::Error);

fn main() -> Result<()> {
    let my_kv = Kv::open("my_folder")?;
    // ^^^ will create a folder under /tmp/my_folder
    // that will contain your key-value pairs

    my_kv.set("hello-spiderlightning", "Hello, SpiderLightning!")?;
    println!(
        "{}",
        std::str::from_utf8(&my_kv.get("hello-spiderlightning")?)?,
    );

    Ok(())
}
