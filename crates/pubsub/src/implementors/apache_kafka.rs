use std::sync::Arc;

use anyhow::{Context, Result};
use rdkafka::{consumer::BaseConsumer, producer::BaseProducer, ClientConfig};
use slight_runtime::resource::BasicState;

use crate::providers::confluent::{self, KafkaMessage};

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
    pub fn new(slight_state: &BasicState) -> Self {
        let akc = ApacheKafkaConfigs::from_state(slight_state).unwrap();
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

    pub fn send_message_to_topic(
        &self,
        msg_key: &[u8],
        msg_value: &[u8],
        topic: &str,
    ) -> Result<()> {
        confluent::send(&self.producer, msg_key, msg_value, topic)
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
    consumer: Arc<BaseConsumer>,
}

impl std::fmt::Debug for SubConfluentApacheKafkaImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SubConfluentApacheKafkaImplementor")
    }
}

impl SubConfluentApacheKafkaImplementor {
    pub fn new(slight_state: &BasicState) -> Self {
        let akc = ApacheKafkaConfigs::from_state(slight_state).unwrap();
        let group_id = String::from_utf8(
            slight_runtime_configs::get(
                &slight_state.secret_store,
                "CK_GROUP_ID",
                &slight_state.config_toml_file_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'CK_GROUP_ID' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();

        let consumer: BaseConsumer = ClientConfig::new()
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

    pub fn subscribe_to_topic(&self, topic: Vec<&str>) -> Result<()> {
        confluent::subscribe(&self.consumer, topic).with_context(|| "failed to subscribe to topic")
    }

    pub fn poll_for_message(&self, timeout_in_secs: u64) -> Result<KafkaMessage> {
        confluent::poll(&self.consumer, timeout_in_secs)
            .with_context(|| "failed to poll for message")
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

fn get_config(config_name: &str, state: &BasicState) -> Result<String> {
    let config = String::from_utf8(
        slight_runtime_configs::get(
            &state.secret_store,
            config_name,
            &state.config_toml_file_path,
        )
        .with_context(|| {
            format!(
                "failed to get '{}' secret using secret store type: {}",
                config_name, state.secret_store
            )
        })?,
    )?;
    Ok(config)
}

impl ApacheKafkaConfigs {
    fn from_state(slight_state: &BasicState) -> Result<Self> {
        let bootstap_servers = get_config("CAK_ENDPOINT", slight_state)?;
        let security_protocol = get_config("CAK_SECURITY_PROTOCOL", slight_state)?;
        let sasl_mechanisms = get_config("CAK_SASL_MECHANISMS", slight_state)?;
        let sasl_username = get_config("CAK_SASL_USERNAME", slight_state)?;

        let sasl_password = get_config("CAK_SASL_PASSWORD", slight_state)?;

        Ok(Self {
            bootstap_servers,
            security_protocol,
            sasl_mechanisms,
            sasl_username,
            sasl_password,
        })
    }
}
