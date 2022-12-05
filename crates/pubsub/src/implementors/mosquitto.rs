use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_channel::Receiver;
use mosquitto_rs::{Client, QoS, Message};
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;
use tokio::{runtime::Handle, task::block_in_place};

#[derive(Clone)]
pub struct MosquittoImplementor {
    client: Arc<Mutex<Client>>,
    subscriptions: Arc<Mutex<Option<Receiver<Message>>>>,
}

// TODO: We need to improve these Debug implementations
impl std::fmt::Debug for MosquittoImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MosquittoImplementor")
    }
}

// Pub+Sub
impl MosquittoImplementor {
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

        let ( client, subscriptions ) = block_in_place(|| {
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

        Self { client, subscriptions }
    }
}

// Pub
impl MosquittoImplementor {
    pub async fn publish(&self, msg_value: &[u8], topic: &str) -> Result<()> {
        block_in_place(|| {
            Handle::current().block_on(async move {
                self.client
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

// Sub
impl MosquittoImplementor {
    pub async fn subscribe(&self, topic: &str) -> Result<()> {
        block_in_place(|| {
            Handle::current().block_on(async move {
                self.client
                    .lock()
                    .unwrap()
                    .subscribe(topic, QoS::AtMostOnce)
                    .await
                    .unwrap();

                tracing::info!("Subscribed to topic {}", topic);
            })
        });

        *self.subscriptions.lock().unwrap() = self.client.lock().unwrap().subscriber();

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
