use anyhow::Result;
use as_any::{AsAny, Downcast};
use url::Url;
pub use wasmtime::Linker;

pub use crate::Context;

pub type DataT = (Box<dyn Resource>, Box<dyn ResourceTables<dyn Resource>>);

/// A trait for wit-bindgen resource tables. see [here](https://github.com/bytecodealliance/wit-bindgen/blob/main/crates/wasmtime/src/table.rs) for more details:
pub trait ResourceTables<T: ?Sized>: AsAny {}

/// A trait for wit-bindgen resource.
pub trait Resource: AsAny {
    /// Given a resource url, return a resource.
    fn from_url(url: Url) -> Result<Self>
    where
        Self: Sized;
}

pub trait HostResource {
    fn add_to_linker(linker: &mut Linker<Context<DataT>>) -> Result<()>;
    fn build_state(url: Url) -> Result<DataT>;
}

pub fn get<T, TTables>(cx: &mut Context<DataT>) -> (&mut T, &mut TTables)
where
    T: 'static,
    TTables: 'static,
{
    let data = cx
        .data
        .as_mut()
        .expect("internal error: Runtime context data is None");
    let resource = data.0.as_mut().downcast_mut::<T>().unwrap();
    let resource_tables = data.1.as_mut().downcast_mut::<TTables>().unwrap();
    (resource, resource_tables)
}
