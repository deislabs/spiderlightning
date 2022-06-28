use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");
wit_error_rs::impl_error!(Error);

fn main() -> Result<()> {
    // application devleoper does not need to know the host implementation details.

    let rd1 = get_kv("my-container")?;
    // let rd2 = get_kv("my-container2")?;
    let value = "wasi-cloud".as_bytes();
    set(&rd1, "key", value)?;
    // set(&rd2, "key", value)?;
    println!(
        "Hello, world! the value for rd1 is: {}",
        std::str::from_utf8(&get(&rd1, "key")?)?,
        // std::str::from_utf8(&get(&rd2, "key")?)?,
    );
    delete(&rd1, "key")?;
    // delete(&rd2, "key")?;
    let value = get(&rd1, "key");
    assert_eq!(value.is_err(), true);
    drop(rd1); // drop != close
               // drop(rd2);
    Ok(())
}
