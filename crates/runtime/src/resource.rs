use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use as_any::{AsAny, Downcast};
pub use wasmtime::Linker;

pub use crate::Context;

pub type DataT = Box<dyn Resource>;
pub type ResourceMap = Arc<Mutex<HashMap<String, Box<dyn Resource>>>>;
pub type ResourceConfig = String;

/// A trait for wit-bindgen resource tables. see [here](https://github.com/bytecodealliance/wit-bindgen/blob/main/crates/wasmtime/src/table.rs) for more details:
pub trait ResourceTables<T: ?Sized>: AsAny {}

/// A trait for wit-bindgen resource.
pub trait Resource: AsAny {
    /// get inner representation of the resource.
    fn get_inner(&self) -> &dyn Any;

    /// Add resource map to resource.
    fn add_resource_map(&mut self, resource_map: ResourceMap) -> Result<()>;
}

/// A trait for wit-bindgen host resource composed of a resource and a resource table.
pub trait HostResource {
    fn add_to_linker(linker: &mut Linker<Context<DataT>>) -> Result<()>;
    fn build_data() -> Result<DataT>;
}

/// dynamic dispatch to respective host resource.
pub fn get<T>(cx: &mut Context<DataT>, resource_key: String) -> &mut T
where
    T: 'static,
{
    let data = cx
        .data
        .get_mut(&resource_key)
        .expect("internal error: Runtime context data is None");

    data.as_mut().downcast_mut().unwrap()
}
