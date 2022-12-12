mod implementors;
pub mod providers;

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use implementors::{
    awsdynamodb::AwsDynamoDbImplementor, azblob::AzBlobImplementor,
    filesystem::FilesystemImplementor, redis::RedisImplementor,
};

use slight_common::{impl_resource, BasicState};

/// It is mandatory to `use <interface>::*` due to `impl_resource!`.
/// That is because `impl_resource!` accesses the `crate`'s
/// `add_to_linker`, and not the `<interface>::add_to_linker` directly.
use keyvalue::*;
wit_bindgen_wasmtime::export!({paths: ["../../wit/keyvalue.wit"], async: *});
wit_error_rs::impl_error!(keyvalue::KeyvalueError);
wit_error_rs::impl_from!(anyhow::Error, keyvalue::KeyvalueError::UnexpectedError);

/// The `Keyvalue` structure is what will implement the `keyvalue::Keyvalue` trait
/// coming from the generated code of off `keyvalue.wit`.
///
/// It holds:
///     - a `keyvalue_implementor` `String` — this comes directly from a
///     user's `slightfile` and it is what allows us to dynamically
///     dispatch to a specific implementor's implentation, and
///     - the `slight_state` (of type `BasicState`) that contains common
///     things received from the slight binary (i.e., the `config_type`
///     and the `config_toml_file_path`).
#[derive(Clone, Default)]
pub struct Keyvalue {
    implementor: String,
    capability_store: HashMap<String, BasicState>,
}

impl Keyvalue {
    pub fn new(implementor: String, keyvalue_store: HashMap<String, BasicState>) -> Self {
        Self {
            implementor,
            capability_store: keyvalue_store,
        }
    }
}

/// This is the type of the associated type coming from the `keyvalue::Keyvalue` trait
/// implementation.
///
/// It holds:
///     - a `keyvalue_implementor` (i.e., a variant `KeyvalueImplementor` `enum`), and
///     - a `resource_descriptor` (i.e., an UUID that uniquely identifies
///     resource's instance).
///
/// It must `derive`:
///     - `Debug` due to a constraint on the associated type.
///     - `Clone` because the `ResourceMap` it will be added onto,
///     must own its' data.
///
/// It must be public because the implementation of `keyvalye::Keyvalue` cannot leak
/// a private type.
#[derive(Debug, Clone)]
pub struct KeyvalueInner {
    keyvalue_implementor: KeyvalueImplementors,
}

impl KeyvalueInner {
    async fn new(keyvalue_implementor: &str, slight_state: &BasicState, name: &str) -> Self {
        Self {
            keyvalue_implementor: KeyvalueImplementors::new(keyvalue_implementor, slight_state, name).await,
        }
    }
}

/// This defines the available implementor implementations for the `Keyvalue` interface.
///
/// As per its' usage in `KeyvalueInner`, it must `derive` `Debug`, and `Clone`.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
enum KeyvalueImplementors {
    Filesystem(FilesystemImplementor),
    AzBlob(AzBlobImplementor),
    AwsDynamoDb(AwsDynamoDbImplementor),
    Redis(RedisImplementor),
}

impl KeyvalueImplementors {
    async fn new(keyvalue_implementor: &str, slight_state: &BasicState, name: &str) -> Self {
        match keyvalue_implementor {
            "keyvalue.filesystem" => Self::Filesystem(FilesystemImplementor::new(name)),
            "keyvalue.azblob" => Self::AzBlob(AzBlobImplementor::new(slight_state, name).await),
            "keyvalue.awsdynamodb" => Self::AwsDynamoDb(AwsDynamoDbImplementor::new(name).await),
            "keyvalue.redis" => Self::Redis(RedisImplementor::new(slight_state, name).await),
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}

// This implements the `CapabilityBuilder`, and `Capability` trait
// for our `Keyvalue` `struct`, and `CapabilityIndexTable` for our `keyvalue::KeyvalueTables` object.
//
// The `CapabilityBuilder` trait provides two functions:
// - `add_to_linker`, and
// - `build_data`.
//
// The `Capability` and `CapabilityIndexTable` traits are empty traits that allow
// grouping of resources through `dyn Capability`, and `dyn CapabilityIndexTable`.
impl_resource!(
    Keyvalue,
    keyvalue::KeyvalueTables<Keyvalue>,
    KeyvalueState,
    keyvalue::add_to_linker,
    "keyvalue".to_string()
);

/// This is the implementation for the generated `keyvalue::Keyvalue` trait from the `keyvalue.wit` file.
#[async_trait]
impl keyvalue::Keyvalue for Keyvalue {
    type Keyvalue = KeyvalueInner;

    async fn keyvalue_open(&mut self, name: &str) -> Result<Self::Keyvalue, KeyvalueError> {
        // populate our inner keyvalue object w/ the state received from `slight`
        // (i.e., what type of keyvalue implementor we are using), and the assigned
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

        let inner = Self::Keyvalue::new(&state.implementor, &state, name).await;

        Ok(inner)
    }

    async fn keyvalue_get(&mut self, self_: &Self::Keyvalue, key: &str) -> Result<Vec<u8>, KeyvalueError> {
        Ok(match &self_.keyvalue_implementor {
            KeyvalueImplementors::Filesystem(fi) => fi.get(key)?,
            KeyvalueImplementors::AzBlob(ai) => ai.get(key).await?,
            KeyvalueImplementors::AwsDynamoDb(adp) => adp.get(key).await?,
            KeyvalueImplementors::Redis(ri) => ri.get(key)?,
        })
    }

    async fn keyvalue_set(
        &mut self,
        self_: &Self::Keyvalue,
        key: &str,
        value: &[u8],
    ) -> Result<(), KeyvalueError> {
        match &self_.keyvalue_implementor {
            KeyvalueImplementors::Filesystem(fi) => fi.set(key, value)?,
            KeyvalueImplementors::AzBlob(ai) => ai.set(key, value).await?,
            KeyvalueImplementors::AwsDynamoDb(adp) => adp.set(key, value).await?,
            KeyvalueImplementors::Redis(ri) => ri.set(key, value)?,
        };
        Ok(())
    }

    async fn keyvalue_keys(&mut self, self_: &Self::Keyvalue) -> Result<Vec<String>, KeyvalueError> {
        Ok(match &self_.keyvalue_implementor {
            KeyvalueImplementors::Filesystem(fi) => fi.keys()?,
            KeyvalueImplementors::AzBlob(ai) => ai.keys().await?,
            KeyvalueImplementors::AwsDynamoDb(adp) => adp.keys().await?,
            KeyvalueImplementors::Redis(ri) => ri.keys()?,
        })
    }

    async fn keyvalue_delete(&mut self, self_: &Self::Keyvalue, key: &str) -> Result<(), KeyvalueError> {
        match &self_.keyvalue_implementor {
            KeyvalueImplementors::Filesystem(fi) => fi.delete(key)?,
            KeyvalueImplementors::AzBlob(ai) => ai.delete(key).await?,
            KeyvalueImplementors::AwsDynamoDb(adp) => adp.delete(key).await?,
            KeyvalueImplementors::Redis(ri) => ri.delete(key)?,
        };
        Ok(())
    }
}
