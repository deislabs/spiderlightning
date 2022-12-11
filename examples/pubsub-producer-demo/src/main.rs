use anyhow::Result;
use std::{thread, time::Duration};

use pubsub::*;
wit_bindgen_rust::import!("../../wit/pubsub.wit");
wit_error_rs::impl_error!(pubsub::Error);

fn main() -> Result<()> {
    let ps = Pub::open("my-pubsub")?;
    for i in 0..3 {
        println!("sending messages");
        ps.publish(format!("rust-value-{}", i).as_bytes(), "rust")?;
        ps.publish(format!("gc-value-{}", i).as_bytes(), "global-chat")?;
        thread::sleep(Duration::from_secs(3));
    }
    Ok(())
}
