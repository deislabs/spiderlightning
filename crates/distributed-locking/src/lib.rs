mod implementors;
pub mod providers;

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use implementors::etcd::EtcdImplementor;
use slight_common::{impl_resource, BasicState};

/// It is mandatory to `use <interface>::*` due to `impl_resource!`.
/// That is because `impl_resource!` accesses the `crate`'s
/// `add_to_linker`, and not the `<interface>::add_to_linker` directly.
use distributed_locking::*;
wit_bindgen_wasmtime::export!({paths: ["../../wit/distributed-locking.wit"], async: *});
wit_error_rs::impl_error!(distributed_locking::DistributedLockingError);
wit_error_rs::impl_from!(
    anyhow::Error,
    distributed_locking::DistributedLockingError::UnexpectedError
);
wit_error_rs::impl_from!(
    std::string::FromUtf8Error,
    distributed_locking::DistributedLockingError::UnexpectedError
);

/// The `DistributedLocking` structure is what will implement the `distributed_locking::DistributedLocking` trait
/// coming from the generated code of off `distributed-locking.wit`.
///
/// It holds:
///     - a `distributed_locking_implementor` `String` — this comes directly from a
///     user's `slightfile` and it is what allows us to dynamically
///     dispatch to a specific implementor's implentation, and
///     - the `slight_state` (of type `BasicState`) that contains common
///     things received from the slight binary (i.e., the `config_type`
///     and the `config_toml_file_path`).
#[derive(Clone, Default)]
pub struct DistributedLocking {
    implementor: String,
    capability_store: HashMap<String, BasicState>,
}

impl DistributedLocking {
    pub fn new(implementor: String, capability_store: HashMap<String, BasicState>) -> Self {
        Self {
            implementor,
            capability_store,
        }
    }
}

impl_resource!(
    DistributedLocking,
    distributed_locking::DistributedLockingTables<DistributedLocking>,
    DistributedLockingState,
    distributed_locking::add_to_linker,
    "distributed_locking".to_string()
);

#[async_trait]
impl distributed_locking::DistributedLocking for DistributedLocking {
    type DistributedLocking = DistributedLockingInner;

    async fn distributed_locking_open(
        &mut self,
        name: &str,
    ) -> Result<Self::DistributedLocking, distributed_locking::DistributedLockingError> {
        // populate our inner distributed_locking object w/ the state received from `slight`
        // (i.e., what type of distributed_locking implementor we are using), and the assigned
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

        let inner = Self::DistributedLocking::new(&state.implementor, &state).await;

        Ok(inner)
    }

    async fn distributed_locking_lock(
        &mut self,
        self_: &Self::DistributedLocking,
        lock_name: &[u8],
    ) -> Result<Vec<u8>, distributed_locking::DistributedLockingError> {
        Ok(match &self_.distributed_locking_implementor {
            DistributedLockingImplementor::Etcd(ei) => ei.lock(lock_name).await?,
        })
    }

    async fn distributed_locking_lock_with_time_to_live(
        &mut self,
        self_: &Self::DistributedLocking,
        lock_name: &[u8],
        time_to_live_in_secs: i64,
    ) -> Result<Vec<u8>, distributed_locking::DistributedLockingError> {
        Ok(match &self_.distributed_locking_implementor {
            DistributedLockingImplementor::Etcd(ei) => {
                ei.lock_with_time_to_live(lock_name, time_to_live_in_secs)
                    .await?
            }
        })
    }

    async fn distributed_locking_unlock(
        &mut self,
        self_: &Self::DistributedLocking,
        lock_key: &[u8],
    ) -> Result<(), DistributedLockingError> {
        match &self_.distributed_locking_implementor {
            DistributedLockingImplementor::Etcd(ei) => ei.unlock(lock_key).await?,
        };
        Ok(())
    }
}

/// This is the type of the associated type coming from the `distributed_locking::DistributedLocking` trait
/// implementation.
///
/// It holds:
///     - a `distributed_locking_implementor` (i.e., a variant `DistributedLockingImplementor` `enum`), and
///
/// It must `derive`:
///     - `Debug` due to a constraint on the associated type.
///     - `Clone` because the `ResourceMap` it will be added onto,
///     must own its' data.
///
/// It must be public because the implementation of `distributed_locking::DistributedLocking` cannot leak
/// a private type.
#[derive(Debug, Clone)]
pub struct DistributedLockingInner {
    distributed_locking_implementor: DistributedLockingImplementor,
}

impl DistributedLockingInner {
    async fn new(distributed_locking_implementor: &str, slight_state: &BasicState) -> Self {
        Self {
            distributed_locking_implementor: DistributedLockingImplementor::new(
                distributed_locking_implementor,
                slight_state,
            )
            .await,
        }
    }
}

/// This defines the available implementor implementations for the `DistributedLocking` interface.
///
/// As per its' usage in `DistributedLockingInner`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
enum DistributedLockingImplementor {
    Etcd(EtcdImplementor),
}

impl DistributedLockingImplementor {
    async fn new(distributed_locking_implementor: &str, slight_state: &BasicState) -> Self {
        match distributed_locking_implementor {
            "distributed_locking.etcd" | "lockd.etcd" => {
                Self::Etcd(EtcdImplementor::new(slight_state).await)
            }
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}