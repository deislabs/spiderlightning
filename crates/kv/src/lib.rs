mod implementors;
pub mod providers;

/// The `SCHEME_NAME` defines the name under which a resource is
/// identifiable by in a `ResourceMap`.
const SCHEME_NAME: &str = "kv";

use std::sync::{Arc, Mutex};

use anyhow::Result;
use crossbeam_channel::Sender;
use events_api::Event;
use implementors::{azblob::AzBlobImplementor, filesystem::FilesystemImplementor};
use uuid::Uuid;

use runtime::{impl_resource, resource::BasicState};

/// It is mandatory to `use <interface>::*` due to `impl_resource!`.
/// That is because `impl_resource!` accesses the `crate`'s
/// `add_to_linker`, and not the `<interface>::add_to_linker` directly.
use kv::*;
wit_bindgen_wasmtime::export!("../../wit/kv.wit");
wit_error_rs::impl_error!(kv::Error);
wit_error_rs::impl_from!(anyhow::Error, kv::Error::ErrorWithDescription);

/// The `Kv` structure is what will implement the `kv::Kv` trait
/// coming from the generated code of off `kv.wit`.
///
/// It maintains a `host_state`.
pub struct Kv {
    host_state: KvState,
}

/// This is the type of the `host_state` property from our `Kv` structure.
///
/// It holds:
///     - a `kv_implementor` `String` — this comes directly from a
///     user's `slightfile` and it is what allows us to dynamically
///     dispatch to a specific implementor's implentation, and
///     - the `slight_state` (of type `BasicState`) that contains common
///     things received from the slight binary (i.e., the `resource_map`,
///     the `config_type`, and the `config_toml_file_path`).
pub struct KvState {
    kv_implementor: String,
    slight_state: BasicState,
}

impl KvState {
    pub fn new(kv_implementor: String, slight_state: BasicState) -> Self {
        Self {
            kv_implementor,
            slight_state,
        }
    }
}

/// This is the type of the associated type coming from the `kv::Kv` trait
/// implementation.
///
/// It holds:
///     - a `kv_implementor` (i.e., a variant `KvImplementor` `enum`), and
///     - a `resource_descriptor` (i.e., an UUID that uniquely identifies
///     resource's instance).
///
/// It must `derive`:
///     - `Debug` due to a constraint on the associated type.
///     - `Clone` because the `ResourceMap` it will be added onto,
///     must own its' data.
///
/// It must be public because the implementation of `kv::Kv` cannot leak
/// a private type.
#[derive(Debug, Clone)]
pub struct KvInner {
    kv_implementor: KvImplementors,
    resource_descriptor: String,
}

impl KvInner {
    fn new(kv_implementor: &str, slight_state: &BasicState, name: &str) -> Self {
        Self {
            kv_implementor: KvImplementors::new(kv_implementor, slight_state, name),
            resource_descriptor: Uuid::new_v4().to_string(),
        }
    }
}

impl runtime::resource::Watch for KvInner {
    fn watch(&mut self, key: &str, sender: Arc<Mutex<Sender<Event>>>) -> Result<()> {
        match &mut self.kv_implementor {
            KvImplementors::Filesystem(fi) => fi.watch(key, sender),
            _ => todo!(),
        }
    }
}

/// This defines the available implementor implementations for the `Kv` interface.
///
/// As per its' usage in `KvInner`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
enum KvImplementors {
    Filesystem(FilesystemImplementor),
    AzBlob(AzBlobImplementor),
}

impl KvImplementors {
    fn new(kv_implementor: &str, slight_state: &BasicState, name: &str) -> Self {
        match kv_implementor {
            "kv.filesystem" => Self::Filesystem(FilesystemImplementor::new(name)),
            "kv.azblob" => Self::AzBlob(AzBlobImplementor::new(slight_state, name)),
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}

// This implements the `ResourceBuilder`, and `Resource` trait
// for our `Kv` `struct`, and `ResourceTables` for our `kv::KvTables` object.
//
// The `ResourceBuilder` trait provides two functions:
// - `add_to_linker`, and
// - `builda_data`.
//
// The `Resource` and `ResourceTables` traits are empty traits that allow
// grouping of resources through `dyn Resource`, and `dyn ResourceTables`.
impl_resource!(Kv, kv::KvTables<Kv>, KvState, SCHEME_NAME.to_string());

/// This is the implementation for the generated `kv::Kv` trait from the `kv.wit` file.
impl kv::Kv for Kv {
    type Kv = KvInner;

    fn kv_open(&mut self, name: &str) -> Result<Self::Kv, Error> {
        // populate our inner kv object w/ the state received from `slight`
        // (i.e., what type of kv implementor we are using), and the assigned
        // name of the object.
        let inner = Self::Kv::new(
            &self.host_state.kv_implementor,
            &self.host_state.slight_state,
            name,
        );

        self.host_state
            .slight_state
            .resource_map
            .lock()
            .unwrap()
            .set(inner.resource_descriptor.clone(), Box::new(inner.clone()));

        Ok(inner)
    }

    fn kv_get(&mut self, self_: &Self::Kv, key: &str) -> Result<PayloadResult, Error> {
        Ok(match &self_.kv_implementor {
            KvImplementors::Filesystem(fi) => fi.get(key)?,
            KvImplementors::AzBlob(ai) => ai.get(key)?,
        })
    }

    fn kv_set(
        &mut self,
        self_: &Self::Kv,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        match &self_.kv_implementor {
            KvImplementors::Filesystem(fi) => fi.set(key, value)?,
            KvImplementors::AzBlob(ai) => ai.set(key, value)?,
        };
        Ok(())
    }

    fn kv_delete(&mut self, self_: &Self::Kv, key: &str) -> Result<(), Error> {
        match &self_.kv_implementor {
            KvImplementors::Filesystem(fi) => fi.delete(key)?,
            KvImplementors::AzBlob(ai) => ai.delete(key)?,
        };
        Ok(())
    }

    fn kv_watch(&mut self, self_: &Self::Kv, key: &str) -> Result<Observable, Error> {
        Ok(Observable {
            rd: self_.resource_descriptor.clone(),
            key: key.to_string(),
        })
    }
}
