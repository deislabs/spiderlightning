pub use wasmtime::Linker;

use crate::Ctx;
use anyhow::Result;

/// A trait for WasmtimeLinkable resources
pub trait WasmtimeLinkable {
    /// Link the resource to the runtime
    fn add_to_linker<T: Ctx + Send + Sync + 'static>(linker: &mut Linker<T>) -> Result<()>;
}
