use anyhow::Result;
use as_any::{AsAny, Downcast};
use url::Url;
pub use wasmtime::Linker;

pub use crate::Context;

pub type DataT = Box<dyn Resource>;

/// A trait for wit-bindgen resource tables. see [here](https://github.com/bytecodealliance/wit-bindgen/blob/main/crates/wasmtime/src/table.rs) for more details:
pub trait ResourceTables<T: ?Sized>: AsAny {}

/// A trait for wit-bindgen resource.
pub trait Resource: AsAny {
    /// Given a resource url, return a resource.
    fn from_url(url: Url) -> Result<Self>
    where
        Self: Sized;
}

/// A trait for wit-bindgen host resource composed of a resource and a resource table.
pub trait HostResource {
    fn add_to_linker(linker: &mut Linker<Context<DataT>>) -> Result<()>;
    fn build_data(url: Url) -> Result<DataT>;
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
