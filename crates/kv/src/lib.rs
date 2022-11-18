mod implementors;
pub mod providers;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use async_trait::async_trait;
use crossbeam_channel::Sender;
use implementors::{
    awsdynamodb::AwsDynamoDbImplementor, azblob::AzBlobImplementor,
    filesystem::FilesystemImplementor, redis::RedisImplementor,
};
use slight_events_api::Event;
use uuid::Uuid;

use slight_common::{impl_resource, BasicState};

/// It is mandatory to `use <interface>::*` due to `impl_resource!`.
/// That is because `impl_resource!` accesses the `crate`'s
/// `add_to_linker`, and not the `<interface>::add_to_linker` directly.
use kv::*;
wit_bindgen_wasmtime::export!({paths: ["../../wit/kv.wit"], async: *});
wit_error_rs::impl_error!(kv::Error);
wit_error_rs::impl_from!(anyhow::Error, kv::Error::ErrorWithDescription);

/// The `Kv` structure is what will implement the `kv::Kv` trait
/// coming from the generated code of off `kv.wit`.
///
/// It holds:
///     - a `kv_implementor` `String` — this comes directly from a
///     user's `slightfile` and it is what allows us to dynamically
///     dispatch to a specific implementor's implentation, and
///     - the `slight_state` (of type `BasicState`) that contains common
///     things received from the slight binary (i.e., the `resource_map`,
///     the `config_type`, and the `config_toml_file_path`).
#[derive(Clone, Default)]
pub struct Kv {
    implementor: String,
    capability_store: HashMap<String, BasicState>,
}

impl Kv {
    pub fn new(implementor: String, kv_store: HashMap<String, BasicState>) -> Self {
        Self {
            implementor,
            capability_store: kv_store,
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
    async fn new(kv_implementor: &str, slight_state: &BasicState, name: &str) -> Self {
        Self {
            kv_implementor: KvImplementors::new(kv_implementor, slight_state, name).await,
            resource_descriptor: Uuid::new_v4().to_string(),
        }
    }
}

impl slight_events_api::Watch for KvInner {
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
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
enum KvImplementors {
    Filesystem(FilesystemImplementor),
    AzBlob(AzBlobImplementor),
    AwsDynamoDb(AwsDynamoDbImplementor),
    Redis(RedisImplementor),
}

impl KvImplementors {
    async fn new(kv_implementor: &str, slight_state: &BasicState, name: &str) -> Self {
        match kv_implementor {
            "kv.filesystem" => Self::Filesystem(FilesystemImplementor::new(name)),
            "kv.azblob" => Self::AzBlob(AzBlobImplementor::new(slight_state, name).await),
            "kv.awsdynamodb" => Self::AwsDynamoDb(AwsDynamoDbImplementor::new(name).await),
            "kv.redis" => Self::Redis(RedisImplementor::new(slight_state, name).await),
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}

// This implements the `CapabilityBuilder`, and `Capability` trait
// for our `Kv` `struct`, and `CapabilityIndexTable` for our `kv::KvTables` object.
//
// The `CapabilityBuilder` trait provides two functions:
// - `add_to_linker`, and
// - `builda_data`.
//
// The `Capability` and `CapabilityIndexTable` traits are empty traits that allow
// grouping of resources through `dyn Capability`, and `dyn CapabilityIndexTable`.
impl_resource!(
    Kv,
    kv::KvTables<Kv>,
    KvState,
    kv::add_to_linker,
    "kv".to_string()
);

/// This is the implementation for the generated `kv::Kv` trait from the `kv.wit` file.
#[async_trait]
impl kv::Kv for Kv {
    type Kv = KvInner;

    async fn kv_open(&mut self, name: &str) -> Result<Self::Kv, Error> {
        // populate our inner kv object w/ the state received from `slight`
        // (i.e., what type of kv implementor we are using), and the assigned
        // name of the object.

        let state = if let Some(r) = self.capability_store.get(name) {
            r.clone()
        } else if let Some(r) = self.capability_store.get(&self.implementor) {
            r.clone()
        } else {
            panic!(
                "could not find capability under name '{}' for implementor '{}'",
                name, &self.implementor
            );
        };

        tracing::log::info!("Opening implementor {}", &state.implementor);

        let inner = Self::Kv::new(&state.implementor, &state, name).await;

        state
            .resource_map
            .lock()
            .unwrap()
            .set(inner.resource_descriptor.clone(), Box::new(inner.clone()));

        Ok(inner)
    }

    async fn kv_get(&mut self, self_: &Self::Kv, key: &str) -> Result<PayloadResult, Error> {
        Ok(match &self_.kv_implementor {
            KvImplementors::Filesystem(fi) => fi.get(key)?,
            KvImplementors::AzBlob(ai) => ai.get(key).await?,
            KvImplementors::AwsDynamoDb(adp) => adp.get(key).await?,
            KvImplementors::Redis(ri) => ri.get(key)?,
        })
    }

    async fn kv_set(
        &mut self,
        self_: &Self::Kv,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        match &self_.kv_implementor {
            KvImplementors::Filesystem(fi) => fi.set(key, value)?,
            KvImplementors::AzBlob(ai) => ai.set(key, value).await?,
            KvImplementors::AwsDynamoDb(adp) => adp.set(key, value).await?,
            KvImplementors::Redis(ri) => ri.set(key, value)?,
        };
        Ok(())
    }

    async fn kv_keys(&mut self, self_: &Self::Kv) -> Result<Vec<String>, Error> {
        Ok(match &self_.kv_implementor {
            KvImplementors::Filesystem(fi) => fi.keys()?,
            KvImplementors::AzBlob(ai) => ai.keys().await?,
            KvImplementors::AwsDynamoDb(adp) => adp.keys().await?,
            KvImplementors::Redis(ri) => ri.keys()?,
        })
    }

    async fn kv_delete(&mut self, self_: &Self::Kv, key: &str) -> Result<(), Error> {
        match &self_.kv_implementor {
            KvImplementors::Filesystem(fi) => fi.delete(key)?,
            KvImplementors::AzBlob(ai) => ai.delete(key).await?,
            KvImplementors::AwsDynamoDb(adp) => adp.delete(key).await?,
            KvImplementors::Redis(ri) => ri.delete(key)?,
        };
        Ok(())
    }

    async fn kv_watch(&mut self, self_: &Self::Kv, key: &str) -> Result<Observable, Error> {
        Ok(Observable {
            rd: self_.resource_descriptor.clone(),
            key: key.to_string(),
        })
    }
}
