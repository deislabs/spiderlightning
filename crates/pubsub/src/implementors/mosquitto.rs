use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use mosquitto_client::Mosquitto;
use names::{Generator, Name};
use slight_common::BasicState;

#[derive(Clone)]
pub struct MosquittoImplementor {
    mqtt: Mosquitto,
    host: String,
    port: u32,
    subscriptions: Arc<Mutex<Vec<String>>>,
}

impl std::fmt::Debug for MosquittoImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MosquittoImplementor")
    }
}

// Pub+Sub
impl MosquittoImplementor {
    pub fn new(slight_state: &BasicState) -> Self {
        let mqtt = Mosquitto::new(&Generator::with_naming(Name::Numbered).next().unwrap());
        let host = String::from_utf8(
            slight_runtime_configs::get(
                &slight_state.secret_store,
                "MOSQUITTO_HOST",
                &slight_state.config_toml_file_path,
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
                &slight_state.config_toml_file_path,
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
        .parse::<u32>()
        .unwrap();

        Self {
            mqtt,
            host,
            port,
            subscriptions: Arc::new(Mutex::new(Vec::new())),
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

        self.mqtt.connect(&self.host, self.port, 5)?;
        self.mqtt
            .publish(topic, formatted_message_with_key.as_bytes(), 1, false)?;
        Ok(())
    }
}

// Sub
impl MosquittoImplementor {
    pub fn subscribe_to_topic(&self, topic: Vec<String>) -> Result<()> {
        for t in topic {
            self.subscriptions.lock().unwrap().push(t);
        }
        Ok(())
    }

    pub fn poll_for_message(&self, timeout_in_secs: u64) -> Result<String> {
        let timeout_in_millis: i32 = (timeout_in_secs * 100).try_into().unwrap();

        self.mqtt.connect(&self.host, self.port, 5)?;

        let mut all_msgs: Vec<String> = Vec::new();

        for t in self.subscriptions.lock().unwrap().iter() {
            let topic = self.mqtt.subscribe(t, 1)?;
            let mosq_msg = topic.receive_one(timeout_in_millis);
            if let Ok(m) = mosq_msg {
                all_msgs.push(format!("{}-{}", m.topic(), m.text()));
            }
        }

        Ok(format!("{:?}", all_msgs))
    }
}
