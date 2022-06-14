use anyhow::Result;

use mq::*;
wit_bindgen_rust::import!("../../wit/mq.wit");

fn main() -> Result<()> {
    let resource_descriptor = get_mq("orders")?;
    for _ in 0..3 {
        println!("sending \"hello, world!\" to queue");
        send(&resource_descriptor, "hello, world!".as_bytes())?;
    }

    Ok(())
}

impl From<mq::Error> for anyhow::Error {
    fn from(_: mq::Error) -> Self {
        anyhow::anyhow!("mq error")
    }
}
