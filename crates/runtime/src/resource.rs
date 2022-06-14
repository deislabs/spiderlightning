use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{Context, Result};
use as_any::{AsAny, Downcast};
pub use wasmtime::Linker;

pub use crate::RuntimeContext;

pub type DataT = (
    Box<dyn Resource>,
    Option<Box<dyn ResourceTables<dyn Resource>>>,
);
pub type ResourceConfig = String;
pub type ResourceMap = Arc<Mutex<Map>>;
pub type Ctx = RuntimeContext<DataT, GuestState>;

#[derive(Default)]
pub struct Map(HashMap<String, DataT>);

impl Map {
    pub fn lock(wrapped_map: &mut Option<Arc<Mutex<Map>>>) -> Result<MutexGuard<Map>> {
        wrapped_map
            .as_mut()
            .with_context(|| "failed because resource map is not initialized")?
            .lock()
            .map_err(|_| anyhow::anyhow!("failed to acquire lock on resource map"))
    }

    pub fn set(&mut self, key: String, value: DataT) {
        self.0.insert(key, value);
    }

    pub fn get<T: 'static>(&self, key: &str) -> Result<&T> {
        let value = self.0.get(key).with_context(|| {
            "failed to match resource descriptor in map of instantiated resources"
        })?;
        let inner = value.0.get_inner();
        <&dyn std::any::Any>::clone(&inner)
            .downcast_ref::<T>()
            .with_context(|| "failed to acquire matched resource descriptor service")
    }

    pub fn get_dynamic(&self, key: &str) -> Result<&DataT> {
        let value = self
            .0
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("key not found"))?;
        Ok(value)
    }
}

/// A trait for wit-bindgen resource tables. see [here](https://github.com/bytecodealliance/wit-bindgen/blob/main/crates/wasmtime/src/table.rs) for more details:
pub trait ResourceTables<T: ?Sized>: AsAny {}

pub trait Resource: AsAny {
    /// get inner representation of the resource.
    fn get_inner(&self) -> &dyn Any;

    /// Add resource map to resource.
    fn add_resource_map(&mut self, resource_map: ResourceMap) -> Result<()>;

    /// check if the resource has changed on key.
    fn changed(&self, key: &str) -> bool;
}

/// A trait for wit-bindgen host resource composed of a resource and a resource table.
pub trait RuntimeResource {
    fn add_to_linker(linker: &mut Linker<Ctx>) -> Result<()>;
    fn build_data() -> Result<DataT>;
}

/// dynamic dispatch to respective host resource.
pub fn get<T>(cx: &mut Ctx, resource_key: String) -> &mut T
where
    T: 'static,
{
    let data = cx
        .data
        .get_mut(&resource_key)
        .expect("internal error: Runtime context data is None");

    data.0.as_mut().downcast_mut().unwrap()
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
        data.0.as_mut().downcast_mut().unwrap(),
        data.1.as_mut().unwrap().as_mut().downcast_mut().unwrap(),
    )
}

// guest resource
use event_handler::EventHandlerData;

wit_bindgen_wasmtime::import!("../../wit/event-handler.wit");

pub type GuestState = EventHandlerData;
