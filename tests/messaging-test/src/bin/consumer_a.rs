use anyhow::Result;

use messaging::*;

wit_bindgen_rust::import!("../../wit/messaging.wit");
wit_error_rs::impl_error!(messaging::MessagingError);

fn main() -> Result<()> {
    let sub = Sub::open("my-messaging")?;
    let sub_token = sub.subscribe("room")?;
    let ps = Pub::open("my-messaging")?;
    loop {
        let msg = sub.receive(&sub_token)?;
        println!("Received message: {msg:?}");
        ps.publish(&msg, "service-a-channel-out")?;
    }
}