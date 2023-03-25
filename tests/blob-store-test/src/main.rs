use anyhow::{bail, Result};

use blob_store::*;
wit_bindgen_rust::import!("../../wit/blob-store.wit");
wit_error_rs::impl_error!(Error);

fn main() -> Result<()> {
    let body = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Mauris nulla dolor, varius vel vehicula vitae, dapibus id tellus. Vivamus nisl risus, pretium in nisi non, venenatis rhoncus ligula. Sed pharetra nibh nulla. Integer vitae mollis nisi. Nunc ante nulla, cursus id dolor sit amet, suscipit molestie est. Vestibulum sollicitudin fermentum arcu ut dictum. Sed finibus feugiat libero, sit amet fermentum ex consequat auctor. Donec varius fringilla sagittis. Sed gravida efficitur erat et viverra. Nulla malesuada metus vitae lacus malesuada, at faucibus velit tincidunt. Cras fermentum blandit purus. Integer rutrum semper lorem, rutrum aliquet elit egestas ac. Integer tincidunt sem nec turpis aliquet consectetur. Nunc accumsan est leo, ut tincidunt turpis interdum in. Ut finibus nibh et ullamcorper pharetra.
    Vivamus quis dictum ipsum. Proin congue vulputate elit rhoncus congue. Mauris elementum libero mollis, mattis nibh eu, commodo erat. Duis sodales feugiat diam, ut tempor purus viverra eu. Vestibulum sodales sit amet eros quis placerat. Phasellus venenatis condimentum nisl eget tristique. Suspendisse potenti. Cras a orci nec elit pulvinar condimentum at et diam. Mauris faucibus aliquam posuere. Donec accumsan, metus ac scelerisque iaculis, felis enim ullamcorper ante, quis tristique elit justo et ex. Donec et velit pharetra, viverra lacus vitae, laoreet elit.
    Duis sodales odio velit, nec ornare leo venenatis ut. Pellentesque in libero sit amet libero vestibulum lobortis. Vivamus laoreet ex eget mi suscipit, vitae volutpat felis iaculis. Etiam rhoncus ac arcu nec commodo. Phasellus posuere, mi eget egestas tincidunt, orci tellus placerat turpis, sit amet blandit nibh magna eu est. Nunc ultrices tincidunt rhoncus. Praesent faucibus id augue quis laoreet. Maecenas ac pellentesque eros, id pellentesque magna. Integer faucibus auctor ante. Mauris vitae pharetra erat. Nulla facilisi. Fusce ac ex libero. Duis posuere lacus lectus, ut varius sapien efficitur eget. Nunc enim urna, congue non tristique id, consequat eget nisi. Nulla in massa sit amet nisi rhoncus tempor sagittis id erat.
    Donec in accumsan odio. Integer faucibus velit posuere sem commodo tincidunt. Fusce facilisis ex eget nisl feugiat pulvinar. Curabitur dolor diam, tempus in ligula ut, feugiat cursus elit. Sed et neque molestie, vehicula diam sed, ultricies justo. Maecenas rutrum pharetra sapien eu interdum. Donec vulputate, massa quis malesuada eleifend, ex arcu viverra magna, sit amet volutpat sem risus sit amet dui. Sed ultricies at eros at consequat. Morbi fringilla tristique mauris vel iaculis. Pellentesque ornare dictum finibus. Fusce aliquet odio a blandit interdum.
    Donec mollis finibus metus in cursus. In blandit ornare purus. Vestibulum a ipsum diam. Curabitur vel iaculis diam, id egestas ligula. Morbi condimentum imperdiet tellus. Aenean ut ligula nulla. Nam quis auctor odio. Sed eget blandit magna, sit amet consectetur ex. Quisque fermentum nunc at nisi dictum molestie. Ut tempus fermentum ipsum, vel aliquet sem rutrum quis. Nunc nec tristique diam.
    Touch test.".as_bytes();
    let bucket = blob_store::Container::open("slight-test-bucket")?;

    // verify container name
    assert_eq!(bucket.name()?, "slight-test-bucket");

    for name in bucket.list_objects()? {
        println!("Found object: {}", name);
        bucket.delete_object(&name)?;
    }

    // upload 3 files
    for i in 0..3 {
        // let body = std::fs::read(format!("testfile{i}.txt")).expect("should have been able to read the file");
        bucket
            .write_object(&format!("testfile{}.txt", i))?
            .write(&body)?;
    }

    // read 3 files
    for name in bucket.list_objects()? {
        println!("Found object: {}", name);
        let read_stream = bucket.read_object(&name)?;
        let contents = read_stream.read(1024 * 4)?.unwrap();

        // compare the contents with content on filesystem
        // let body = std::fs::read(format!("{}", name)).expect("should have been able to read the file");
        assert_eq!(body, contents);
    }

    // read one file
    if bucket.has_object("testfile1.txt")? {
        let read_stream = bucket.read_object("testfile1.txt")?;
        let contents = read_stream.read(1024 * 4)?.unwrap();

        // compare the content with content on filesystem
        // let body = std::fs::read("testfile1.txt").expect("should have been able to read the file");
        assert_eq!(body, contents);
    } else {
        bail!("testfile1.txt not found")
    }

    // return metadata for the testfile1.txt
    let metadata = bucket.object_info("testfile1.txt")?;
    assert_eq!(metadata.name, "testfile1.txt");
    assert_eq!(metadata.size, body.len() as u64);
    assert_eq!(metadata.container, "slight-test-bucket");
    println!("metadata created-at: {:?}", metadata.created_at);

    // delete all three files
    bucket.delete_objects(&["testfile0.txt", "testfile1.txt", "testfile2.txt"])?;
    Ok(())
}
