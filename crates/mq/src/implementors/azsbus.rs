use std::borrow::BorrowMut;
use std::sync::Arc;
use tokio::sync::Mutex;

use anyhow::{Context, Result};
use azure_messaging_servicebus::prelude::Client;
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;

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
    pub async fn new(slight_state: &BasicState, name: &str) -> Self {
        let service_bus_namespace = get_from_state("AZURE_SERVICE_BUS_NAMESPACE", slight_state)
            .await
            .unwrap();
        let policy_name = get_from_state("AZURE_POLICY_NAME", slight_state)
            .await
            .unwrap();
        let policy_key = get_from_state("AZURE_POLICY_KEY", slight_state)
            .await
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

    pub async fn send(&self, msg: &[u8]) -> Result<()> {
        let inner = &self.client.as_ref().unwrap();
        azure::send(
            inner.lock().await.borrow_mut(),
            std::str::from_utf8(msg)
                .with_context(|| "failed to parse message as UTF-8")?
                .to_string(),
        )
        .await
        .with_context(|| "failed to send message to Azure Service Bus")?;
        Ok(())
    }

    pub async fn receive(&self) -> Result<Vec<u8>> {
        let inner = &self.client.as_ref().unwrap();
        let result = azure::receive(inner.lock().await.borrow_mut())
            .await
            .with_context(|| "failed to receive message from Azure Service Bus")?;
        Ok(result)
    }
}
