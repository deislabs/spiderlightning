use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use as_any::AsAny;
use async_trait::async_trait;

use wasmtime::{Instance, Store};

use slight_events_api::{EventHandlerData, ResourceMap};
use slight_http_api::HttpHandlerData;

pub use wasmtime::Linker;
/// `BasicState` provides an attempt at a "fit-all" for basic scenarios
/// of a host's state.
///
/// It contains:
///     - a `resource_map`,
///     - a `implementor`,
///     - a `name`,
///     - a `configs_map`, and
///     - the `slightfile_path`.
#[derive(Clone, Default)]
pub struct BasicState {
    pub resource_map: ResourceMap,
    pub secret_store: Option<String>,
    pub implementor: String,
    pub name: String,
    pub configs_map: Option<HashMap<String, String>>,
    pub slightfile_path: PathBuf,
}

impl BasicState {
    pub fn new(
        resource_map: ResourceMap,
        secret_store: Option<String>,
        implementor: String,
        name: String,
        configs_map: Option<HashMap<String, String>>,
        slightfile_path: impl AsRef<Path>,
    ) -> Self {
        Self {
            resource_map,
            secret_store,
            implementor,
            name,
            configs_map,
            slightfile_path: slightfile_path.as_ref().to_owned(),
        }
    }
}

impl std::fmt::Debug for BasicState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "BasicState {{name: {}, implementor: {}}}",
            self.name, self.implementor
        )
    }
}

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

pub trait Ctx {
    fn get_http_state_mut(&mut self) -> &mut HttpHandlerData;
    fn get_events_state_mut(&mut self) -> &mut EventHandlerData;
    fn get_table<T: 'static, TTable: 'static>(
        &mut self,
        resource_key: String,
    ) -> (&mut T, &mut TTable);
}

/// A trait for builder
#[async_trait]
pub trait Buildable: Clone {
    type Ctx: Ctx + Send + Sync;

    async fn build(&self) -> (Store<Self::Ctx>, Instance);
}

#[derive(Clone)]
pub struct Builder<T: Buildable> {
    inner: T,
}

impl<T: Buildable> Builder<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }
}

/// A trait for Linkable resources
pub trait Linkable {
    /// Link the resource to the runtime
    fn add_to_linker<T: Ctx + Send + Sync + 'static>(linker: &mut Linker<T>) -> Result<()>;
}

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

        impl slight_common::Linkable for $resource {
            fn add_to_linker<Ctx: slight_common::Ctx + Send + Sync + 'static>(linker: &mut slight_common::Linker<Ctx>) -> anyhow::Result<()> {
                $add_to_linker(linker, |ctx| {
                    Ctx::get_table::<$resource, $resource_table>(ctx, $scheme_name)
                })
            }
        }
    };

    ($resource:ty, $resource_table:ty, $state:ty, $lt:tt, $add_to_linker:path, $scheme_name:expr) => {
        impl<$lt> slight_common::Resource for $resource
        where
            $lt: slight_common::Buildable + 'static
        {}
        impl<$lt> slight_common::ResourceTables<dyn slight_common::Resource> for $resource_table
        where
            $lt: slight_common::Buildable + Send + Sync + 'static
        {}
        impl<$lt> slight_common::ResourceBuilder for $resource
        where
            $lt: slight_common::Buildable + Send + Sync + 'static
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

        impl<$lt> slight_common::Linkable for $resource
        where
            $lt: slight_common::Buildable + Send + Sync + 'static
        {
            fn add_to_linker<Ctx: slight_common::Ctx + Send + Sync + 'static>(linker: &mut slight_common::Linker<Ctx>) -> anyhow::Result<()> {
                $add_to_linker(linker, |ctx| {
                    Ctx::get_table::<$resource, $resource_table>(ctx, $scheme_name)
                })
            }
        }
    };
}
