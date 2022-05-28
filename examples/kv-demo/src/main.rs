use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");

fn main() -> Result<()> {
    // application devleoper does not need to know the host implementation details.

    let rd1 = get_kv("azblob://my-container")?;
    let rd2 = get_kv("file:///tmp")?;
    let value = "wasi-cloud".as_bytes();
    set(&rd1, "key", value)?;
    println!(
        "Hello, world! the value is: {}",
        std::str::from_utf8(&get(&rd1, "key")?)?
    );
    delete(&rd1, "key")?;
    let value = get(&rd1, "key");
    assert!(value.is_err());
    drop(rd1);

    let res = get(&rd2, "key");
    assert!(res.is_err());

    set(&rd2, "key", "wasi-cloud-2".as_bytes())?;
    println!(
        "Hello, world! the value is: {}",
        std::str::from_utf8(&get(&rd2, "key")?)?
    );
    delete(&rd2, "key")?;
    let value = get(&rd2, "key");
    assert!(value.is_err());
    
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
