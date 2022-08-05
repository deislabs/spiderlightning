use std::sync::Arc;

use anyhow::{Context, Result};
use rdkafka::{consumer::BaseConsumer, producer::BaseProducer, ClientConfig};
use runtime::resource::{BasicState, Watch};

use crate::providers::confluent::{self, KafkaMessage};

/// This is one of the underlying structs behind the `ConfluentApacheKafka` variant of the `PubsubImplementor` enum.
///
/// It provides a property that pertains solely to Confluent's Apache Kafka's implementation
/// of this capability:
///     - `producer`
///
/// As per its' usage in `PubsubImplementor`, it must `derive` `Debug`, and `Clone`.
#[derive(Clone)]
pub struct PubConfluentApacheKafkaImplementor {
    producer: Arc<BaseProducer>,
}

impl Watch for PubConfluentApacheKafkaImplementor {}

impl std::fmt::Debug for PubConfluentApacheKafkaImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PubConfluentApacheKafkaImplementor")
    }
}

impl PubConfluentApacheKafkaImplementor {
    pub fn new(slight_state: &BasicState) -> Self {
        let bootstap_servers = String::from_utf8(
            runtime_configs::providers::get(
                &slight_state.secret_store,
                "CK_ENDPOINT",
                &slight_state.config_toml_file_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'CK_ENDPOINT' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();
        let security_protocol = String::from_utf8(
            runtime_configs::providers::get(
                &slight_state.secret_store,
                "CK_SECURITY_PROTOCOL",
                &slight_state.config_toml_file_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'CK_SECURITY_PROTOCOL' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();
        let sasl_mechanisms = String::from_utf8(
            runtime_configs::providers::get(
                &slight_state.secret_store,
                "CK_SASL_MECHANISMS",
                &slight_state.config_toml_file_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'CK_SASL_MECHANISMS' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();
        let sasl_username = String::from_utf8(
            runtime_configs::providers::get(
                &slight_state.secret_store,
                "CK_SASL_USERNAME",
                &slight_state.config_toml_file_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'CK_SASL_USERNAME' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();

        let sasl_password = String::from_utf8(
            runtime_configs::providers::get(
                &slight_state.secret_store,
                "CK_SASL_PASSWORD",
                &slight_state.config_toml_file_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'CK_SASL_PASSWORD' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();
        let producer: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", bootstap_servers)
            .set("security.protocol", security_protocol)
            .set("sasl.mechanisms", sasl_mechanisms)
            .set("sasl.username", sasl_username)
            .set("sasl.password", sasl_password)
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

/// This is one of the underlying structs behind the `ConfluentApacheKafka` variant of the `PubsubImplementor` enum.
///
/// It provides a property that pertains solely to Confluent's Apache Kafka's implementation
/// of this capability:
///     - `consumer`
///
/// As per its' usage in `PubsubImplementor`, it must `derive` `std::fmt::Debug`, and `Clone`.
#[derive(Clone)]
pub struct SubConfluentApacheKafkaImplementor {
    consumer: Arc<BaseConsumer>,
}

impl Watch for SubConfluentApacheKafkaImplementor {}

impl std::fmt::Debug for SubConfluentApacheKafkaImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SubConfluentApacheKafkaImplementor")
    }
}

impl SubConfluentApacheKafkaImplementor {
    pub fn new(slight_state: &BasicState) -> Self {
        let bootstap_servers = String::from_utf8(
            runtime_configs::providers::get(
                &slight_state.secret_store,
                "CK_ENDPOINT",
                &slight_state.config_toml_file_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'CK_ENDPOINT' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();
        let security_protocol = String::from_utf8(
            runtime_configs::providers::get(
                &slight_state.secret_store,
                "CK_SECURITY_PROTOCOL",
                &slight_state.config_toml_file_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'CK_SECURITY_PROTOCOL' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();
        let sasl_mechanisms = String::from_utf8(
            runtime_configs::providers::get(
                &slight_state.secret_store,
                "CK_SASL_MECHANISMS",
                &slight_state.config_toml_file_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'CK_SASL_MECHANISMS' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();
        let sasl_username = String::from_utf8(
            runtime_configs::providers::get(
                &slight_state.secret_store,
                "CK_SASL_USERNAME",
                &slight_state.config_toml_file_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'CK_SASL_USERNAME' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();

        let sasl_password = String::from_utf8(
            runtime_configs::providers::get(
                &slight_state.secret_store,
                "CK_SASL_PASSWORD",
                &slight_state.config_toml_file_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'CK_SASL_PASSWORD' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();
        let group_id = String::from_utf8(
            runtime_configs::providers::get(
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
            .set("bootstrap.servers", bootstap_servers)
            .set("security.protocol", security_protocol)
            .set("sasl.mechanisms", sasl_mechanisms)
            .set("sasl.username", sasl_username)
            .set("sasl.password", sasl_password)
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
