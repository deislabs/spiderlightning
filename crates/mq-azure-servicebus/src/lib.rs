use std::{sync::{Arc, Mutex}, str::Utf8Error};

use azure_sdk_service_bus::prelude::*;
use capability::{Resource, ResourceTables};
use url::Url;
use anyhow::Result;
use futures::executor::block_on;

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
        event_hub_name: &str,
        policy_name: &str,
        policy_key: &str,
    ) -> Self {
        let inner = Some(Arc::new(Mutex::new(Client::new(
            service_bus_namespace.to_owned(),
            event_hub_name.to_owned(),
            policy_name.to_owned(),
            policy_key.to_owned()
        ).unwrap())));
        Self { inner }
    }
}

impl Resource for MqAzureServiceBus {
    fn from_url(_: Url) -> Result<Self> {
        // get environment var AZURE_SERVICE_BUS_NAMESPACE
        let service_bus_namespace = std::env::var("AZURE_SERVICE_BUS_NAMESPACE")?;
        // get environment var AZURE_EVENT_HUB_NAME
        let event_hub_name  = std::env::var("AZURE_EVENT_HUB_NAME")?;
        // get environment var AZURE_POLICY_NAME
        let policy_name = std::env::var("AZURE_POLICY_NAME")?;
        // get environment var AZURE_POLICY_KEY
        let policy_key = std::env::var("AZURE_POLICY_KEY")?;

        Ok(MqAzureServiceBus::new(
            &service_bus_namespace,
            &event_hub_name,
            &policy_name,
            &policy_key,
        ))
    }
}

impl mq::Mq for MqAzureServiceBus {
    type ResourceDescriptor = u64;

    fn get_mq(&mut self) -> Result<Self::ResourceDescriptor, Error> {
        Ok(0)
    }

    fn send(&mut self, rd: &Self::ResourceDescriptor, msg: PayloadParam<'_>) -> Result<(), Error> {
        if *rd != 0 {
            return Err(Error::OtherError);
        }
        block_on(azure::send(&mut self.inner.as_ref().unwrap().lock().unwrap(), std::str::from_utf8(msg)?.to_string()))?;
        Ok(())
    }

    fn receive(&mut self, rd: &Self::ResourceDescriptor) -> Result<PayloadResult, Error> {
        if *rd != 0 {
            return Err(Error::OtherError);
        }

        let result = block_on(azure::receive(&mut self.inner.as_ref().unwrap().lock().unwrap()))?;
        Ok(result)
    }
}

impl<T> ResourceTables<dyn Resource> for MqTables<T> where T: Mq + 'static {}

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