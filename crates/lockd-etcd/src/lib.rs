use std::fmt::Debug;
use std::sync::{Arc, Mutex};

pub use lockd::add_to_linker;
use lockd::*;

wit_bindgen_wasmtime::export!("../../wit/lockd.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);
wit_error_rs::impl_from!(std::string::FromUtf8Error, Error::ErrorWithDescription);

use anyhow::{Context, Result};
use crossbeam_channel::Sender;
use etcd_client::Client;
use events_api::Event;
use futures::executor::block_on;
use proc_macro_utils::{Resource, Watch};
use runtime::{
    impl_resource,
    resource::{
        get_table, BasicState, Ctx, HostState, Linker, Resource, ResourceBuilder, ResourceTables,
        Watch,
    },
};
use uuid::Uuid;

mod etcd;

const SCHEME_NAME: &str = "etcdlockd";

/// An etcd implementation for the lockd (i.e., distributed locking) Interface
#[derive(Default, Clone, Resource)]
pub struct LockdEtcd {
    host_state: BasicState,
}

impl_resource!(
    LockdEtcd,
    lockd::LockdTables<LockdEtcd>,
    BasicState,
    SCHEME_NAME.to_string()
);

#[derive(Default, Clone, Watch)]
pub struct LockdEtcdInner {
    client: Option<Arc<Mutex<Client>>>,
}

impl Debug for LockdEtcdInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LockdEtcdInner")
    }
}

impl LockdEtcdInner {
    /// Create a new `LockdEtcd`
    fn new(endpoint: &str) -> Self {
        let client = block_on(Client::connect([endpoint], None))
            .with_context(|| "failed to connect to etcd server")
            .unwrap();
        // ^^^ from my tests with localhost client, this never fails
        Self {
            client: Some(Arc::new(Mutex::new(client))),
        }
    }
}

impl lockd::Lockd for LockdEtcd {
    type Lockd = LockdEtcdInner;
    /// Construct a new `LockdEtcd` instance
    fn lockd_open(&mut self) -> Result<Self::Lockd, Error> {
        let endpoint = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "ETCD_ENDPOINT",
            &self.host_state.config_toml_file_path,
        )?)?;
        let etcd_lockd_guest = Self::Lockd::new(&endpoint);

        let rd = Uuid::new_v4().to_string();
        self.host_state
            .resource_map
            .lock()
            .unwrap()
            .set(rd, Box::new(etcd_lockd_guest.clone()));
        Ok(etcd_lockd_guest)
    }

    /// Create a lock without a time to live
    fn lockd_lock(
        &mut self,
        self_: &Self::Lockd,
        lock_name: PayloadParam<'_>,
    ) -> Result<PayloadResult, Error> {
        let inner = self_.client.as_ref().unwrap();
        let pr = block_on(etcd::lock(&mut inner.lock().unwrap(), lock_name))
            .with_context(|| "failed to acquire lock")?;
        Ok(pr)
    }

    /// Create a lock with a time to live. Once the time to live runs out, the lock is no longer locking
    fn lockd_lock_with_time_to_live(
        &mut self,
        self_: &Self::Lockd,
        lock_name: PayloadParam<'_>,
        time_to_live_in_secs: i64,
    ) -> Result<PayloadResult, Error> {
        let inner = self_.client.as_ref().unwrap();
        let pr = block_on(etcd::lock_with_lease(
            &mut inner.lock().unwrap(),
            lock_name,
            time_to_live_in_secs,
        ))
        .with_context(|| "failed to acquire lock with time to live")?;
        Ok(pr)
    }

    /// Unlock a lock
    fn lockd_unlock(
        &mut self,
        self_: &Self::Lockd,
        lock_key: PayloadParam<'_>,
    ) -> Result<(), Error> {
        let inner = self_.client.as_ref().unwrap();
        block_on(etcd::unlock(&mut inner.lock().unwrap(), lock_key))
            .with_context(|| "failed to unlock")?;
        Ok(())
    }
}
