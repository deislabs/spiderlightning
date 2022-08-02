mod implementors;
pub mod providers;

/// The `SCHEME_NAME` defines the name under which a resource is
/// identifiable by in a `ResourceMap`.
const SCHEME_NAME: &str = "lockd";

use std::sync::{Arc, Mutex};

use anyhow::Result;
use crossbeam_channel::Sender;
use uuid::Uuid;

use events_api::Event;
use implementors::etcd::EtcdImplementor;
use runtime::{impl_resource, resource::BasicState};

/// It is mandatory to `use <interface>::*` due to `impl_resource!`.
/// That is because `impl_resource!` accesses the `crate`'s
/// `add_to_linker`, and not the `<interface>::add_to_linker` directly.
use lockd::*;
wit_bindgen_wasmtime::export!("../../wit/lockd.wit");
wit_error_rs::impl_error!(lockd::Error);
wit_error_rs::impl_from!(anyhow::Error, lockd::Error::ErrorWithDescription);
wit_error_rs::impl_from!(
    std::string::FromUtf8Error,
    lockd::Error::ErrorWithDescription
);

/// The `Lockd` structure is what will implement the `lockd::Lockd` trait
/// coming from the generated code of off `lockd.wit`.
///
/// It maintains a `host_state`.
pub struct Lockd {
    host_state: LockdState,
}

impl_resource!(
    Lockd,
    lockd::LockdTables<Lockd>,
    LockdState,
    SCHEME_NAME.to_string()
);

/// This is the type of the `host_state` property from our `Lockd` structure.
///
/// It holds:
///     - a `lockd_implementor` `String` — this comes directly from a
///     user's `slightfile` and it is what allows us to dynamically
///     dispatch to a specific implementor's implentation, and
///     - the `slight_state` (of type `BasicState`) that contains common
///     things received from the slight binary (i.e., the `resource_map`,
///     the `config_type`, and the `config_toml_file_path`).
pub struct LockdState {
    lockd_implementor: String,
    slight_state: BasicState,
}

impl LockdState {
    pub fn new(lockd_implementor: String, slight_state: BasicState) -> Self {
        Self {
            lockd_implementor,
            slight_state,
        }
    }
}

impl lockd::Lockd for Lockd {
    type Lockd = LockdInner;

    fn lockd_open(&mut self) -> Result<Self::Lockd, Error> {
        // populate our inner lockd object w/ the state received from `slight`
        // (i.e., what type of lockd implementor we are using), and the assigned
        // name of the object.
        let inner = Self::Lockd::new(
            &self.host_state.lockd_implementor,
            &self.host_state.slight_state,
        );

        self.host_state
            .slight_state
            .resource_map
            .lock()
            .unwrap()
            .set(inner.resource_descriptor.clone(), Box::new(inner.clone()));

        Ok(inner)
    }

    fn lockd_lock(
        &mut self,
        self_: &Self::Lockd,
        lock_name: PayloadParam<'_>,
    ) -> Result<PayloadResult, Error> {
        Ok(match &self_.lockd_implementor {
            LockdImplementor::Etcd(ei) => ei.lock(lock_name)?,
        })
    }

    fn lockd_lock_with_time_to_live(
        &mut self,
        self_: &Self::Lockd,
        lock_name: PayloadParam<'_>,
        time_to_live_in_secs: i64,
    ) -> Result<PayloadResult, Error> {
        Ok(match &self_.lockd_implementor {
            LockdImplementor::Etcd(ei) => {
                ei.lock_with_time_to_live(lock_name, time_to_live_in_secs)?
            }
        })
    }

    fn lockd_unlock(
        &mut self,
        self_: &Self::Lockd,
        lock_key: PayloadParam<'_>,
    ) -> Result<(), Error> {
        match &self_.lockd_implementor {
            LockdImplementor::Etcd(ei) => ei.unlock(lock_key)?,
        };
        Ok(())
    }
}

/// This is the type of the associated type coming from the `lockd::Lockd` trait
/// implementation.
///
/// It holds:
///     - a `lockd_implementor` (i.e., a variant `LockdImplementor` `enum`), and
///     - a `resource_descriptor` (i.e., an UUID that uniquely identifies
///     resource's instance).
///
/// It must `derive`:
///     - `Debug` due to a constraint on the associated type.
///     - `Clone` because the `ResourceMap` it will be added onto,
///     must own its' data.
///
/// It must be public because the implementation of `lockd::Lockd` cannot leak
/// a private type.
#[derive(Debug, Clone)]
pub struct LockdInner {
    lockd_implementor: LockdImplementor,
    resource_descriptor: String,
}

impl LockdInner {
    fn new(lockd_implementor: &str, slight_state: &BasicState) -> Self {
        Self {
            lockd_implementor: LockdImplementor::new(lockd_implementor, slight_state),
            resource_descriptor: Uuid::new_v4().to_string(),
        }
    }
}

impl runtime::resource::Watch for LockdInner {
    fn watch(&mut self, key: &str, sender: Arc<Mutex<Sender<Event>>>) -> Result<()> {
        todo!(
            "got {} and {:?}, but got nothing to do with it yet",
            key,
            sender
        );
    }
}

/// This defines the available implementor implementations for the `Lockd` interface.
///
/// As per its' usage in `LockdInner`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
enum LockdImplementor {
    Etcd(EtcdImplementor),
}

impl LockdImplementor {
    fn new(lockd_implementor: &str, slight_state: &BasicState) -> Self {
        match lockd_implementor {
            "lockd.etcd" => Self::Etcd(EtcdImplementor::new(slight_state)),
            _ => panic!("failed to match provided kv name to any known host implementations"),
        }
    }
}
