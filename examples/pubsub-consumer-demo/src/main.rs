use std::time::SystemTime;

use anyhow::Result;
use pubsub::*;
wit_bindgen_rust::import!("../../wit/pubsub.wit");
wit_error_rs::impl_error!(pubsub::Error);

fn main() -> Result<()> {
    let ps = Sub::open()?;
    let now = SystemTime::now();
    ps.subscribe_to_topic(&["rust"])?;
    let timeout_as_secs = 30;
    while now.elapsed().unwrap().as_secs() < timeout_as_secs {
        let message = ps.poll_for_message(timeout_as_secs)?;
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
