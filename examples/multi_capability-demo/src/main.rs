use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");
wit_error_rs::impl_error!(kv::Error);

use mq::*;
wit_bindgen_rust::import!("../../wit/mq.wit");
wit_error_rs::impl_error!(mq::Error);

fn main() -> Result<()> {
    // application developer does not need to know the host implementation details.

    let kv = Kv::open("my-container")?;
    let mq = Mq::open("wasi-cloud-queue")?;

    for _ in 0..3 {
        println!("sending \"hello, world!\" to the queue");
        mq.send("hello, world!".as_bytes())?;
    }

    let mut messages_vec: Vec<String> = vec![];
    for _ in 0..3 {
        let top_message = mq.receive()?;
        messages_vec.push(String::from_utf8(top_message)?);
        println!("top message in the queue: {:#?}", messages_vec.last());
    }

    let all_messages = messages_vec.join("\n");
    kv.set("messages", all_messages.as_bytes())?;
    println!("Adding all messages ever sent to the queue to the kv store...");

    println!(
        "Retrieving all messages ever sent to the queue:\n{}",
        std::str::from_utf8(&kv.get("messages")?)?
    );

    kv.delete("messages")?;
    println!("Deleting all messages ever sent to a queue from the kv store...");

    Ok(())
}
