use anyhow::Result;

use event_handler::Event;
use events::Events;
use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_error!(events::EventError);
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
    let events = Events::get()?;
    events
        .listen(events::Observable {
            rd: ob1.rd.as_str(),
            key: ob1.key.as_str(),
        })?
        .exec(100)?;

    drop(rd1); // drop != close
    drop(rd2);
    Ok(())
}

pub struct EventHandler {}

impl event_handler::EventHandler for EventHandler {
    fn handle_event(ev: Event) -> Result<Option<Event>, String> {
        println!("{}", ev.data.unwrap());
        Ok(None)
    }
}
