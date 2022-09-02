use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use async_channel::Receiver;
use futures::executor::block_on;
use mosquitto_rs::{Client, Message, QoS};
use slight_common::BasicState;

#[derive(Clone)]
pub struct MosquittoImplementor {
    mqtt: Arc<Mutex<Client>>,
    host: String,
    port: i32,
    subscriptions: Arc<Mutex<Vec<String>>>,
    subscriber: Arc<Mutex<Option<Receiver<Message>>>>,
}

impl std::fmt::Debug for MosquittoImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MosquittoImplementor")
    }
}

// Pub+Sub
impl MosquittoImplementor {
    pub fn new(slight_state: &BasicState) -> Self {
        let mqtt = Client::with_auto_id().unwrap();
        let host = String::from_utf8(
            slight_runtime_configs::get(
                &slight_state.secret_store,
                "MOSQUITTO_HOST",
                &slight_state.slightfile_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'MOSQUITTO_HOST' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();

        let port = String::from_utf8(
            slight_runtime_configs::get(
                &slight_state.secret_store,
                "MOSQUITTO_PORT",
                &slight_state.slightfile_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'MOSQUITTO_PORT' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
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
        // ^^^ arbitrarily chosing to create with 5 threads
        // (threads run notification functions)
    }
}

// Pub
impl MosquittoImplementor {
    pub fn send_message_to_topic(
        &self,
        msg_key: &[u8],
        msg_value: &[u8],
        topic: &str,
    ) -> Result<()> {
        let formatted_message_with_key = &format!(
            "{}-{}",
            std::str::from_utf8(msg_key)?,
            std::str::from_utf8(msg_value)?
        );
        // ^^^ arbitrarily formatting msg key and value like
        // (as we have more implementors for pubsub, we should consider if we even
        // want a key in the pubsub implementation)

        block_on(self.mqtt.lock().as_mut().unwrap().connect(
            &self.host,
            self.port,
            std::time::Duration::from_secs(5),
            None,
        ))?;

        block_on(self.mqtt.lock().as_mut().unwrap().publish(
            topic,
            formatted_message_with_key.as_bytes(),
            QoS::AtMostOnce,
            false,
        ))?;
        Ok(())
    }
}

// Sub
impl MosquittoImplementor {
    pub fn subscribe_to_topic(&self, topic: Vec<String>) -> Result<()> {
        for t in topic {
            self.subscriptions.lock().unwrap().push(t);
        }

        *self.subscriber.lock().unwrap() = self.mqtt.lock().as_mut().unwrap().subscriber();
        Ok(())
    }

    pub fn poll_for_message(&self, _: u64) -> Result<String> {
        // ^^^ timeout unused here, this probably hints it's not something we want in the
        // overall interface
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

        let msg = format!(
            "{:?}",
            String::from_utf8(
                block_on(
                    self.subscriber
                        .lock()
                        .as_mut()
                        .unwrap()
                        .as_mut()
                        .unwrap()
                        .recv()
                )?
                .payload
            )
        );

        Ok(msg)
    }
}
