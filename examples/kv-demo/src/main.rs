use anyhow::Result;

use event_handler::Event;
use events::Events;
use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_error!(events::Error);
wit_bindgen_rust::import!("../../wit/events.wit");
wit_bindgen_rust::export!("../../wit/event-handler.wit");

fn main() -> Result<()> {
    // application devleoper does not need to know the host implementation details.

    let rd1 = get_kv("my-container")?;
    let rd2 = get_kv("my-container2")?;
    let value = "wasi-cloud".as_bytes();
    set(&rd1, "key", value)?;
    set(&rd2, "key", value)?;
    println!(
        "Hello, world! the value for rd1 is: {}, rd2 is {}",
        std::str::from_utf8(&get(&rd1, "key")?)?,
        std::str::from_utf8(&get(&rd2, "key")?)?,
    );
    delete(&rd1, "key")?;
    delete(&rd2, "key")?;
    let value = get(&rd1, "key");
    assert_eq!(value.is_err(), true);

    let ob1 = watch(&rd1, "my-key")?;
    let ob2 = watch(&rd1, "my-key2")?;
    let events = Events::get()?;
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

    drop(rd1); // drop != close
    drop(rd2);
    Ok(())
}

pub struct EventHandler {}

impl event_handler::EventHandler for EventHandler {
    fn handle_event(ev: Event) -> Result<Option<Event>, String> {
        let rd = get_kv("my-container").unwrap();
        let key = ev.data.unwrap();
        let value = get(&rd, &key).unwrap();
        println!(
            "received event of type {}, key: {}, new value: {}",
            &ev.event_type,
            &key,
            std::str::from_utf8(&value).unwrap()
        );
        Ok(None)
    }
}
