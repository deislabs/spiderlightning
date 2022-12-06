use anyhow::Result;
use pubsub::*;
wit_bindgen_rust::import!("../../wit/pubsub.wit");
wit_error_rs::impl_error!(pubsub::Error);

fn main() -> Result<()> {
    let ps = Sub::open("my-pubsub")?;
    let sub_tok = ps.subscribe("rust")?;
    for _ in 0..3 {
        loop {
            let msg = ps.receive(sub_tok)?;
            if !msg.is_empty() {
                println!("received message> value: {:?}", String::from_utf8(msg));
                break;
            }
        }
    }
    Ok(())
}
