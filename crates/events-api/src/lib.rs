// guest resource
pub use event_handler::{EventHandler, EventHandlerData, EventParam};

wit_bindgen_wasmtime::import!("../../wit/event-handler.wit");
pub use cloudevents::AttributesReader;
pub use cloudevents::AttributesWriter;
pub use cloudevents::Event;
pub use cloudevents::EventBuilder;
pub use cloudevents::EventBuilderV10;
