use anyhow::Result;

use blob::*;
wit_bindgen_rust::import!("../../wit/blob.wit");

fn main() -> Result<()> {
    // application devleoper does not need to know the host implementation details.

    let resource_descriptor = get_blob()?;
    let value = "wasi-cloud".as_bytes();
    set(&resource_descriptor, "key", value)?;
    println!(
        "Hello, world! the value is: {}",
        std::str::from_utf8(&get(&resource_descriptor, "key")?)?
    );
    delete(&resource_descriptor, "key")?;
    let value = get(&resource_descriptor, "key");
    assert_eq!(value.is_err(), true);
    drop(resource_descriptor);
    Ok(())
}

impl From<blob::Error> for anyhow::Error {
    fn from(e: blob::Error) -> Self {
        match e {
            blob::Error::OtherError => anyhow::anyhow!("other error"),
            blob::Error::IoError => anyhow::anyhow!("io error"),
            blob::Error::DescriptorError => anyhow::anyhow!("descriptor error"),
        }
    }
}
