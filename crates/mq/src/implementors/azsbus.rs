use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use azure_messaging_servicebus::prelude::Client;
use futures::executor::block_on;
use slight_common::BasicState;

use crate::providers::azure;

#[derive(Clone)]
pub struct AzSbusImplementor {
    client: Option<Arc<Mutex<Client>>>,
}

impl std::fmt::Debug for AzSbusImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AzSbusImplementor")
    }
}

impl AzSbusImplementor {
    pub fn new(slight_state: &BasicState, name: &str) -> Self {
        let service_bus_namespace = String::from_utf8(
            slight_runtime_configs::get(
                &slight_state.secret_store,
                "AZURE_SERVICE_BUS_NAMESPACE",
                &slight_state.slightfile_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'AZURE_SERVICE_BUS_NAMESPACE' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();
        let policy_name = String::from_utf8(
            slight_runtime_configs::get(
                &slight_state.secret_store,
                "AZURE_POLICY_NAME",
                &slight_state.slightfile_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'AZURE_POLICY_NAME' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();
        let policy_key = String::from_utf8(
            slight_runtime_configs::get(
                &slight_state.secret_store,
                "AZURE_POLICY_KEY",
                &slight_state.slightfile_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'AZURE_POLICY_KEY' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();

        let http_client = azure_core::new_http_client();

        let client = Some(Arc::new(Mutex::new(
            Client::new(
                http_client,
                service_bus_namespace,
                name.to_owned(),
                policy_name,
                policy_key,
            )
            .with_context(|| "failed to connect to Azure Service Bus")
            .unwrap(),
        )));
        Self { client }
    }

    pub fn send(&self, msg: &[u8]) -> Result<()> {
        let inner = &self.client.as_ref().unwrap();
        block_on(azure::send(
            &mut inner.lock().unwrap(),
            std::str::from_utf8(msg)
                .with_context(|| "failed to parse message as UTF-8")?
                .to_string(),
        ))
        .with_context(|| "failed to send message to Azure Service Bus")?;
        Ok(())
    }

    pub fn receive(&self) -> Result<Vec<u8>> {
        let inner = &self.client.as_ref().unwrap();
        let result = block_on(azure::receive(&mut inner.lock().unwrap()))
            .with_context(|| "failed to receive message from Azure Service Bus")?;
        Ok(result)
    }
}
