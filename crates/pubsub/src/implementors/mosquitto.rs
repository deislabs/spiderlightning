use std::sync::{Arc, Mutex};

use anyhow::Result;
use mosquitto_rs::{Client, QoS};
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;
use tokio::{runtime::Handle, task::block_in_place};

#[derive(Clone)]
pub struct MosquittoImplementor {
    host: String,
    port: i32,
    subscriptions: Arc<Mutex<Vec<String>>>,
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
        Self {
            host,
            port,
            subscriptions: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

// Pub
impl MosquittoImplementor {
    pub async fn publish(&self, msg_value: &[u8], topic: &str) -> Result<()> {
        let mut mqtt = Client::with_auto_id().unwrap();
        block_in_place(|| {
            Handle::current().block_on(async move {
                mqtt.connect(
                    &self.host,
                    self.port,
                    std::time::Duration::from_secs(5),
                    None,
                )
                .await
                .unwrap();

                mqtt.publish(topic, msg_value, QoS::AtMostOnce, false)
                    .await
                    .unwrap()
            })
        });

        Ok(())
    }
}

// Sub
impl MosquittoImplementor {
    pub fn subscribe(&self, topic: &str) -> Result<()> {
        self.subscriptions.lock().unwrap().push(topic.to_string());
        Ok(())
    }

    pub async fn receive(&self) -> Result<Vec<u8>> {
        let mut mqtt = Client::with_auto_id().unwrap();
        let mut res: Vec<u8> = vec![];
        block_in_place(|| {
            res = Handle::current().block_on(async move {
                mqtt.connect(
                    &self.host,
                    self.port,
                    std::time::Duration::from_secs(5),
                    None,
                )
                .await
                .unwrap();

                let subs_lock = self.subscriptions.lock().unwrap();

                for t in subs_lock.iter() {
                    mqtt.subscribe(t, QoS::AtMostOnce).await.unwrap();
                }

                mqtt.subscriber()
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
