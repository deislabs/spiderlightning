use anyhow::Result;

use blob_store::*;
wit_bindgen_rust::import!("../../wit/blob-store.wit");
wit_error_rs::impl_error!(Error);

fn main() -> Result<()> {
    let bucket = blob_store::Container::open("slight-example-bucket")?;
    if bucket.has_object("file.txt")? {
        let read_stream = bucket.read_object("file.txt")?;
        let contents = read_stream.read(1024)?.unwrap();

        println!("The contents of file.txt are: {}", std::str::from_utf8(&contents)?);
    } else {
        let writer = bucket.write_object("file.txt")?;
        writer.write(b"Hello, world!")?;
    }
    Ok(())
}
