use anyhow::Result;
use crossbeam_channel::Sender;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

// guest resource
pub use event_handler::{EventHandler, EventHandlerData, EventParam};

wit_bindgen_wasmtime::import!({paths: ["../../wit/event-handler.wit"], async: *});
pub use cloudevents::AttributesReader;
pub use cloudevents::AttributesWriter;
pub use cloudevents::Event;
pub use cloudevents::EventBuilder;
pub use cloudevents::EventBuilderV10;

/// A trait for inner representation of the resource
pub trait Watch {
    fn watch(&mut self, key: &str, sender: Arc<Mutex<Sender<Event>>>) -> Result<()> {
        todo!(
            "received {}, and {:#?}, but there's nothing to do with it yet",
            key,
            sender
        );
    }
}

/// Watch state is a dynamic type for resources that implement the `watch` function.
pub type WatchState = Box<dyn Watch + Send + Sync>;

/// A alias to sharable state table
pub type ResourceMap = Arc<Mutex<StateTable>>;

/// A state table that is indexed by each resource unique identifier.
/// The state table stores each resource inner of type WatchState.
#[derive(Default)]
pub struct StateTable(HashMap<String, WatchState>);

impl StateTable {
    /// A wrapper function for inserting a key, value pair in the map
    pub fn set(&mut self, key: String, value: WatchState) {
        self.0.insert(key, value);
    }

    /// A wrapper function for getting a mutable value from the map
    pub fn get_mut(&mut self, key: &str) -> Result<&mut WatchState> {
        let value = self
            .0
            .get_mut(key)
            .ok_or_else(|| anyhow::anyhow!("failed because key '{}' was not found", &key))?;
        Ok(value)
    }

    /// A wrapper function for getting a value from the map
    pub fn get(&mut self, key: &str) -> Result<&WatchState> {
        let value = self
            .0
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("failed because key '{}' was not found", &key))?;
        Ok(value)
    }
}
