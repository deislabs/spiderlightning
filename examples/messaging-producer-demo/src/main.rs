use anyhow::Result;
use std::{thread, time::Duration};

use messaging::*;
wit_bindgen_rust::import!("../../wit/messaging.wit");
wit_error_rs::impl_error!(messaging::MessagingError);

fn main() -> Result<()> {
    let ps = Pub::open("my-messaging")?;
    for i in 0..3 {
        println!("sending messages");
        ps.publish(format!("rust-value-{}", i).as_bytes(), "rust")?;
        ps.publish(format!("gc-value-{}", i).as_bytes(), "global-chat")?;
        thread::sleep(Duration::from_secs(3));
    }
    Ok(())
}
