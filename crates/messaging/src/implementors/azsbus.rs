use async_trait::async_trait;
use azure_messaging_servicebus::service_bus::SubscriptionReceiver;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;
use tokio::task::block_in_place;

use anyhow::{Context, Result};
use azure_messaging_servicebus::prelude::TopicClient;
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;

use super::{PubImplementor, SubImplementor};

#[derive(Clone)]
pub struct AzSbusImplementor {
    service_bus_namespace: String,
    policy_name: String,
    policy_key: String,
    http_client: Arc<dyn azure_core::HttpClient>,
    subscription_tokens: Arc<Mutex<HashMap<String, SubscriptionReceiver>>>,
}

impl std::fmt::Debug for AzSbusImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AzSbusImplementor")
    }
}

impl AzSbusImplementor {
    pub async fn new(slight_state: &BasicState) -> Self {
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

        Self {
            service_bus_namespace,
            policy_name,
            policy_key,
            http_client,
            subscription_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn make_topic_client(&self, topic: &str) -> TopicClient {
        TopicClient::new(
            self.http_client.clone(),
            &self.service_bus_namespace,
            topic,
            &self.policy_name,
            &self.policy_key,
        )
        .unwrap()
    }
}

#[async_trait]
impl PubImplementor for AzSbusImplementor {
    async fn publish(&self, msg: &[u8], topic: &str) -> Result<()> {
        let topic_client = self.make_topic_client(topic);

        topic_client
            .topic_sender()
            .send_message(std::str::from_utf8(msg).unwrap())
            .await?;

        Ok(())
    }
}

#[async_trait]
impl SubImplementor for AzSbusImplementor {
    async fn subscribe(&self, topic: &str) -> Result<String> {
        let sub_tok = uuid::Uuid::new_v4().to_string();

        let topic_client = self.make_topic_client(topic);

        let receiver = topic_client.subscription_receiver(topic);

        self.subscription_tokens
            .lock()
            .unwrap()
            .insert(sub_tok.clone(), receiver);

        Ok(sub_tok)
    }

    async fn receive(&self, sub_tok: &str) -> Result<Vec<u8>> {
        block_in_place(|| {
            Handle::current().block_on(async move {
                let sub_toks = self.subscription_tokens.lock().unwrap();

                let accessed_consumer = sub_toks
                    .get(sub_tok)
                    .with_context(|| "failed to get consumer from subscription token")?;

                let msg = accessed_consumer.receive_and_delete_message().await?;

                Ok(msg.as_bytes().to_vec())
            })
        })
    }
}
