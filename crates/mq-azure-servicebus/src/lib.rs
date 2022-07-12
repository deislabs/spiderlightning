use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use azure_messaging_servicebus::prelude::*;
use futures::executor::block_on;
use proc_macro_utils::{Resource, RuntimeResource};
use runtime::resource::{
    get, Ctx, DataT, Event, Linker, Map, Resource, ResourceMap, RuntimeResource,
};

use crossbeam_channel::Sender;
pub use mq::add_to_linker;
use mq::*;
use uuid::Uuid;

pub mod azure;

wit_bindgen_wasmtime::export!("../../wit/mq.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);

const SCHEME_NAME: &str = "azsbusmq";

/// A Azure ServiceBus Message Queue service implementation for the mq interface
#[derive(Default, Clone, Resource, RuntimeResource)]
pub struct MqAzureServiceBus {
    inner: Option<Arc<Mutex<Client>>>,
    host_state: Option<ResourceMap>,
}

impl MqAzureServiceBus {
    /// Create a new `MqAzureServiceBus`
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
            .with_context(|| "failed to connect to Azure Service Bus")
            .unwrap(),
        )));
        Self {
            inner,
            host_state: None,
        }
    }
}

impl mq::Mq for MqAzureServiceBus {
    /// Construct a new `MqAzureServiceBus` instance provided a queue name.
    fn get_mq(&mut self, name: &str) -> Result<ResourceDescriptorResult, Error> {
        let queue_name = name;
        let service_bus_namespace = std::env::var("AZURE_SERVICE_BUS_NAMESPACE")
            .with_context(|| "failed to read AZURE_SERVICE_BUS_NAMESPACE environment variable")?;
        // get environment var AZURE_POLICY_NAME
        let policy_name = std::env::var("AZURE_POLICY_NAME")
            .with_context(|| "failed to read AZURE_POLICY_NAME environment variable")?;
        // get environment var AZURE_POLICY_KEY
        let policy_key = std::env::var("AZURE_POLICY_KEY")
            .with_context(|| "failed to read AZURE_POLICY_KEY environment variable")?;

        let mq_azure_serivcebus = MqAzureServiceBus::new(
            &service_bus_namespace,
            queue_name,
            &policy_name,
            &policy_key,
        );
        self.inner = mq_azure_serivcebus.inner;

        let rd = Uuid::new_v4().to_string();
        let cloned = self.clone();
        let mut map = Map::lock(&mut self.host_state)?;
        map.set(rd.clone(), (Box::new(cloned), None));
        Ok(rd)
    }

    /// Send a message to your service bus' queue
    fn send(&mut self, rd: ResourceDescriptorParam, msg: PayloadParam<'_>) -> Result<(), Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.host_state)?;
        let inner = map.get::<Arc<Mutex<Client>>>(rd)?;

        block_on(azure::send(
            &mut inner.lock().unwrap(),
            std::str::from_utf8(msg)
                .with_context(|| "failed to parse message as UTF-8")?
                .to_string(),
        ))
        .with_context(|| "failed to send message to Azure Service Bus")?;
        Ok(())
    }

    /// Receive the top message from your service bus' queue
    fn receive(&mut self, rd: ResourceDescriptorParam) -> Result<PayloadResult, Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.host_state)?;
        let inner = map.get::<Arc<Mutex<Client>>>(rd)?;

        let result = block_on(azure::receive(&mut inner.lock().unwrap()))
            .with_context(|| "failed to receive message from Azure Service Bus")?;
        Ok(result)
    }
}
