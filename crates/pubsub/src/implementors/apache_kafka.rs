use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use rdkafka::{consumer::StreamConsumer, producer::BaseProducer, ClientConfig};
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;

use crate::providers::confluent;

/// This is one of the underlying structs behind the `ConfluentApacheKafka` variant of the `PubImplementor` enum.
///
/// It provides a property that pertains solely to Confluent's Apache Kafka's implementation
/// of this capability:
///     - `producer`
///
/// As per its' usage in `PubImplementor`, it must `derive` `Debug`, and `Clone`.
#[derive(Clone)]
pub struct PubConfluentApacheKafkaImplementor {
    producer: Arc<BaseProducer>,
}

impl std::fmt::Debug for PubConfluentApacheKafkaImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PubConfluentApacheKafkaImplementor")
    }
}

impl PubConfluentApacheKafkaImplementor {
    pub async fn new(slight_state: &BasicState) -> Self {
        let akc = ApacheKafkaConfigs::from_state(slight_state).await.unwrap();
        let producer: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", akc.bootstap_servers)
            .set("security.protocol", akc.security_protocol)
            .set("sasl.mechanisms", akc.sasl_mechanisms)
            .set("sasl.username", akc.sasl_username)
            .set("sasl.password", akc.sasl_password)
            .create()
            .with_context(|| "failed to create producer client")
            .unwrap(); // panic if we fail to create client

        Self {
            producer: Arc::new(producer),
        }
    }

    pub fn publish(&self, msg_value: &[u8], topic: &str) -> Result<()> {
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

/// This is one of the underlying structs behind the `ConfluentApacheKafka` variant of the `SubImplementor` enum.
///
/// It provides a property that pertains solely to Confluent's Apache Kafka's implementation
/// of this capability:
///     - `consumer`
///
/// As per its' usage in `SubImplementor`, it must `derive` `std::fmt::Debug`, and `Clone`.
#[derive(Clone)]
pub struct SubConfluentApacheKafkaImplementor {
    consumer: Arc<StreamConsumer>,
}

impl std::fmt::Debug for SubConfluentApacheKafkaImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SubConfluentApacheKafkaImplementor")
    }
}

impl SubConfluentApacheKafkaImplementor {
    pub async fn new(slight_state: &BasicState) -> Self {
        let akc = ApacheKafkaConfigs::from_state(slight_state).await.unwrap();
        let group_id = get_from_state("CAK_GROUP_ID", slight_state).await.unwrap();

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", akc.bootstap_servers)
            .set("security.protocol", akc.security_protocol)
            .set("sasl.mechanisms", akc.sasl_mechanisms)
            .set("sasl.username", akc.sasl_username)
            .set("sasl.password", akc.sasl_password)
            .set("group.id", group_id)
            .create()
            .with_context(|| "failed to create consumer client")
            .unwrap(); // panic if we fail to create client

        Self {
            consumer: Arc::new(consumer),
        }
    }

    pub fn subscribe(&self, topic: &str) -> Result<()> {
        confluent::subscribe(&self.consumer, vec![topic])
            .with_context(|| "failed to subscribe to topic")
    }

    pub async fn receive(&self) -> Result<Vec<u8>> {
        confluent::receive(&self.consumer).await.with_context(|| "failed to poll for message")
    }
}

/// `ApacheKafkaConfigs` is a convenience structure to avoid the innate
/// repetitiveness of code that comes w/ getting `runtime_configs`.
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
