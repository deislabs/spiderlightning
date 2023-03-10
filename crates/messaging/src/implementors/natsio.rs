use async_trait::async_trait;
use nats::{Connection, Subscription};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;
use tokio::task::block_in_place;

use anyhow::{Context, Result};
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;

use super::{PubImplementor, SubImplementor};

#[derive(Clone, Debug)]
pub struct NatsIoImplementor {
    connection: Connection,
    subscription_tokens: Arc<Mutex<HashMap<String, Subscription>>>,
}

impl NatsIoImplementor {
    pub async fn new(slight_state: &BasicState) -> Self {
        let nats_creds_path = get_from_state("NATS_CREDS_PATH", slight_state)
            .await
            .unwrap();

        let connection = nats::Options::with_credentials(&nats_creds_path).connect("connect.ngs.global").unwrap();
        let subscription_tokens = Arc::new(Mutex::new(HashMap::new()));

        Self {
            connection,
            subscription_tokens
        }
    }
}

#[async_trait]
impl PubImplementor for NatsIoImplementor {
    async fn publish(&self, msg: &[u8], topic: &str) -> Result<()> {
        self.connection.publish(topic, msg).unwrap();
        Ok(())
    }
}

#[async_trait]
impl SubImplementor for NatsIoImplementor {
    async fn subscribe(&self, topic: &str) -> Result<String> {
        let sub = self.connection.subscribe(topic).unwrap();

        let sub_tok = uuid::Uuid::new_v4().to_string();
        self.subscription_tokens
            .lock()
            .unwrap()
            .insert(sub_tok.clone(), sub);

        Ok(sub_tok)
    }

    async fn receive(&self, sub_tok: &str) -> Result<Vec<u8>> {
        block_in_place(|| {
            Handle::current().block_on(async move {
                let sub_toks = self.subscription_tokens.lock().unwrap();

                let accessed_consumer = sub_toks
                    .get(sub_tok)
                    .with_context(|| "failed to get consumer from subscription token")?;

                let msg = accessed_consumer.next().unwrap();

                Ok(msg.data)
            })
        })
    }
}
