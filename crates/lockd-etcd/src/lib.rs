use std::sync::{Mutex, Arc};

pub use lockd::add_to_linker;
use lockd::*;

wit_bindgen_wasmtime::export!("../../wit/lockd.wit");

use anyhow::Result;
use etcd_client::Client;
use futures::executor::block_on;
use proc_macro_utils::{Resource, RuntimeResource};
use runtime::resource::{
    get, Context as RuntimeContext, DataT, Linker, Resource, ResourceMap, RuntimeResource,
};
use uuid::Uuid;

mod etcd;

const SCHEME_NAME: &str = "etcdlockd";

#[derive(Default, Clone, Resource, RuntimeResource)]
pub struct LockdEtcd {
    inner: Option<Arc<Mutex<Client>>>,
    resource_map: Option<ResourceMap>,
}

impl LockdEtcd {
    fn new(endpoint: &str) -> Self {
        let client =
            block_on(Client::connect([endpoint], None)).expect("failed to connect to etcd client");
        Self {
            inner: Some(Arc::new(Mutex::new(client))),
            resource_map: None,
        }
    }
}

impl lockd::Lockd for LockdEtcd {
    fn get_lockd(&mut self, name: &str) -> Result<ResourceDescriptorResult, Error> {
        let etcd_lockd = Self::new(&name);
        self.inner = etcd_lockd.inner;
        let uuid = Uuid::new_v4();
        let rd = uuid.to_string();
        let cloned = self.clone();
        let mut map = self
            .resource_map
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        map.set(rd.clone(), Box::new(cloned))?;
        Ok(rd)
    }

    fn lock(
        &mut self,
        rd: ResourceDescriptorParam,
        lock_name: PayloadParam<'_>,
    ) -> Result<PayloadResult, Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }

        let map = self
        .resource_map
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
        .lock()
        .unwrap();
        let inner = map.get::<Arc<Mutex<Client>>>(rd)?;

        let pr = block_on(etcd::lock( &mut inner.lock().unwrap(), lock_name)).map_err(|_| Error::OtherError)?;
        Ok(pr)
    }

    fn lock_with_time_to_live(
        &mut self,
        rd: ResourceDescriptorParam,
        lock_name: PayloadParam<'_>,
        time_to_live_in_secs: i64,
    ) -> Result<PayloadResult, Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }

        let map = self
        .resource_map
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
        .lock()
        .unwrap();
        let inner = map.get::<Arc<Mutex<Client>>>(rd)?;

        let pr = block_on(etcd::lock_with_lease(
            &mut inner.lock().unwrap(),
            lock_name,
            time_to_live_in_secs,
        ))
        .map_err(|_| Error::OtherError)?;
        Ok(pr)
    }

    fn unlock(
        &mut self,
        rd: ResourceDescriptorParam,
        lock_key: PayloadParam<'_>,
    ) -> Result<(), Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }

        let map = self
        .resource_map
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
        .lock()
        .unwrap();
        let inner = map.get::<Arc<Mutex<Client>>>(rd)?;

        let pr = block_on(etcd::unlock(&mut inner.lock().unwrap(), lock_key))
            .map_err(|_| Error::OtherError)?;
        Ok(pr)
    }
}

impl From<anyhow::Error> for Error {
    fn from(_: anyhow::Error) -> Self {
        Self::OtherError
    }
}
