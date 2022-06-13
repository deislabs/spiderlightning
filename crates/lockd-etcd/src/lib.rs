pub use lockd::add_to_linker;
use lockd::*;

wit_bindgen_wasmtime::export!("../../wit/lockd.wit");

use anyhow::Result;
use etcd_client::Client;
use futures::executor::block_on;
use runtime::{
    resource::{get, DataT, HostResource, Linker, Resource},
    Context,
};
use url::{Position, Url};

mod etcd;

const SCHEMA_NAME: &str = "etcdlockd";

#[derive(Clone)]
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
    fn get_lockd(&mut self) -> Result<ResourceDescriptor, Error> {
        Ok(0)
    }

    fn lock(
        &mut self,
        rd: ResourceDescriptor,
        lock_name: PayloadParam<'_>,
    ) -> Result<PayloadResult, Error> {
        if rd != 0 {
            return Err(Error::DescriptorError);
        }

        block_on(etcd::lock(&mut self.client, lock_name)).map_err(|_| Error::OtherError)
    }

    fn lock_with_time_to_live(
        &mut self,
        rd: ResourceDescriptor,
        lock_name: PayloadParam<'_>,
        time_to_live_in_secs: i64,
    ) -> Result<PayloadResult, Error> {
        if rd != 0 {
            return Err(Error::DescriptorError);
        }

        block_on(etcd::lock_with_lease(
            &mut self.client,
            lock_name,
            time_to_live_in_secs,
        ))
        .map_err(|_| Error::OtherError)
    }

    fn unlock(&mut self, rd: ResourceDescriptor, lock_key: PayloadParam<'_>) -> Result<(), Error> {
        if rd != 0 {
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

impl HostResource for LockdEtcd {
    fn add_to_linker(linker: &mut Linker<Context<DataT>>) -> Result<()> {
        crate::add_to_linker(linker, |cx| get::<Self>(cx, SCHEMA_NAME.to_string()))
    }

    fn build_data(url: Url) -> Result<DataT> {
        let mq_azure_servicebus = Self::from_url(url)?;
        Ok(Box::new(mq_azure_servicebus))
    }
}
