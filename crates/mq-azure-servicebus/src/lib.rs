use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use azure_messaging_servicebus::prelude::*;
use events_api::Event;
use futures::executor::block_on;
use proc_macro_utils::Resource;
use runtime::{
    impl_resource,
    resource::{
        get_table, BasicState, Ctx, DataT, Linker, Map, Resource, ResourceTables, RuntimeResource,
    },
};

use crossbeam_channel::Sender;
pub use mq::add_to_linker;
use mq::*;
use uuid::Uuid;

pub mod azure;

wit_bindgen_wasmtime::export!("../../wit/mq.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);
wit_error_rs::impl_from!(std::string::FromUtf8Error, Error::ErrorWithDescription);

const SCHEME_NAME: &str = "azsbusmq";

/// A Azure ServiceBus Message Queue service implementation for the mq interface
#[derive(Default, Clone, Resource)]
pub struct MqAzureServiceBus {
    inner: Option<Arc<Mutex<Client>>>,
    host_state: Option<BasicState>,
}

impl_resource!(
    MqAzureServiceBus,
    mq::MqTables<MqAzureServiceBus>,
    BasicState,
    SCHEME_NAME.to_string()
);

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
    type Mq = String;
    /// Construct a new `MqAzureServiceBus` instance provided a queue name.
    fn mq_open(&mut self, name: &str) -> Result<Self::Mq, Error> {
        let queue_name = name;
        let service_bus_namespace = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.as_ref().unwrap().secret_store,
            "AZURE_SERVICE_BUS_NAMESPACE",
            &self.host_state.as_ref().unwrap().config_toml_file_path,
        )?)?;
        let policy_name = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.as_ref().unwrap().secret_store,
            "AZURE_POLICY_NAME",
            &self.host_state.as_ref().unwrap().config_toml_file_path,
        )?)?;
        let policy_key = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.as_ref().unwrap().secret_store,
            "AZURE_POLICY_KEY",
            &self.host_state.as_ref().unwrap().config_toml_file_path,
        )?)?;

        let mq_azure_serivcebus = MqAzureServiceBus::new(
            &service_bus_namespace,
            queue_name,
            &policy_name,
            &policy_key,
        );
        self.inner = mq_azure_serivcebus.inner;

        let rd = Uuid::new_v4().to_string();
        let cloned = self.clone();
        let mut map = Map::lock(&mut self.host_state.as_mut().unwrap().resource_map)?;
        map.set(rd.clone(), (Box::new(cloned), None));
        Ok(rd)
    }

    /// Send a message to your service bus' queue
    fn mq_send(&mut self, self_: &Self::Mq, msg: PayloadParam<'_>) -> Result<(), Error> {
        Uuid::parse_str(self_)
            .with_context(|| "internal error: failed to parse internal handle to this resource")?;

        let map = Map::lock(&mut self.host_state.as_mut().unwrap().resource_map)?;
        let inner = map.get::<Arc<Mutex<Client>>>(self_)?;

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
        Uuid::parse_str(self_)
            .with_context(|| "internal error: failed to parse internal handle to this resource")?;

        let map = Map::lock(&mut self.host_state.as_mut().unwrap().resource_map)?;
        let inner = map.get::<Arc<Mutex<Client>>>(self_)?;

        let result = block_on(azure::receive(&mut inner.lock().unwrap()))
            .with_context(|| "failed to receive message from Azure Service Bus")?;
        Ok(result)
    }
}
