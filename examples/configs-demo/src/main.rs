use anyhow::Result;

use configs::*;
wit_bindgen_rust::import!("../../wit/configs.wit");
wit_error_rs::impl_error!(Error);

fn main() -> Result<()> {
    let ptr = Box::into_raw(Box::new(123));
    // ^^^ getting a random number to differentiate non-transient configs created below

    let rand_key = format!("THIS_IS_ANOTHER_TEST_CONFIG_{}", ptr as usize);
    let configs = Configs::open()?;
    configs.set(&rand_key, "Hello, World!".as_bytes())?;
    dbg!(String::from_utf8(configs.get(&rand_key)?)?);
    // ^^^ if you look in your spiderlightning config file after this, you should have the configs show up!
    Ok(())
}
