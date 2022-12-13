use anyhow::Result;

use keyvalue::*;
wit_bindgen_rust::import!("../../wit/keyvalue.wit");
wit_error_rs::impl_error!(keyvalue::KeyvalueError);

use messaging::*;
wit_bindgen_rust::import!("../../wit/messaging.wit");
wit_error_rs::impl_error!(messaging::MessagingError);

fn main() -> Result<()> {
    let keyvalue = Keyvalue::open("my-container")?;
    let p = Pub::open("wasi-cloud-queue")?;
    let s = Sub::open("wasi-cloud-queue")?;

    for _ in 0..3 {
        println!("sending \"hello, world!\" to the queue");
        p.publish("hello, world!".as_bytes(), "")?;
    }

    let mut messages_vec: Vec<String> = vec![];
    for _ in 0..3 {
        let top_message = s.receive("")?;
        messages_vec.push(String::from_utf8(top_message)?);
        println!("top message in the queue: {:#?}", messages_vec.last());
    }

    let all_messages = messages_vec.join("\n");
    keyvalue.set("messages", all_messages.as_bytes())?;
    println!("Adding all messages ever sent to the queue to the keyvalue...");

    println!(
        "Retrieving all messages ever sent to the queue:\n{}",
        std::str::from_utf8(&keyvalue.get("messages")?)?
    );

    keyvalue.delete("messages")?;
    println!("Deleting all messages ever sent to a queue from the keyvalue...");

    Ok(())
}
