use anyhow::{Context, Result};

use rdkafka::{consumer::BaseConsumer, producer::BaseProducer, ClientConfig};
use runtime::{
    impl_resource,
    resource::{BasicState, Watch},
};
use std::fmt::Debug;

use pubsub::*;
use uuid::Uuid;
wit_bindgen_wasmtime::export!("../../wit/pubsub.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);
wit_error_rs::impl_from!(std::string::FromUtf8Error, Error::ErrorWithDescription);

use std::sync::Arc;

mod confluent;

const SCHEME_NAME: &str = "pubsub.confluent_kafka";

/// A Confluent Apache Kafka implementation for the pub interface.
#[derive(Default, Clone)]
pub struct PubSubConfluentKafka {
    host_state: BasicState,
}

#[derive(Clone)]
pub struct PubConfluentKafkaInner {
    producer: Option<Arc<BaseProducer>>,
}

impl Watch for PubConfluentKafkaInner {}

#[derive(Clone)]
pub struct SubConfluentKafkaInner {
    consumer: Option<Arc<BaseConsumer>>,
}

impl Watch for SubConfluentKafkaInner {}

impl_resource!(
    PubSubConfluentKafka,
    pubsub::PubsubTables<PubSubConfluentKafka>,
    BasicState,
    SCHEME_NAME.to_string()
);

impl Debug for PubConfluentKafkaInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PubConfluentKafkaInner")
    }
}

impl Debug for SubConfluentKafkaInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SubConfluentKafkaInner")
    }
}

impl PubConfluentKafkaInner {
    /// Create a new producer
    pub fn new(
        bootstap_servers: &str,
        security_protocol: &str,
        sasl_mechanisms: &str,
        sasl_username: &str,
        sasl_password: &str,
    ) -> Self {
        // basic producer
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
            producer: Some(Arc::new(producer)),
        }
    }
}

impl SubConfluentKafkaInner {
    /// Create a new consumer
    pub fn new(
        bootstap_servers: &str,
        security_protocol: &str,
        sasl_mechanisms: &str,
        sasl_username: &str,
        sasl_password: &str,
        group_id: &str,
    ) -> Self {
        // basic consumer
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
            consumer: Some(Arc::new(consumer)),
        }
    }
}

impl pubsub::Pubsub for PubSubConfluentKafka {
    type Pub = PubConfluentKafkaInner;
    type Sub = SubConfluentKafkaInner;

    /// Construct a new `PubConfluentKafka`
    fn pub_open(&mut self) -> Result<Self::Pub, Error> {
        let bootstap_servers = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "CK_ENDPOINT",
            &self.host_state.config_toml_file_path,
        )?)?;
        let security_protocol = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "CK_SECURITY_PROTOCOL",
            &self.host_state.config_toml_file_path,
        )?)?;
        let sasl_mechanisms = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "CK_SASL_MECHANISMS",
            &self.host_state.config_toml_file_path,
        )?)?;
        let sasl_username = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "CK_SASL_USERNAME",
            &self.host_state.config_toml_file_path,
        )?)?;

        let sasl_password = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "CK_SASL_PASSWORD",
            &self.host_state.config_toml_file_path,
        )?)?;

        let ck_pubsub_guest = Self::Pub::new(
            &bootstap_servers,
            &security_protocol,
            &sasl_mechanisms,
            &sasl_username,
            &sasl_password,
        );

        let rd = Uuid::new_v4().to_string();
        self.host_state
            .resource_map
            .lock()
            .unwrap()
            .set(rd, Box::new(ck_pubsub_guest.clone()));
        Ok(ck_pubsub_guest)
    }

    fn sub_open(&mut self) -> Result<Self::Sub, Error> {
        let bootstap_servers = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "CK_ENDPOINT",
            &self.host_state.config_toml_file_path,
        )?)?;
        let security_protocol = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "CK_SECURITY_PROTOCOL",
            &self.host_state.config_toml_file_path,
        )?)?;
        let sasl_mechanisms = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "CK_SASL_MECHANISMS",
            &self.host_state.config_toml_file_path,
        )?)?;
        let sasl_username = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "CK_SASL_USERNAME",
            &self.host_state.config_toml_file_path,
        )?)?;

        let sasl_password = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "CK_SASL_PASSWORD",
            &self.host_state.config_toml_file_path,
        )?)?;
        let group_id = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "CK_GROUP_ID",
            &self.host_state.config_toml_file_path,
        )?)?;

        let ck_pubsub_guest = Self::Sub::new(
            &bootstap_servers,
            &security_protocol,
            &sasl_mechanisms,
            &sasl_username,
            &sasl_password,
            &group_id,
        );

        let rd = Uuid::new_v4().to_string();
        self.host_state
            .resource_map
            .lock()
            .unwrap()
            .set(rd, Box::new(ck_pubsub_guest.clone()));
        Ok(ck_pubsub_guest)
    }

    /// Send messages to a topic
    fn pub_send_message_to_topic(
        &mut self,
        self_: &Self::Pub,
        msg_key: PayloadParam<'_>,
        msg_value: PayloadParam<'_>,
        topic: &str,
    ) -> Result<(), Error> {
        Ok(confluent::send(
            self_
                .producer
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("cannot send a message without a pub object"))
                .unwrap(),
            msg_key,
            msg_value,
            topic,
        )
        .with_context(|| "failed to send message to a topic")?)
    }

    /// Subscribe to a topic
    fn sub_subscribe_to_topic(&mut self, self_: &Self::Sub, topic: Vec<&str>) -> Result<(), Error> {
        Ok(confluent::subscribe(
            self_
                .consumer
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("cannot subscribe to topic without a sub object"))
                .unwrap(),
            topic,
        )
        .with_context(|| "failed to subscribe to topic")?)
    }

    /// Receive/poll for messages
    fn sub_poll_for_message(
        &mut self,
        self_: &Self::Sub,
        timeout_in_secs: u64,
    ) -> Result<Message, Error> {
        Ok(confluent::poll(
            self_
                .consumer
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("cannot poll for message without a sub object"))
                .unwrap(),
            timeout_in_secs,
        )
        .map(|f| pubsub::Message {
            key: f.0,
            value: f.1,
        })
        .with_context(|| "failed to poll for message")?)
    }
}
