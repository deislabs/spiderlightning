use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

pub use crate::RuntimeContext;
use anyhow::{Context, Result};
use as_any::{AsAny, Downcast};
use crossbeam_channel::Sender;
pub use wasmtime::Linker;

pub type DataT = (
    Box<dyn Resource + Send + Sync>,
    Option<Box<dyn ResourceTables<dyn Resource> + Send + Sync>>,
);
pub type ResourceConfig = String;
pub type ResourceMap = Arc<Mutex<Map>>;
pub type Ctx = RuntimeContext<DataT>;

/// A map wrapper type for the resource map
#[derive(Default)]
pub struct Map(HashMap<String, DataT>);

impl Map {
    /// A convinience function for grabbing a lock provided a map
    pub fn lock(wrapped_map: &mut Option<Arc<Mutex<Map>>>) -> Result<MutexGuard<Map>> {
        wrapped_map
            .as_mut()
            .with_context(|| "failed because resource map is not initialized")?
            .lock()
            .map_err(|_| anyhow::anyhow!("failed to acquire lock on resource map"))
    }

    /// A wrapper function for inserting a key in the map
    pub fn set(&mut self, key: String, value: DataT) {
        self.0.insert(key, value);
    }

    /// A wrapper funciton around getting a value from a map
    pub fn get<T: 'static>(&self, key: &str) -> Result<&T> {
        let value = self.0.get(key).with_context(|| {
            "failed to match resource descriptor in map of instantiated resources"
        })?;
        let inner = value.0.get_inner();
        <&dyn std::any::Any>::clone(&inner)
            .downcast_ref::<T>()
            .with_context(|| "failed to acquire matched resource descriptor service")
    }

    pub fn get_dynamic_mut(&mut self, key: &str) -> Result<&mut DataT> {
        let value = self
            .0
            .get_mut(key)
            .ok_or_else(|| anyhow::anyhow!("failed because key '{}' was not found", &key))?;
        Ok(value)
    }
}

/// A trait for wit-bindgen resource tables. see [here](https://github.com/bytecodealliance/wit-bindgen/blob/main/crates/wasmtime/src/table.rs) for more details:
pub trait ResourceTables<T: ?Sized>: AsAny {}

/// An implemented service interface
pub trait Resource: AsAny {
    /// Get inner representation of the resource
    fn get_inner(&self) -> &dyn Any;

    /// Add resource map to resource
    fn add_resource_map(&mut self, resource_map: ResourceMap) -> Result<()>;

    /// check if the resource has changed on key.
    fn watch(
        &mut self,
        data: &str,
        rd: &str,
        key: &str,
        sender: Arc<Mutex<Sender<Event>>>,
    ) -> Result<()>;
}

/// A trait for wit-bindgen host resource composed of a resource
pub trait RuntimeResource {
    fn add_to_linker(linker: &mut Linker<Ctx>) -> Result<()>;
    fn build_data() -> Result<DataT>;
}

/// Dynamically dispatch to respective host resource
pub fn get<T>(cx: &mut Ctx, resource_key: String) -> &mut T
where
    T: 'static,
{
    let data = cx
        .data
        .get_mut(&resource_key)
        .expect("internal error: Runtime context data is None");

    data.0.as_mut().downcast_mut().unwrap_or_else(|| {
        panic!(
            "internal error: context has key {} but can't be downcast to resource {}",
            &resource_key,
            std::any::type_name::<T>()
        )
    })
}

pub fn get_table<T, TTable>(cx: &mut Ctx, resource_key: String) -> (&mut T, &mut TTable)
where
    T: 'static,
    TTable: 'static,
{
    let data = cx
        .data
        .get_mut(&resource_key)
        .expect("internal error: Runtime context data is None");
    (
        data.0.as_mut().downcast_mut().unwrap_or_else(|| {
            panic!(
                "internal error: context has key {} but can't be downcast to resource {}",
                &resource_key,
                std::any::type_name::<T>()
            )
        }),
        data.1
            .as_mut()
            .unwrap_or_else(|| {
                panic!(
                    "internal error: table {} is not initialized",
                    std::any::type_name::<TTable>()
                )
            })
            .as_mut()
            .downcast_mut()
            .unwrap_or_else(|| {
                panic!(
                    "internal error: context has key {} but can't be downcast to resource_table {}",
                    &resource_key,
                    std::any::type_name::<TTable>()
                )
            }),
    )
}

// guest resource
use event_handler::EventHandlerData;

wit_bindgen_wasmtime::import!("../../wit/event-handler.wit");

pub type GuestState = EventHandlerData;

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
