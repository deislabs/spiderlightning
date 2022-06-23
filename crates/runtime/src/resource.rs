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

#[derive(Default)]
pub struct Map(HashMap<String, Box<dyn Resource>>);

impl Map {
    pub fn unwrap(wrapped_map: &mut Option<Arc<Mutex<Map>>>) -> Result<MutexGuard<Map>> {
        let res = wrapped_map
            .as_mut()
            .with_context(|| "resource map is not initialized")?
            .lock()
            .unwrap(); // panic if we cannot acquire a lock

        Ok(res)
    }

    pub fn set(&mut self, key: String, value: DataT) {
        self.0.insert(key, value);
    }

    pub fn get<T: 'static>(&self, key: &str) -> Result<&T> {
        let value = self.0.get(key).with_context(|| {
            "failed to match resource descriptor with map of instantiated resources"
        })?;
        let inner = value.get_inner();
        <&dyn std::any::Any>::clone(&inner)
            .downcast_ref::<T>()
            .with_context(|| "failed acquire matched resource descriptor service")
    }
}

/// A trait for wit-bindgen resource tables. see [here](https://github.com/bytecodealliance/wit-bindgen/blob/main/crates/wasmtime/src/table.rs) for more details:
pub trait ResourceTables<T: ?Sized>: AsAny {}

pub trait Resource: AsAny {
    /// get inner representation of the resource.
    fn get_inner(&self) -> &dyn Any;

    /// Add resource map to resource.
    fn add_resource_map(&mut self, resource_map: ResourceMap) -> Result<()>;
}

/// A trait for wit-bindgen host resource composed of a resource and a resource table.
pub trait RuntimeResource {
    fn add_to_linker(linker: &mut Linker<RuntimeContext<DataT>>) -> Result<()>;
    fn build_data() -> Result<DataT>;
}

/// dynamic dispatch to respective host resource.
pub fn get<T>(cx: &mut RuntimeContext<DataT>, resource_key: String) -> &mut T
where
    T: 'static,
{
    let data = cx
        .data
        .get_mut(&resource_key)
        .expect("internal error: Runtime context data is None");

    data.as_mut().downcast_mut().unwrap()
}
