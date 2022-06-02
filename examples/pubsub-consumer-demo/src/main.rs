use std::time::SystemTime;

use anyhow::Result;
use pubsub::*;
wit_bindgen_rust::import!("../../wit/pubsub.wit");

fn main() -> Result<()> {
    let resource_descriptor = get_pubsub()?;
    let now = SystemTime::now();
    while now.elapsed().unwrap().as_secs() < 60 {
        subscribe_to_topic(&resource_descriptor, &["rust"])?;
        print_message_stream(&resource_descriptor)?;        
    }
    Ok(())
}

impl From<pubsub::Error> for anyhow::Error {
    fn from(_: pubsub::Error) -> Self {
        anyhow::anyhow!("pubsub error")
    }
}