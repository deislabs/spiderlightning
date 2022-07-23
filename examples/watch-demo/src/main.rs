use anyhow::Result;

use event_handler::Event;

use state_store::*;
wit_bindgen_rust::import!("../../wit/state_store.wit");
wit_error_rs::impl_error!(state_store::Error);
wit_error_rs::impl_error!(events::Error);
wit_bindgen_rust::import!("../../wit/events.wit");
wit_bindgen_rust::export!("../../wit/event-handler.wit");

fn main() -> Result<()> {
    // application devleoper does not need to know the host implementation details.

    let ss1 = StateStore::open("my-container")?;
    let ss2 = StateStore::open("my-container2")?;
    let value = "spiderlightning".as_bytes();
    ss1.set("key", value)?;
    ss2.set("key", value)?;
    println!(
        "The value for ss1 is: {}, ss2 is {}",
        std::str::from_utf8(&ss1.get("key")?)?,
        std::str::from_utf8(&ss2.get("key")?)?,
    );
    ss1.delete("key")?;
    ss2.delete("key")?;
    let value = ss1.get("key");
    assert!(value.is_err());

    let ob1 = ss1.watch("my-key")?;
    let ob2 = ss1.watch("my-key2")?;
    let events = events::Events::get()?;
    // TODO (mosssaka): I had to construct a copy of Observable because wit_bindgen generates two
    // observables in different mods: events::Observable vs. state_store::Observable.
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

    drop(ss1); // drop != close
    drop(ss2);
    Ok(())
}

pub struct EventHandler {}

impl event_handler::EventHandler for EventHandler {
    fn handle_event(ev: Event) -> Result<Option<Event>, String> {
        // event.data has value: "String data: key: my-key2"
        let ss = StateStore::open("my-container").unwrap();
        let data = ev.data.unwrap();
        let value =
            serde_json::from_str::<serde_json::Value>(std::str::from_utf8(&data).unwrap()).unwrap();
        let key = value["key"].as_str().unwrap();
        let value = ss.get(key).unwrap();
        println!(
            "received event of type {}, key: {}, new value: {}",
            &ev.ty,
            key,
            std::str::from_utf8(&value).unwrap()
        );
        Ok(None)
    }
}
