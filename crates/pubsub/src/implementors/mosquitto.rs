use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_channel::Receiver;
use futures::executor::block_on;
use mosquitto_rs::{Client, Message, QoS};
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;

#[derive(Clone)]
pub struct MosquittoImplementor {
    mqtt: Arc<Mutex<Client>>,
    host: String,
    port: i32,
    subscriptions: Arc<Mutex<Vec<String>>>,
    subscriber: Arc<Mutex<Option<Receiver<Message>>>>,
}

// TODO: We need to improve these Debug implementations
impl std::fmt::Debug for MosquittoImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MosquittoImplementor")
    }
}

// Pub+Sub
impl MosquittoImplementor {
    pub fn new(slight_state: &BasicState) -> Self {
        let mqtt = Client::with_auto_id().unwrap();
        let host = block_on(get_from_state("MOSQUITTO_HOST", slight_state)).unwrap();
        let port = block_on(get_from_state("MOSQUITTO_PORT", slight_state))
            .unwrap()
            .parse::<i32>()
            .unwrap();

        Self {
            mqtt: Arc::new(Mutex::new(mqtt)),
            host,
            port,
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            subscriber: Arc::new(Mutex::new(None)),
        }
    }
}

// Pub
impl MosquittoImplementor {
    pub fn publish(&self, msg_value: &[u8], topic: &str) -> Result<()> {
        block_on(self.mqtt.lock().as_mut().unwrap().connect(
            &self.host,
            self.port,
            std::time::Duration::from_secs(5),
            None,
        ))?;

        block_on(self.mqtt.lock().as_mut().unwrap().publish(
            topic,
            msg_value,
            QoS::AtMostOnce,
            false,
        ))?;
        Ok(())
    }
}

// Sub
impl MosquittoImplementor {
    pub fn subscribe(&self, topic: &str) -> Result<()> {
        self.subscriptions.lock().unwrap().push(topic.to_string());
        *self.subscriber.lock().unwrap() = self.mqtt.lock().as_mut().unwrap().subscriber();
        Ok(())
    }

    pub fn receive(&self) -> Result<Vec<u8>> {
        block_on(self.mqtt.lock().as_mut().unwrap().connect(
            &self.host,
            self.port,
            std::time::Duration::from_secs(5),
            None,
        ))?;

        for t in self.subscriptions.lock().unwrap().iter() {
            block_on(
                self.mqtt
                    .lock()
                    .as_mut()
                    .unwrap()
                    .subscribe(t, QoS::AtMostOnce),
            )?;
        }

        Ok(block_on(
            self.subscriber
                .lock()
                .as_mut()
                .unwrap()
                .as_mut()
                .unwrap()
                .recv(),
        )?
        .payload)
    }
}
