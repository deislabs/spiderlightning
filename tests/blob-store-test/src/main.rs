use anyhow::{bail, Result};

use blob_store::*;
wit_bindgen_rust::import!("../../wit/blob-store.wit");
wit_error_rs::impl_error!(Error);

fn main() -> Result<()> {
    let bucket = blob_store::Container::open("slight-test-bucket")?;

    // verify container name
    assert_eq!(bucket.name()?, "slight-test-bucket");

    for name in bucket.list_objects()? {
        println!("Found object: {}", name);
        bucket.delete_object(&name)?;
    }

    // upload 3 files
    for i in 0..3 {
        let body = std::fs::read(format!("testfile{i}.txt")).expect("should have been able to read the file");
        bucket
            .write_object(&format!("testfile{}.txt", i))?
            .write(body.as_ref())?;
    }

    // read 3 files
    for name in bucket.list_objects()? {
        println!("Found object: {}", name);
        let read_stream = bucket.read_object(&name)?;
        let contents = read_stream.read(1024 * 4)?.unwrap();

        // compare the contents with content on filesystem
        let body = std::fs::read(format!("{}", name)).expect("should have been able to read the file");
        assert_eq!(body, contents);
    }

    // read one file
    if bucket.has_object("testfile1.txt")? {
        let read_stream = bucket.read_object("testfile1.txt")?;
        let contents = read_stream.read(1024 * 4)?.unwrap();

        // compare the content with content on filesystem
        let body = std::fs::read("testfile1.txt").expect("should have been able to read the file");
        assert_eq!(body, contents);
    } else {
        bail!("testfile1.txt not found")
    }

    // return metadata for the testfile1.txt
    let body = std::fs::read("testfile1.txt").expect("should have been able to read the file");
    let metadata = bucket.object_info("testfile1.txt")?;
    assert_eq!(metadata.name, "testfile1.txt");
    assert_eq!(metadata.size, body.len() as u64);
    assert_eq!(metadata.container, "slight-test-bucket");
    println!("metadata created-at: {:?}", metadata.created_at);

    // delete all three files
    // TODO: re-enable this once the delete_objects() method is implemented in azblob
    // bucket.delete_objects(&["testfile0.txt", "testfile1.txt", "testfile2.txt"])?;

    for name in bucket.list_objects()? {
        bucket.delete_object(&name)?;
    }
    Ok(())
}
