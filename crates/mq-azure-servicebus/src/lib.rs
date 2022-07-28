use anyhow::{Context, Result};
use azure_messaging_servicebus::prelude::*;
use events_api::Event;
use futures::executor::block_on;
use proc_macro_utils::{Resource, Watch};
use runtime::{
    impl_resource,
    resource::{
        BasicState,
        Watch,
    },
};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crossbeam_channel::Sender;
pub use mq::add_to_linker;
use mq::*;
use uuid::Uuid;

pub mod azure;

wit_bindgen_wasmtime::export!("../../wit/mq.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);
wit_error_rs::impl_from!(std::string::FromUtf8Error, Error::ErrorWithDescription);

const SCHEME_NAME: &str = "mq.azsbus";

/// A Azure ServiceBus Message Queue service implementation for the mq interface
#[derive(Default, Clone, Resource)]
pub struct MqAzureServiceBus {
    host_state: BasicState,
}

impl_resource!(
    MqAzureServiceBus,
    mq::MqTables<MqAzureServiceBus>,
    BasicState,
    SCHEME_NAME.to_string()
);

#[derive(Default, Clone, Watch)]
pub struct MqAzureServiceBusInner {
    // FIXME: file an issue to azure-sdk-for-rust to impl Debug for this
    client: Option<Arc<Mutex<Client>>>,
}

impl Debug for MqAzureServiceBusInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MqAzureServiceBusInner")
    }
}

impl MqAzureServiceBusInner {
    /// Create a new `MqAzureServiceBus`
    pub fn new(
        service_bus_namespace: &str,
        queue_name: &str,
        policy_name: &str,
        policy_key: &str,
    ) -> Self {
        let http_client = azure_core::new_http_client();

        let client = Some(Arc::new(Mutex::new(
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
        Self { client }
    }
}

impl mq::Mq for MqAzureServiceBus {
    type Mq = MqAzureServiceBusInner;
    /// Construct a new `MqAzureServiceBus` instance provided a queue name.
    fn mq_open(&mut self, name: &str) -> Result<Self::Mq, Error> {
        let queue_name = name;
        let service_bus_namespace = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "AZURE_SERVICE_BUS_NAMESPACE",
            &self.host_state.config_toml_file_path,
        )?)?;
        let policy_name = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "AZURE_POLICY_NAME",
            &self.host_state.config_toml_file_path,
        )?)?;
        let policy_key = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "AZURE_POLICY_KEY",
            &self.host_state.config_toml_file_path,
        )?)?;

        let mq_azure_serivcebus_guest = Self::Mq::new(
            &service_bus_namespace,
            queue_name,
            &policy_name,
            &policy_key,
        );

        let rd = Uuid::new_v4().to_string();
        self.host_state
            .resource_map
            .lock()
            .unwrap()
            .set(rd, Box::new(mq_azure_serivcebus_guest.clone()));
        Ok(mq_azure_serivcebus_guest)
    }

    /// Send a message to your service bus' queue
    fn mq_send(&mut self, self_: &Self::Mq, msg: PayloadParam<'_>) -> Result<(), Error> {
        let inner = self_.client.as_ref().unwrap();
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
    fn mq_receive(&mut self, self_: &Self::Mq) -> Result<PayloadResult, Error> {
        let inner = self_.client.as_ref().unwrap();
        let result = block_on(azure::receive(&mut inner.lock().unwrap()))
            .with_context(|| "failed to receive message from Azure Service Bus")?;
        Ok(result)
    }
}
