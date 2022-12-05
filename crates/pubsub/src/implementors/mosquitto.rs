use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_channel::Receiver;
use mosquitto_rs::{Client, QoS, Message};
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
    consumer: Arc<Mutex<Client>>,
    subscriptions: Arc<Mutex<Option<Receiver<Message>>>>,
}

impl std::fmt::Debug for Sub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mosquitto's Sub")
    }
}

// Pub
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

        let ( consumer, subscriptions ) = block_in_place(|| {
            Handle::current().block_on(async move {
                let mut client = Client::with_auto_id().unwrap();

                client
                    .connect(&host, port, std::time::Duration::from_secs(5), None)
                    .await
                    .unwrap();

                let ret0 = Arc::new(Mutex::new(client));
                let ret1 = Arc::new(Mutex::new(None));

                (ret0, ret1)
            })
        });

        Self { consumer, subscriptions }
    }

    pub async fn subscribe(&self, topic: &str) -> Result<()> {
        block_in_place(|| {
            Handle::current().block_on(async move {
                self.consumer
                    .lock()
                    .unwrap()
                    .subscribe(topic, QoS::AtMostOnce)
                    .await
                    .unwrap();

                tracing::info!("Subscribed to topic {}", topic);
            })
        });

        *self.subscriptions.lock().unwrap() = self.consumer.lock().unwrap().subscriber();

        Ok(())
    }

    pub async fn receive(&self) -> Result<Vec<u8>> {
        let mut res: Vec<u8> = vec![];

        block_in_place(|| {
            res = Handle::current().block_on(async move {
                self.subscriptions
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
