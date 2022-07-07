// guest resource
pub use event_handler::EventHandlerData;

wit_bindgen_wasmtime::import!("../../wit/event-handler.wit");


#[derive(Debug, Default, Clone)]
pub struct Event {
    pub source: String,
    pub event_type: String,
    pub specversion: String,
    pub id: String,
    pub data: Option<String>,
}

impl Event {
    pub fn new(
        source: String,
        event_type: String,
        specversion: String,
        id: String,
        data: Option<String>,
    ) -> Self {
        Self {
            source,
            event_type,
            specversion,
            id,
            data,
        }
    }
}
