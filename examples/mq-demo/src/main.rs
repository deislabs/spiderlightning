use anyhow::Result;

use mq::*;
wit_bindgen_rust::import!("../../wit/mq.wit");

fn main() -> Result<()> {
    let resource_descriptor = get_mq()?;
    println!("sending \"hello, world!\" to queue");
    send(&resource_descriptor, "hello, world!".as_bytes())?;
    println!(
        "top message in the queue: {:#?}",
        std::str::from_utf8(&receive(&resource_descriptor)?)?
    );
    println!(
        "top message in the queue after receive: {:#?}",
        std::str::from_utf8(&receive(&resource_descriptor)?)?
    );

    Ok(())
}

impl From<mq::Error> for anyhow::Error {
    fn from(_: mq::Error) -> Self {
        anyhow::anyhow!("mq error")
    }
}
