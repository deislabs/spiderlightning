use anyhow::Result;

use mq::*;
wit_bindgen_rust::import!("../../wit/mq.wit");
wit_error_rs::impl_error!(Error);

fn main() -> Result<()> {
    let mq = Mq::open("wasi-cloud-queue")?;
    for _ in 0..3 {
        println!(
            "top message in the queue: {:#?}",
            std::str::from_utf8(&mq.receive()?)?
        );
    }
    Ok(())
}
