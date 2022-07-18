use anyhow::Result;
use std::{thread, time::Duration};

use pubsub::*;
wit_bindgen_rust::import!("../../wit/pubsub.wit");
wit_error_rs::impl_error!(Error);

fn main() -> Result<()> {
    let pubsub = Pubsub::open()?;
    for i in 0..3 {
        println!("sending message");
        pubsub.send_message_to_topic(
            format!("key-{}", i).as_bytes(),
            format!("value-{}", i).as_bytes(),
            "rust",
        )?;
        thread::sleep(Duration::from_secs(3));
    }
    Ok(())
}
