use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use rdkafka::{consumer::StreamConsumer, producer::BaseProducer, ClientConfig};
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;
use tokio::{runtime::Handle, task::block_in_place};

use crate::providers::confluent;

#[derive(Clone)]
pub struct Pub {
    producer: Arc<BaseProducer>,
}

impl std::fmt::Debug for Pub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Apache Kafka's Pub")
    }
}

impl Pub {
    pub async fn new(slight_state: &BasicState) -> Self {
        let akc = ApacheKafkaConfigs::from_state(slight_state).await.unwrap();
        let producer: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", &akc.bootstap_servers)
            .set("security.protocol", &akc.security_protocol)
            .set("sasl.mechanisms", &akc.sasl_mechanisms)
            .set("sasl.username", &akc.sasl_username)
            .set("sasl.password", &akc.sasl_password)
            .create()
            .with_context(|| "failed to create producer client")
            .unwrap(); // panic if we fail to create client

        tracing::debug!("created producer client");

        Self {
            producer: Arc::new(producer),
        }
    }

    pub fn publish(&self, msg_value: &[u8], topic: &str) -> Result<()> {
        tracing::info!("publishing to topic {}", topic);

        confluent::publish(
            &self.producer,
            format!(
                "{:?}",
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
            )
            .as_bytes(), // rand key
            msg_value,
            topic,
        )
        .with_context(|| "failed to send message to a topic")
    }
}

#[derive(Clone)]
pub struct Sub {
    apache_kafka_config: ApacheKafkaConfigs,
    group_id: String,
    consumers: Arc<Mutex<HashMap<String, Arc<StreamConsumer>>>>,
}

impl std::fmt::Debug for Sub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Apache Kafka's Sub")
    }
}

impl Sub {
    pub async fn new(slight_state: &BasicState) -> Self {
        let akc = ApacheKafkaConfigs::from_state(slight_state).await.unwrap();
        let group_id = get_from_state("CAK_GROUP_ID", slight_state).await.unwrap();

        Self {
            apache_kafka_config: akc,
            group_id,
            consumers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn subscribe(&self, topic: &str) -> Result<String> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set(
                "bootstrap.servers",
                &self.apache_kafka_config.bootstap_servers,
            )
            .set(
                "security.protocol",
                &self.apache_kafka_config.security_protocol,
            )
            .set("sasl.mechanisms", &self.apache_kafka_config.sasl_mechanisms)
            .set("sasl.username", &self.apache_kafka_config.sasl_username)
            .set("sasl.password", &self.apache_kafka_config.sasl_password)
            .set("group.id", &self.group_id)
            .create()
            .with_context(|| "failed to create consumer client")
            .unwrap(); // panic if we fail to create client

        confluent::subscribe(&consumer, vec![topic])
            .with_context(|| "failed to subscribe to topic")?;

        // generate uuid for subscription
        let sub_tok = uuid::Uuid::new_v4().to_string();

        self.consumers
            .lock()
            .unwrap()
            .insert(sub_tok.clone(), Arc::new(consumer));

        Ok(sub_tok)
    }

    pub async fn receive(&self, sub_tok: &str) -> Result<Vec<u8>> {
        block_in_place(|| {
            Handle::current().block_on(async move {
                let consumers_lock = self.consumers.lock().unwrap();

                let accessed_consumer = consumers_lock
                    .get(sub_tok)
                    .with_context(|| "failed to get consumer from subscription token")?;

                confluent::receive(&accessed_consumer)
                    .await
                    .with_context(|| "failed to poll for message")
            })
        })
    }
}

/// `ApacheKafkaConfigs` is a convenience structure to avoid the innate
/// repetitiveness of code that comes w/ getting `runtime_configs`.
#[derive(Clone)]
struct ApacheKafkaConfigs {
    bootstap_servers: String,
    security_protocol: String,
    sasl_mechanisms: String,
    sasl_username: String,
    sasl_password: String,
}

impl ApacheKafkaConfigs {
    async fn from_state(slight_state: &BasicState) -> Result<Self> {
        let bootstap_servers = get_from_state("CAK_ENDPOINT", slight_state).await?;
        let security_protocol = get_from_state("CAK_SECURITY_PROTOCOL", slight_state).await?;
        let sasl_mechanisms = get_from_state("CAK_SASL_MECHANISMS", slight_state).await?;
        let sasl_username = get_from_state("CAK_SASL_USERNAME", slight_state).await?;
        let sasl_password = get_from_state("CAK_SASL_PASSWORD", slight_state).await?;

        Ok(Self {
            bootstap_servers,
            security_protocol,
            sasl_mechanisms,
            sasl_username,
            sasl_password,
        })
    }
}
