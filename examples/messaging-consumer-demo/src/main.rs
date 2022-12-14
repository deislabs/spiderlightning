use anyhow::Result;
use messaging::*;
wit_bindgen_rust::import!("../../wit/messaging.wit");
wit_error_rs::impl_error!(messaging::MessagingError);

fn main() -> Result<()> {
    let ps = Sub::open("my-messaging")?;
    let sub_tok = ps.subscribe("rust")?;
    let sub_tok1 = ps.subscribe("global-chat")?;

    for _ in 0..3 {
        loop {
            let msg = ps.receive(&sub_tok)?;
            if !msg.is_empty() {
                println!("received message from topic 'rust'> value: {:?}", String::from_utf8(msg));
                break;
            }
        }

        loop {
            let msg = ps.receive(&sub_tok1)?;
            if !msg.is_empty() {
                println!("received message from topic 'global-chat'> value: {:?}", String::from_utf8(msg));
                break;
            }
        }
    }
    Ok(())
}
