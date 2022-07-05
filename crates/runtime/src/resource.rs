use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{Context, Result};
use as_any::{AsAny, Downcast};
pub use wasmtime::Linker;

pub use crate::RuntimeContext;

pub type DataT = Box<dyn Resource>;
pub type ResourceConfig = String;
pub type ResourceMap = Arc<Mutex<Map>>;
pub type Ctx = RuntimeContext<DataT>;

/// A map wrapper type for the resource map
#[derive(Default)]
pub struct Map(HashMap<String, Box<dyn Resource>>);

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
        let inner = value.get_inner();
        <&dyn std::any::Any>::clone(&inner)
            .downcast_ref::<T>()
            .with_context(|| "failed to acquire matched resource descriptor service")
    }
}

/// An implemented service interface
pub trait Resource: AsAny {
    /// Get inner representation of the resource
    fn get_inner(&self) -> &dyn Any;

    /// Add resource map to resource
    fn add_resource_map(&mut self, resource_map: ResourceMap) -> Result<()>;
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

    data.as_mut().downcast_mut().unwrap()
}
