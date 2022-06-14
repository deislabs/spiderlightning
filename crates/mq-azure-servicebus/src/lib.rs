use std::{
    str::Utf8Error,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use azure_messaging_servicebus::prelude::*;
use futures::executor::block_on;
use runtime::resource::{
    get, Context as RuntimeContext, DataT, HostResource, Linker, Resource, ResourceMap,
};

pub use mq::add_to_linker;
use mq::*;
use uuid::Uuid;

pub mod azure;

wit_bindgen_wasmtime::export!("../../wit/mq.wit");

const SCHEME_NAME: &str = "azmq";

/// A Azure ServiceBus Message Queue binding for the mq interface.
#[derive(Default, Clone)]
pub struct MqAzureServiceBus {
    inner: Option<Arc<Mutex<Client>>>,
    resource_map: Option<ResourceMap>,
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
        Self {
            inner,
            resource_map: None,
        }
    }
}

impl Resource for MqAzureServiceBus {
    fn add_resource_map(&mut self, resource_map: ResourceMap) -> Result<()> {
        self.resource_map = Some(resource_map);
        Ok(())
    }

    fn get_inner(&self) -> &dyn std::any::Any {
        let inner = self.inner.as_ref().unwrap();
        inner
    }
}

impl HostResource for MqAzureServiceBus {
    fn add_to_linker(linker: &mut Linker<RuntimeContext<DataT>>) -> Result<()> {
        crate::add_to_linker(linker, |cx| get::<Self>(cx, SCHEME_NAME.to_string()))
    }

    fn build_data() -> Result<DataT> {
        let mq_azure_servicebus = Self::default();
        Ok(Box::new(mq_azure_servicebus))
    }
}

impl mq::Mq for MqAzureServiceBus {
    /// Get the resource descriptor for your Azure Service Bus message queue
    fn get_mq(&mut self, name: &str) -> Result<ResourceDescriptorResult, Error> {
        let queue_name = name;
        let service_bus_namespace = std::env::var("AZURE_SERVICE_BUS_NAMESPACE")
            .context("AZURE_SERVICE_BUS_NAMESPACE environment variable not found")?;
        // get environment var AZURE_POLICY_NAME
        let policy_name = std::env::var("AZURE_POLICY_NAME")
            .context("AZURE_POLICY_NAME environment variable not found")?;
        // get environment var AZURE_POLICY_KEY
        let policy_key = std::env::var("AZURE_POLICY_KEY")
            .context("AZURE_POLICY_KEY environment variable not found")?;

        let mq_azure_serivcebus = MqAzureServiceBus::new(
            &service_bus_namespace,
            queue_name,
            &policy_name,
            &policy_key,
        );
        self.inner = mq_azure_serivcebus.inner;
        let uuid = Uuid::new_v4();
        let rd = uuid.to_string();
        let cloned = self.clone();
        let mut map = self
            .resource_map
            .as_mut()
            .ok_or(anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        map.set(rd.clone(), Box::new(cloned))?;
        Ok(rd)
    }

    /// Send a message to your service bus' queue
    fn send(&mut self, rd: ResourceDescriptorParam, msg: PayloadParam<'_>) -> Result<(), Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }

        let map = self
            .resource_map
            .as_mut()
            .ok_or(anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        let mut inner = map.get::<Arc<Mutex<Client>>>(rd)?;

        block_on(azure::send(
            &mut inner.lock().unwrap(),
            std::str::from_utf8(msg)?.to_string(),
        ))?;
        Ok(())
    }

    /// Receive the top message from your service bus' queue
    fn receive(&mut self, rd: ResourceDescriptorParam) -> Result<PayloadResult, Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }

        let map = self
            .resource_map
            .as_mut()
            .ok_or(anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        let mut inner = map.get::<Arc<Mutex<Client>>>(rd)?;

        let result = block_on(azure::receive(&mut inner.lock().unwrap()))?;
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
