use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use crate::RuntimeContext;
use anyhow::Result;
use as_any::{AsAny, Downcast};
use crossbeam_channel::Sender;
use events_api::{Event, EventHandlerData};
pub use wasmtime::Linker;

/// HostState abstract out generated bindings for the resource,
/// and the resource table. It is used as a type in the `RuntimeContext`
/// primarily for linking host defined resources for capabilities.
pub type HostState = (
    Box<dyn Resource + Send + Sync>,
    Option<Box<dyn ResourceTables<dyn Resource> + Send + Sync>>,
);

/// Watch state is a dynamic type for resources that implement the `watch` function.
pub type WatchState = Box<dyn Watch + Send + Sync>;

/// A alias to sharable state table
pub type ResourceMap = Arc<Mutex<StateTable>>;

/// Runtime Context for the wasm module
pub type Ctx = RuntimeContext<HostState>;

/// Guest data for event handler
/// TODO (Joe): abstract this to a general guest data
pub type GuestData = EventHandlerData;

/// A convenient Struct for the most basic state a resource can have
#[derive(Clone, Default)]
pub struct BasicState {
    pub resource_map: ResourceMap,
    pub secret_store: String,
    pub config_toml_file_path: String,
}

impl BasicState {
    pub fn new(resource_map: ResourceMap, secret_store: &str, config_toml_file_path: &str) -> Self {
        Self {
            resource_map,
            secret_store: secret_store.to_string(),
            config_toml_file_path: config_toml_file_path.to_string(),
        }
    }
}
/// A state table that is indexed by each resource unique identifier.
/// The state table stores each resource inner of type WatchState.
#[derive(Default)]
pub struct StateTable(HashMap<String, WatchState>);

impl StateTable {
    /// A wrapper function for inserting a key, value pair in the map
    pub fn set(&mut self, key: String, value: WatchState) {
        self.0.insert(key, value);
    }

    /// A wrapper function for getting a mutable value from the map
    pub fn get_mut(&mut self, key: &str) -> Result<&mut WatchState> {
        let value = self
            .0
            .get_mut(key)
            .ok_or_else(|| anyhow::anyhow!("failed because key '{}' was not found", &key))?;
        Ok(value)
    }

    /// A wrapper function for getting a value from the map
    pub fn get(&mut self, key: &str) -> Result<&WatchState> {
        let value = self
            .0
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("failed because key '{}' was not found", &key))?;
        Ok(value)
    }
}

/// A trait for wit-bindgen resource tables. see [here](https://github.com/bytecodealliance/wit-bindgen/blob/main/crates/wasmtime/src/table.rs) for more details:
pub trait ResourceTables<T: ?Sized>: AsAny {}

/// An implemented service interface
pub trait Resource: AsAny {}

/// A trait for inner representation of the resource
pub trait Watch {
    fn watch(&mut self, key: &str, sender: Arc<Mutex<Sender<Event>>>) -> Result<()>;
}

/// A trait for wit-bindgen host resource composed of a resource
pub trait ResourceBuilder {
    type State: Sized;
    fn add_to_linker(linker: &mut Linker<Ctx>) -> Result<()>;
    fn build_data(state: Self::State) -> Result<HostState>;
}

#[macro_export]
#[allow(unknown_lints)]
#[allow(clippy::crate_in_macro_def)]
macro_rules! impl_resource {
    ($resource:ident, $resource_table:ty, $state:ident, $scheme_name:expr) => {
        impl ResourceBuilder for $resource {
            type State = $state;
            fn add_to_linker(linker: &mut Linker<Ctx>) -> Result<()> {
                crate::add_to_linker(linker, |cx| {
                    get_table::<Self, $resource_table>(cx, $scheme_name)
                })
            }

            fn build_data(state: Self::State) -> Result<HostState> {
                /// We prepare a default resource with host-provided state.
                /// Then the guest will pass other configuration state to the resource.
                /// This is done in the `<Capability>::open` function.
                let mut resource = Self { host_state: state };
                Ok((
                    Box::new(resource),
                    Some(Box::new(<$resource_table>::default())),
                ))
            }
        }

        impl ResourceTables<dyn Resource> for $resource_table {}
    };
}

pub use impl_resource;

/// Dynamically dispatch to respective host resource
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
        data.0.as_mut().downcast_mut().unwrap_or_else(|| {
            panic!(
                "internal error: context has key {} but can't be downcast to resource {}",
                &resource_key,
                std::any::type_name::<T>()
            )
        }),
        data.1
            .as_mut()
            .unwrap_or_else(|| {
                panic!(
                    "internal error: table {} is not initialized",
                    std::any::type_name::<TTable>()
                )
            })
            .as_mut()
            .downcast_mut()
            .unwrap_or_else(|| {
                panic!(
                    "internal error: context has key {} but can't be downcast to resource_table {}",
                    &resource_key,
                    std::any::type_name::<TTable>()
                )
            }),
    )
}
