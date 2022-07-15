use anyhow::Result;

use event_handler::Event;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_error!(events::Error);
wit_bindgen_rust::import!("../../wit/events.wit");
wit_bindgen_rust::export!("../../wit/event-handler.wit");

fn main() -> Result<()> {
    // application devleoper does not need to know the host implementation details.
    let kv1 = Kv::open("my-container")?;
    let kv2 = Kv::open("my-container2")?;
    let value = "spiderlightning".as_bytes();
    kv1.set("key", value)?;
    kv2.set("key", value)?;
    println!(
        "Hello, world! the value for kv1 is: {}, kv2 is {}",
        std::str::from_utf8(&kv1.get("key")?)?,
        std::str::from_utf8(&kv2.get("key")?)?,
    );
    kv1.delete("key")?;
    kv2.delete("key")?;
    let value = kv1.get("key");
    assert!(value.is_err());

    let ob1 = kv1.watch("my-key")?;
    let ob2 = kv1.watch("my-key2")?;
    let events = events::Events::get()?;
    // TODO (mosssaka): I had to construct a copy of Observable because wit_bindgen generates two
    // observables in different mods: events::Observable vs. kv::Observable.
    events
        .listen(events::Observable {
            rd: ob1.rd.as_str(),
            key: ob1.key.as_str(),
        })?
        .listen(events::Observable {
            rd: ob2.rd.as_str(),
            key: ob2.key.as_str(),
        })?
        .exec(5)?;

    drop(kv1); // drop != close
    drop(kv2);
    Ok(())
}

pub struct EventHandler {}

impl event_handler::EventHandler for EventHandler {
    fn handle_event(ev: Event) -> Result<Option<Event>, String> {
        let kv = Kv::open("my-container").unwrap();
        let key = ev.data.unwrap();
        let value = kv.get(&key).unwrap();
        println!(
            "received event of type {}, key: {}, new value: {}",
            &ev.event_type,
            &key,
            std::str::from_utf8(&value).unwrap()
        );
        Ok(None)
    }
}
