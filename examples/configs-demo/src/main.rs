use anyhow::Result;

use configs::*;
wit_bindgen_rust::import!("../../wit/configs.wit");
wit_error_rs::impl_error!(Error);

fn main() -> Result<()> {
    let ev_configs = Configs::open("envvars")?;
    ev_configs.set("THIS_IS_A_TEST_CONFIG", "Hello, World!".as_bytes())?;
    dbg!(String::from_utf8(ev_configs.get("THIS_IS_A_TEST_CONFIG")?)?);
    // ^^^ these configs are transient

    let ptr = Box::into_raw(Box::new(123));
    // ^^^ getting a random number to differentiate non-transient configs created below

    let rand_key = format!("THIS_IS_ANOTHER_TEST_CONFIG_{}", ptr as usize);
    let us_configs = Configs::open("usersecrets")?;
    us_configs.set(&rand_key, "Hello, World!".as_bytes())?;
    dbg!(String::from_utf8(us_configs.get(&rand_key)?)?);
    // ^^^ if you look in your spiderlightning config file after this, you should have the configs show up!
    Ok(())
}
