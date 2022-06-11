use std::time::SystemTime;

use anyhow::Result;
use pubsub::*;
wit_bindgen_rust::import!("../../wit/pubsub.wit");

fn main() -> Result<()> {
    let resource_descriptor = get_pubsub()?;
    let now = SystemTime::now();
    subscribe_to_topic(resource_descriptor, &["rust"])?;
    let timeout_as_secs = 30;
    while now.elapsed().unwrap().as_secs() < timeout_as_secs {
        let message = poll_for_message(resource_descriptor, timeout_as_secs)?;
        println!(
            "received message> key: {:?}",
            message.key.as_ref().map(|f| std::str::from_utf8(f))
        );
        println!(
            "received message> value: {:?}",
            message.value.as_ref().map(|f| std::str::from_utf8(f))
        );
    }
    Ok(())
}

impl From<pubsub::Error> for anyhow::Error {
    fn from(_: pubsub::Error) -> Self {
        anyhow::anyhow!("pubsub error")
    }
}
