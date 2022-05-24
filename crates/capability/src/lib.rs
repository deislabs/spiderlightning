use anyhow::Result;
use as_any::AsAny;
use url::Url;

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
