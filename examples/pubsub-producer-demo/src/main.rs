use std::{thread, time::Duration};
use anyhow::Result;

use pubsub::*;
wit_bindgen_rust::import!("../../wit/pubsub.wit");

fn main() -> Result<()> {
    let resource_descriptor = get_pubsub()?;
    for i in 0..3 {
        println!("sending message");
        send_message_to_topic(&resource_descriptor, format!("key-{}", i).as_bytes(), format!("value-{}", i).as_bytes(), "rust")?;
        thread::sleep(Duration::from_secs(3));
    }
    Ok(())
}

impl From<pubsub::Error> for anyhow::Error {
    fn from(_: pubsub::Error) -> Self {
        anyhow::anyhow!("pubsub error")
    }
}