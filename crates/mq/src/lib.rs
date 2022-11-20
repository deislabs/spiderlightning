mod implementors;
pub mod providers;

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use implementors::{azsbus::AzSbusImplementor, filesystem::FilesystemImplementor};
use slight_common::{impl_resource, BasicState};

/// It is mandatory to `use <interface>::*` due to `impl_resource!`.
/// That is because `impl_resource!` accesses the `crate`'s
/// `add_to_linker`, and not the `<interface>::add_to_linker` directly.
use mq::*;
wit_bindgen_wasmtime::export!({paths: ["../../wit/mq.wit"], async: *});
wit_error_rs::impl_error!(mq::Error);
wit_error_rs::impl_from!(anyhow::Error, mq::Error::ErrorWithDescription);

/// The `Mq` structure is what will implement the `mq::Mq` trait
/// coming from the generated code of off `mq.wit`.
///
/// It holds:
///     - a `mq_implementor` `String` — this comes directly from a
///     user's `slightfile` and it is what allows us to dynamically
///     dispatch to a specific implementor's implentation, and
///     - the `slight_state` (of type `BasicState`) that contains common
///     things received from the slight binary (i.e., the `config_type`,
///     and the `config_toml_file_path`).
#[derive(Clone, Default)]
pub struct Mq {
    implementor: String,
    capability_store: HashMap<String, BasicState>,
}

impl Mq {
    pub fn new(implementor: String, capability_store: HashMap<String, BasicState>) -> Self {
        Self {
            implementor,
            capability_store,
        }
    }
}

// This implements the `CapabilityBuilder`, and `Capability` trait
// for our `Mq` `struct`, and `CapabilityIndexTable` for our `mq::MqTables` object.
//
// The `CapabilityBuilder` trait provides two functions:
// - `add_to_linker`, and
// - `builda_data`.
//
// The `Capability` and `CapabilityIndexTable` traits are empty traits that allow
// grouping of resources through `dyn Capability`, and `dyn CapabilityIndexTable`.
impl_resource!(
    Mq,
    mq::MqTables<Mq>,
    MqState,
    mq::add_to_linker,
    "mq".to_string()
);

#[async_trait]
impl mq::Mq for Mq {
    type Mq = MqInner;

    async fn mq_open(&mut self, name: &str) -> Result<Self::Mq, Error> {
        // populate our inner mq object w/ the state received from `slight`
        // (i.e., what type of mq implementor we are using), and the assigned
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

        let inner = Self::Mq::new(&state.implementor, &state, name).await;

        Ok(inner)
    }

    async fn mq_send(&mut self, self_: &Self::Mq, msg: PayloadParam<'_>) -> Result<(), Error> {
        match &self_.mq_implementor {
            MqImplementor::Filesystem(fi) => fi.send(msg)?,
            MqImplementor::AzSbus(ai) => ai.send(msg).await?,
        };
        Ok(())
    }

    async fn mq_receive(&mut self, self_: &Self::Mq) -> Result<PayloadResult, Error> {
        Ok(match &self_.mq_implementor {
            MqImplementor::Filesystem(fi) => fi.receive()?,
            MqImplementor::AzSbus(ai) => ai.receive().await?,
        })
    }
}

/// This is the type of the associated type coming from the `mq::Mq` trait
/// implementation.
///
/// It holds:
///     - a `mq_implementor` (i.e., a variant `MqImplementor` `enum`)
///
/// It must `derive`:
///     - `Debug` due to a constraint on the associated type.
///     - `Clone` because the `ResourceMap` it will be added onto,
///     must own its' data.
///
/// It must be public because the implementation of `mq::Mq` cannot leak
/// a private type.
#[derive(Debug, Clone)]
pub struct MqInner {
    mq_implementor: MqImplementor,
}

impl MqInner {
    async fn new(mq_implementor: &str, slight_state: &BasicState, name: &str) -> Self {
        Self {
            mq_implementor: MqImplementor::new(mq_implementor, slight_state, name).await,
        }
    }
}

/// This defines the available implementor implementations for the `Mq` interface.
///
/// As per its' usage in `MqInner`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
enum MqImplementor {
    Filesystem(FilesystemImplementor),
    AzSbus(AzSbusImplementor),
}

impl MqImplementor {
    async fn new(mq_implementor: &str, slight_state: &BasicState, name: &str) -> Self {
        match mq_implementor {
            "mq.filesystem" => Self::Filesystem(FilesystemImplementor::new(name)),
            "mq.azsbus" => Self::AzSbus(AzSbusImplementor::new(slight_state, name).await),
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}
