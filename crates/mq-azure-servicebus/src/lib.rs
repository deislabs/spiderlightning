use std::{
    str::Utf8Error,
    sync::{Arc, Mutex},
};

use anyhow::{Result, Context};
use azure_messaging_servicebus::prelude::*;
use futures::executor::block_on;
use runtime::resource::{get, Context as RuntimeContext, DataT, HostResource, Linker, Resource, ResourceTables};
use url::Url;

pub use mq::add_to_linker;
use mq::*;

pub mod azure;

wit_bindgen_wasmtime::export!("../../wit/mq.wit");

/// A Azure ServiceBus Message Queue binding for the mq interface.
#[derive(Default)]
pub struct MqAzureServiceBus {
    inner: Option<Arc<Mutex<Client>>>,
}

impl MqAzureServiceBus {
    /// Create a new KvAzureBlob.
    pub fn new(
        service_bus_namespace: &str,
        queue_name: &str,
        policy_name: &str,
        policy_key: &str,
    ) -> Self {
        let http_client = azure_core::new_http_client();

        let inner = Some(Arc::new(Mutex::new(
            Client::new(
                http_client,
                service_bus_namespace.to_owned(),
                queue_name.to_owned(),
                policy_name.to_owned(),
                policy_key,
            )
            .unwrap(),
        )));
        Self { inner }
    }
}

impl Resource for MqAzureServiceBus {
    fn from_url(url: Url) -> Result<Self> {
        let service_bus_namespace = url.username();
        let queue_name = url.host_str().unwrap();
        // get environment var AZURE_POLICY_NAME
        let policy_name = std::env::var("AZURE_POLICY_NAME").context("AZURE_POLICY_NAME environment variable not found")?;
        // get environment var AZURE_POLICY_KEY
        let policy_key = std::env::var("AZURE_POLICY_KEY").context("AZURE_POLICY_KEY environment variable not found")?;

        Ok(MqAzureServiceBus::new(
            service_bus_namespace,
            queue_name,
            &policy_name,
            &policy_key,
        ))
    }
}

impl<T> ResourceTables<dyn Resource> for MqTables<T> where T: Mq + 'static {}

impl HostResource for MqAzureServiceBus {
    fn add_to_linker(linker: &mut Linker<RuntimeContext<DataT>>) -> Result<()> {
        crate::add_to_linker(linker, get::<Self, crate::MqTables<Self>>)
    }

    fn build_data(url: Url) -> Result<DataT> {
        let mq_azure_servicebus = Self::from_url(url)?;
        Ok((
            Box::new(mq_azure_servicebus),
            Box::new(crate::MqTables::<Self>::default()),
        ))
    }
}

impl mq::Mq for MqAzureServiceBus {
    type ResourceDescriptor = u64;

    /// Get the resource descriptor for your Azure Service Bus message queue
    fn get_mq(&mut self) -> Result<Self::ResourceDescriptor, Error> {
        Ok(0)
    }

    /// Send a message to your service bus' queue
    fn send(&mut self, rd: &Self::ResourceDescriptor, msg: PayloadParam<'_>) -> Result<(), Error> {
        if *rd != 0 {
            return Err(Error::OtherError);
        }
        block_on(azure::send(
            &mut self.inner.as_ref().unwrap().lock().unwrap(),
            std::str::from_utf8(msg)?.to_string(),
        ))?;
        Ok(())
    }

    /// Receive the top message from your service bus' queue
    fn receive(&mut self, rd: &Self::ResourceDescriptor) -> Result<PayloadResult, Error> {
        if *rd != 0 {
            return Err(Error::OtherError);
        }

        let result = block_on(azure::receive(
            &mut self.inner.as_ref().unwrap().lock().unwrap(),
        ))?;
        Ok(result)
    }
}

impl From<anyhow::Error> for Error {
    fn from(_: anyhow::Error) -> Self {
        Self::OtherError
    }
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Self::OtherError
    }
}
