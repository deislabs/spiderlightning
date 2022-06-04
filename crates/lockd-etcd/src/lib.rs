pub use lockd::add_to_linker;
use lockd::*;

wit_bindgen_wasmtime::export!("../../wit/lockd.wit");

use anyhow::Result;
use etcd_client::Client;
use futures::executor::block_on;
use runtime::{
    resource::{get, DataT, HostResource, Linker, Resource, ResourceTables},
    Context,
};
use url::{Position, Url};

mod etcd;

pub struct LockdEtcd {
    client: Client,
}

impl LockdEtcd {
    fn new(endpoint: &str) -> Self {
        let client =
            block_on(Client::connect([endpoint], None)).expect("failed to connect to etcd client");
        Self { client }
    }
}

impl lockd::Lockd for LockdEtcd {
    type ResourceDescriptor = u64;

    fn get_lockd(&mut self) -> Result<Self::ResourceDescriptor, Error> {
        Ok(0)
    }

    fn lock(
        &mut self,
        rd: &Self::ResourceDescriptor,
        lock_name: PayloadParam<'_>,
    ) -> Result<PayloadResult, Error> {
        if *rd != 0 {
            return Err(Error::DescriptorError);
        }

        block_on(etcd::lock(&mut self.client, lock_name)).map_err(|_| Error::OtherError)
    }

    fn grant_lease(
        &mut self,
        rd: &Self::ResourceDescriptor,
        time_to_live_in_secs: i64,
    ) -> Result<i64, Error> {
        if *rd != 0 {
            return Err(Error::DescriptorError);
        }

        block_on(etcd::lease_grant(&mut self.client, time_to_live_in_secs))
            .map_err(|_| Error::OtherError)
    }

    fn lock_with_lease(
        &mut self,
        rd: &Self::ResourceDescriptor,
        lock_name: PayloadParam<'_>,
        lease_id: i64,
    ) -> Result<PayloadResult, Error> {
        if *rd != 0 {
            return Err(Error::DescriptorError);
        }

        block_on(etcd::lock_with_lease(&mut self.client, lock_name, lease_id))
            .map_err(|_| Error::OtherError)
    }

    fn unlock(
        &mut self,
        rd: &Self::ResourceDescriptor,
        lock_key: PayloadParam<'_>,
    ) -> Result<(), Error> {
        if *rd != 0 {
            return Err(Error::DescriptorError);
        }

        block_on(etcd::unlock(&mut self.client, lock_key)).map_err(|_| Error::OtherError)
    }
}

impl Resource for LockdEtcd {
    fn from_url(url: Url) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self::new(&url[Position::AfterPassword..]))
    }
}

impl<T> ResourceTables<dyn Resource> for LockdTables<T> where T: Lockd + 'static {}

impl HostResource for LockdEtcd {
    fn add_to_linker(linker: &mut Linker<Context<DataT>>) -> Result<()> {
        crate::add_to_linker(linker, get::<Self, crate::LockdTables<Self>>)
    }

    fn build_data(url: Url) -> Result<DataT> {
        let mq_azure_servicebus = Self::from_url(url)?;
        Ok((
            Box::new(mq_azure_servicebus),
            Box::new(crate::LockdTables::<Self>::default()),
        ))
    }
}
