use std::sync::{Arc, Mutex};

pub use lockd::add_to_linker;
use lockd::*;

wit_bindgen_wasmtime::export!("../../wit/lockd.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);

use anyhow::{Context, Result};
use crossbeam_channel::Sender;
use etcd_client::Client;
use events_api::Event;
use futures::executor::block_on;
use proc_macro_utils::Resource;
use runtime::{
    impl_resource,
    resource::{
        get_table, Ctx, DataT, Linker, Map, Resource, ResourceMap, ResourceTables, RuntimeResource,
    },
};
use uuid::Uuid;

mod etcd;

const SCHEME_NAME: &str = "etcdlockd";

/// An etcd implementation for the lockd (i.e., distributed locking) Interface
#[derive(Default, Clone, Resource)]
pub struct LockdEtcd {
    inner: Option<Arc<Mutex<Client>>>,
    host_state: Option<ResourceMap>,
}

impl_resource!(
    LockdEtcd,
    lockd::LockdTables<LockdEtcd>,
    ResourceMap,
    SCHEME_NAME.to_string()
);

impl LockdEtcd {
    /// Create a new `LockdEtcd`
    fn new(endpoint: &str) -> Self {
        let client = block_on(Client::connect([endpoint], None))
            .with_context(|| "failed to connect to etcd server")
            .unwrap();
        // ^^^ from my tests with localhost client, this never fails
        Self {
            inner: Some(Arc::new(Mutex::new(client))),
            host_state: None,
        }
    }
}

impl lockd::Lockd for LockdEtcd {
    type Lockd = String;
    /// Construct a new `LockdEtcd` instance
    fn lockd_open(&mut self) -> Result<Self::Lockd, Error> {
        let endpoint = std::env::var("ETCD_ENDPOINT")
            .with_context(|| "failed to read ETCD_ENDPOINT environment variable")?;
        let etcd_lockd = Self::new(&endpoint);
        self.inner = etcd_lockd.inner;

        let rd = Uuid::new_v4().to_string();
        let cloned = self.clone();
        let mut map = Map::lock(&mut self.host_state)?;
        map.set(rd.clone(), (Box::new(cloned), None));
        Ok(rd)
    }

    /// Create a lock without a time to live
    fn lockd_lock(
        &mut self,
        self_: &Self::Lockd,
        lock_name: PayloadParam<'_>,
    ) -> Result<PayloadResult, Error> {
        Uuid::parse_str(self_)
            .with_context(|| "internal error: failed to parse internal handle to this resource")?;

        let map = Map::lock(&mut self.host_state)?;
        let inner = map.get::<Arc<Mutex<Client>>>(self_)?;

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
        Uuid::parse_str(self_)
            .with_context(|| "internal error: failed to parse internal handle to this resource")?;

        let map = Map::lock(&mut self.host_state)?;
        let inner = map.get::<Arc<Mutex<Client>>>(self_)?;

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
        Uuid::parse_str(self_)
            .with_context(|| "internal error: failed to parse internal handle to this resource")?;

        let map = Map::lock(&mut self.host_state)?;
        let inner = map.get::<Arc<Mutex<Client>>>(self_)?;

        block_on(etcd::unlock(&mut inner.lock().unwrap(), lock_key))
            .with_context(|| "failed to unlock")?;
        Ok(())
    }
}
