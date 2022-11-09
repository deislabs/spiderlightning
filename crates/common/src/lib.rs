mod context;
mod state;
#[cfg(feature = "wasmtime")]
mod wasmtime_runtime;

use anyhow::Result;
use as_any::AsAny;

pub use context::Ctx;
pub use state::BasicState;

pub use wasmtime_runtime::{Builder, Linker, WasmtimeBuildable, WasmtimeLinkable};

/// A trait for wit-bindgen resources
pub trait Resource: AsAny {}

/// A trait for wit-bindgen resource tables. see [here](https://github.com/bytecodealliance/wit-bindgen/blob/main/crates/wasmtime/src/table.rs) for more details:
pub trait ResourceTables<T: ?Sized>: AsAny {}

pub trait ResourceBuilder {
    fn build(self) -> Result<HostState>;
}

/// HostState abstract out generated bindings for the resource,
/// and the resource table. It is used as a type in the `RuntimeContext`
/// primarily for linking host defined resources for capabilities.
pub type HostState = (
    Box<dyn Resource + Send + Sync>,
    Option<Box<dyn ResourceTables<dyn Resource> + Send + Sync>>,
);

#[macro_export]
#[allow(unknown_lints)]
#[allow(clippy::crate_in_macro_def)]
macro_rules! impl_resource {
    ($resource:ident, $resource_table:ty, $state:ident, $add_to_linker:path, $scheme_name:expr) => {
        impl slight_common::Resource for $resource {}
        impl slight_common::ResourceTables<dyn slight_common::Resource> for $resource_table {}
        impl slight_common::ResourceBuilder for $resource {

            fn build(self) -> anyhow::Result<slight_common::HostState> {
                /// We prepare a default resource with host-provided state.
                /// Then the guest will pass other configuration state to the resource.
                /// This is done in the `<Capability>::open` function.
                Ok((
                    Box::new(self),
                    Some(Box::new(<$resource_table>::default())),
                ))
            }
        }

        impl slight_common::WasmtimeLinkable for $resource {
            fn add_to_linker<Ctx: slight_common::Ctx + Send + Sync + 'static>(linker: &mut slight_common::Linker<Ctx>) -> anyhow::Result<()> {
                $add_to_linker(linker, |ctx| {
                    Ctx::get_host_state::<$resource, $resource_table>(ctx, $scheme_name)
                })
            }
        }
    };

    ($resource:ty, $resource_table:ty, $state:ty, $lt:tt, $add_to_linker:path, $scheme_name:expr) => {
        impl<$lt> slight_common::Resource for $resource
        where
            $lt: slight_common::WasmtimeBuildable + 'static
        {}
        impl<$lt> slight_common::ResourceTables<dyn slight_common::Resource> for $resource_table
        where
            $lt: slight_common::WasmtimeBuildable + Send + Sync + 'static
        {}
        impl<$lt> slight_common::ResourceBuilder for $resource
        where
            $lt: slight_common::WasmtimeBuildable + Send + Sync + 'static
        {
            fn build(self) -> anyhow::Result<slight_common::HostState> {
                /// We prepare a default resource with host-provided state.
                /// Then the guest will pass other configuration state to the resource.
                /// This is done in the `<Capability>::open` function.
                Ok((
                    Box::new(self),
                    Some(Box::new(<$resource_table>::default())),
                ))
            }
        }

        impl<$lt> slight_common::WasmtimeLinkable for $resource
        where
            $lt: slight_common::WasmtimeBuildable + Send + Sync + 'static
        {
            fn add_to_linker<Ctx: slight_common::Ctx + Send + Sync + 'static>(linker: &mut slight_common::Linker<Ctx>) -> anyhow::Result<()> {
                $add_to_linker(linker, |ctx| {
                    Ctx::get_host_state::<$resource, $resource_table>(ctx, $scheme_name)
                })
            }
        }
    };
}
