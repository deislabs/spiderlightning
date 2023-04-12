mod implementors;
pub mod providers;

use std::{collections::HashMap, fmt::Debug, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use implementors::*;

use opentelemetry::{global, trace::Tracer};
use slight_common::{impl_resource, BasicState};
use slight_file::Resource;

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
#[derive(Clone, Debug)]
pub struct KeyvalueInner {
    keyvalue_implementor: Arc<dyn KeyvalueImplementor + Send + Sync>,
}

impl KeyvalueInner {
    async fn new(
        keyvalue_implementor: KeyvalueImplementors,
        slight_state: &BasicState,
        name: &str,
    ) -> Self {
        Self {
            keyvalue_implementor: match keyvalue_implementor {
                #[cfg(feature = "filesystem")]
                KeyvalueImplementors::Filesystem => {
                    Arc::new(filesystem::FilesystemImplementor::new(slight_state, name).await)
                }
                #[cfg(feature = "azblob")]
                KeyvalueImplementors::AzBlob => {
                    Arc::new(azblob::AzBlobImplementor::new(slight_state, name).await)
                }
                #[cfg(feature = "awsdynamodb")]
                KeyvalueImplementors::AwsDynamoDb => {
                    Arc::new(awsdynamodb::AwsDynamoDbImplementor::new(slight_state, name).await)
                }
                #[cfg(feature = "redis")]
                KeyvalueImplementors::Redis => {
                    Arc::new(redis::RedisImplementor::new(slight_state, name).await)
                }
            },
        }
    }
}

/// This defines the available implementor implementations for the `Keyvalue` interface.
///
/// As per its' usage in `KeyvalueInner`, it must `derive` `Debug`, and `Clone`.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum KeyvalueImplementors {
    #[cfg(feature = "filesystem")]
    Filesystem,
    #[cfg(feature = "azblob")]
    AzBlob,
    #[cfg(feature = "awsdynamodb")]
    AwsDynamoDb,
    #[cfg(feature = "redis")]
    Redis,
}

impl From<Resource> for KeyvalueImplementors {
    fn from(s: Resource) -> Self {
        match s {
            #[cfg(feature = "filesystem")]
            Resource::KeyvalueFilesystem | Resource::V1KeyvalueFilesystem => Self::Filesystem,
            #[cfg(feature = "azblob")]
            Resource::KeyvalueAzblob | Resource::V1KeyvalueAzblob => Self::AzBlob,
            #[cfg(feature = "awsdynamodb")]
            Resource::KeyvalueAwsDynamoDb | Resource::V1KeyvalueAwsDynamoDb => Self::AwsDynamoDb,
            #[cfg(feature = "redis")]
            Resource::KeyvalueRedis | Resource::V1KeyvalueRedis => Self::Redis,
            p => panic!(
                "failed to match provided name (i.e., '{p}') to any known host implementations"
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
    keyvalue::add_to_linker,
    "keyvalue".to_string()
);

/// This is the implementation for the generated `keyvalue::Keyvalue` trait from the `keyvalue.wit` file.
#[async_trait]
impl keyvalue::Keyvalue for Keyvalue {
    type Keyvalue = KeyvalueInner;

    async fn keyvalue_open(&mut self, name: &str) -> Result<Self::Keyvalue, KeyvalueError> {
        let tracer = global::tracer("spiderlightning");
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

        let inner = tracer
            .in_span(
                format!("opened implementer {}", &state.implementor),
                |_cx| async {
                    Self::Keyvalue::new(state.implementor.clone().into(), &state, name).await
                },
            )
            .await;

        Ok(inner)
    }

    async fn keyvalue_get(
        &mut self,
        self_: &Self::Keyvalue,
        key: &str,
    ) -> Result<Vec<u8>, KeyvalueError> {
        Ok(self_.keyvalue_implementor.get(key).await?)
    }

    async fn keyvalue_set(
        &mut self,
        self_: &Self::Keyvalue,
        key: &str,
        value: &[u8],
    ) -> Result<(), KeyvalueError> {
        self_.keyvalue_implementor.set(key, value).await?;
        Ok(())
    }

    async fn keyvalue_keys(
        &mut self,
        self_: &Self::Keyvalue,
    ) -> Result<Vec<String>, KeyvalueError> {
        Ok(self_.keyvalue_implementor.keys().await?)
    }

    async fn keyvalue_delete(
        &mut self,
        self_: &Self::Keyvalue,
        key: &str,
    ) -> Result<(), KeyvalueError> {
        self_.keyvalue_implementor.delete(key).await?;
        Ok(())
    }
}
