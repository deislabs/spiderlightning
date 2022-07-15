use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

pub use crate::RuntimeContext;
use anyhow::{Context, Result};
use as_any::{AsAny, Downcast};
use crossbeam_channel::Sender;
use events_api::{Event, EventHandlerData};
pub use wasmtime::Linker;

pub type DataT = (
    Box<dyn Resource + Send + Sync>,
    Option<Box<dyn ResourceTables<dyn Resource> + Send + Sync>>,
);
pub type ResourceConfig = String;
pub type ResourceMap = Arc<Mutex<Map>>;
pub type Ctx = RuntimeContext<DataT>;
pub type GuestState = EventHandlerData;

/// A map wrapper type for the resource map
#[derive(Default)]
pub struct Map(HashMap<String, DataT>);

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
        let inner = value.0.get_inner();
        <&dyn std::any::Any>::clone(&inner)
            .downcast_ref::<T>()
            .with_context(|| "failed to acquire matched resource descriptor service")
    }

    pub fn get_dynamic_mut(&mut self, key: &str) -> Result<&mut DataT> {
        let value = self
            .0
            .get_mut(key)
            .ok_or_else(|| anyhow::anyhow!("failed because key '{}' was not found", &key))?;
        Ok(value)
    }
}

/// A trait for wit-bindgen resource tables. see [here](https://github.com/bytecodealliance/wit-bindgen/blob/main/crates/wasmtime/src/table.rs) for more details:
pub trait ResourceTables<T: ?Sized>: AsAny {}

/// An implemented service interface
pub trait Resource: AsAny {
    /// Get inner representation of the resource
    fn get_inner(&self) -> &dyn Any;

    /// check if the resource has changed on key.
    fn watch(
        &mut self,
        data: &str,
        rd: &str,
        key: &str,
        sender: Arc<Mutex<Sender<Event>>>,
    ) -> Result<()>;
}

/// A trait for wit-bindgen host resource composed of a resource
pub trait RuntimeResource {
    type State: Sized;
    fn add_to_linker(linker: &mut Linker<Ctx>) -> Result<()>;
    fn build_data(state: Self::State) -> Result<DataT>;
}

#[macro_export]
#[allow(unknown_lints)]
#[allow(clippy::crate_in_macro_def)]
macro_rules! impl_resource {
    ($resource:ident, $resource_table:ty, $state:ident, $scheme_name:expr) => {
        impl RuntimeResource for $resource {
            type State = $state;
            fn add_to_linker(linker: &mut Linker<Ctx>) -> Result<()> {
                crate::add_to_linker(linker, |cx| {
                    get_table::<Self, $resource_table>(cx, $scheme_name)
                })
            }

            fn build_data(state: Self::State) -> Result<DataT> {
                /// We prepare a default resource with host-provided state.
                /// Then the guest will pass other configuration state to the resource.
                /// This is done in the `<Capability>::open` function.
                let mut resource = Self {
                    host_state: Some(state),
                    ..Default::default()
                };
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
pub fn get<T>(cx: &mut Ctx, resource_key: String) -> &mut T
where
    T: 'static,
{
    let data = cx
        .data
        .get_mut(&resource_key)
        .expect("internal error: Runtime context data is None");

    data.0.as_mut().downcast_mut().unwrap_or_else(|| {
        panic!(
            "internal error: context has key {} but can't be downcast to resource {}",
            &resource_key,
            std::any::type_name::<T>()
        )
    })
}

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
