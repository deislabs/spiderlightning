use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");

fn main() -> Result<()> {
    // application devleoper does not need to know the host implementation details.

    let resource_descriptor = get_kv("my-container")?;
    let value = "wasi-cloud".as_bytes();
    set(&resource_descriptor, "key", value)?;
    println!(
        "Hello, world! the value is: {}",
        std::str::from_utf8(&get(&resource_descriptor, "key")?)?
    );
    delete(&resource_descriptor, "key")?;
    let value = get(&resource_descriptor, "key");
    assert_eq!(value.is_err(), true);
    drop(resource_descriptor); // drop != close
    Ok(())
}

impl From<kv::Error> for anyhow::Error {
    fn from(e: kv::Error) -> Self {
        match e {
            kv::Error::OtherError => anyhow::anyhow!("other error"),
            kv::Error::IoError => anyhow::anyhow!("io error"),
            kv::Error::DescriptorError => anyhow::anyhow!("descriptor error"),
        }
    }
}
