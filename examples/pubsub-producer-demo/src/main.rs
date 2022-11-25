use anyhow::Result;
use std::{thread, time::Duration};

use pubsub::*;
wit_bindgen_rust::import!("../../wit/pubsub.wit");
wit_error_rs::impl_error!(pubsub::Error);

fn main() -> Result<()> {
    let ps = Pubsub::open("my-pubsub")?;
    for i in 0..3 {
        println!("sending message");
        ps.publish(format!("value-{}", i).as_bytes(), "rust")?;
        thread::sleep(Duration::from_secs(3));
    }
    Ok(())
}
