use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use async_channel::Receiver;
use mosquitto_rs::{Client, Message, QoS};
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;
use tokio::{runtime::Handle, task::block_in_place};

#[derive(Clone)]
pub struct Pub {
    producer: Arc<Mutex<Client>>,
}

impl std::fmt::Debug for Pub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mosquitto's Pub")
    }
}

#[derive(Clone)]
pub struct Sub {
    host: String,
    port: i32,
    consumers: Arc<Mutex<HashMap<String, Consumer>>>,
}

#[derive(Clone)]
pub struct Consumer {
    _client: Arc<Mutex<Client>>,
    subscriptions: Arc<Mutex<Option<Receiver<Message>>>>,
}

impl std::fmt::Debug for Sub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mosquitto's Sub")
    }
}

impl Pub {
    pub async fn new(slight_state: &BasicState) -> Self {
        let host = get_from_state("MOSQUITTO_HOST", slight_state)
            .await
            .unwrap();
        let port = get_from_state("MOSQUITTO_PORT", slight_state)
            .await
            .unwrap()
            .parse::<i32>()
            .unwrap();

        tracing::debug!("Connecting to Mosquitto broker at {}:{}", host, port);

        let producer = block_in_place(|| {
            Handle::current().block_on(async move {
                let mut client = Client::with_auto_id().unwrap();

                client
                    .connect(&host, port, std::time::Duration::from_secs(5), None)
                    .await
                    .unwrap();

                Arc::new(Mutex::new(client))
            })
        });

        Self { producer }
    }

    pub async fn publish(&self, msg_value: &[u8], topic: &str) -> Result<()> {
        block_in_place(|| {
            Handle::current().block_on(async move {
                self.producer
                    .lock()
                    .unwrap()
                    .publish(topic, msg_value, QoS::AtMostOnce, false)
                    .await
                    .unwrap()
            })
        });

        Ok(())
    }
}

impl Sub {
    pub async fn new(slight_state: &BasicState) -> Self {
        let host = get_from_state("MOSQUITTO_HOST", slight_state)
            .await
            .unwrap();
        let port = get_from_state("MOSQUITTO_PORT", slight_state)
            .await
            .unwrap()
            .parse::<i32>()
            .unwrap();

        tracing::info!("Connecting to Mosquitto broker at {}:{}", host, port);

        Self {
            host,
            port,
            consumers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn subscribe(&self, topic: &str) -> Result<String> {
        let new_consumer = block_in_place(|| {
            Handle::current().block_on(async move {
                let mut client = Client::with_auto_id().unwrap();

                client
                    .connect(
                        &self.host.clone(),
                        self.port,
                        std::time::Duration::from_secs(5),
                        None,
                    )
                    .await
                    .unwrap();

                client.subscribe(topic, QoS::AtLeastOnce).await.unwrap();

                let subscriber = client.subscriber();

                Consumer {
                    _client: Arc::new(Mutex::new(client)),
                    subscriptions: Arc::new(Mutex::new(subscriber)),
                }
            })
        });

        // generate uuid
        let k = uuid::Uuid::new_v4().to_string();

        self.consumers
            .lock()
            .unwrap()
            .insert(k.clone(), new_consumer);

        Ok(k)
    }

    pub async fn receive(&self, sub_tok: &str) -> Result<Vec<u8>> {
        let mut res: Vec<u8> = vec![];

        block_in_place(|| {
            res = Handle::current().block_on(async move {
                self.consumers
                    .lock()
                    .unwrap()
                    .get(sub_tok)
                    .with_context(|| "failed to get consumer from subscription token")
                    .unwrap()
                    .subscriptions
                    .lock()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .recv()
                    .await
                    .unwrap()
                    .payload
            })
        });

        Ok(res)
    }
}
