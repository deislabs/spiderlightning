use std::env;

use anyhow::{Context, Result};
use events_api::Event;
use proc_macro_utils::Resource;
use rdkafka::{consumer::BaseConsumer, producer::BaseProducer, ClientConfig};
use runtime::{
    impl_resource,
    resource::{
        get_table, Ctx, DataT, Linker, Map, Resource, ResourceMap, ResourceTables, RuntimeResource,
    },
};

use pubsub::*;
use uuid::Uuid;
wit_bindgen_wasmtime::export!("../../wit/pubsub.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);
use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};

mod confluent;

const SCHEME_NAME: &str = "ckpubsub";

/// A Confluent Apache Kafka implementation for the pubsub interface.
#[derive(Default, Clone, Resource)]
pub struct PubSubConfluentKafka {
    inner: Option<(Arc<BaseProducer>, Arc<BaseConsumer>)>,
    host_state: Option<ResourceMap>,
}

impl_resource!(
    PubSubConfluentKafka,
    pubsub::PubsubTables<PubSubConfluentKafka>,
    ResourceMap,
    SCHEME_NAME.to_string()
);

impl PubSubConfluentKafka {
    /// Create a new `PubSubConfluentKafka`
    pub fn new(
        bootstap_servers: &str,
        security_protocol: &str,
        sasl_mechanisms: &str,
        sasl_username: &str,
        sasl_password: &str,
        group_id: &str,
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
            inner: Some((Arc::new(producer), Arc::new(consumer))),
            host_state: None,
        }
    }
}

impl pubsub::Pubsub for PubSubConfluentKafka {
    type Pubsub = String;
    /// Construct a new `PubSubConfluentKafka`
    fn pubsub_open(&mut self) -> Result<Self::Pubsub, Error> {
        let bootstap_servers = env::var("CK_ENDPOINT")
            .with_context(|| "failed to read CK_ENDPOINT environment variable")?;
        let security_protocol = env::var("CK_SECURITY_PROTOCOL")
            .with_context(|| "failed to read CK_SECURITY_PROTOCOL environment variable")?;
        let sasl_mechanisms = env::var("CK_SASL_MECHANISMS")
            .with_context(|| "failed to read CK_SASL_MECHANISMS environment variable")?;
        let sasl_username = env::var("CK_SASL_USERNAME")
            .with_context(|| "failed to read CK_SASL_USERNAME environment variable")?;
        let sasl_password = env::var("CK_SASL_PASSWORD")
            .with_context(|| "failed to read CK_SASL_PASSWORD environment variable")?;
        let group_id = env::var("CK_GROUP_ID")
            .with_context(|| "failed to read CK_GROUP_ID environment variable")?;

        let ck_pubsub = Self::new(
            &bootstap_servers,
            &security_protocol,
            &sasl_mechanisms,
            &sasl_username,
            &sasl_password,
            &group_id,
        );
        self.inner = ck_pubsub.inner;

        let rd = Uuid::new_v4().to_string();
        let cloned = self.clone();
        let mut map = Map::lock(&mut self.host_state)?;
        map.set(rd.clone(), (Box::new(cloned), None));
        Ok(rd)
    }

    /// Send messages to a topic
    fn pubsub_send_message_to_topic(
        &mut self,
        self_: &Self::Pubsub,
        msg_key: PayloadParam<'_>,
        msg_value: PayloadParam<'_>,
        topic: &str,
    ) -> Result<(), Error> {
        Uuid::parse_str(self_)
            .with_context(|| "internal error: failed to parse internal handle to this resource")?;

        let map = Map::lock(&mut self.host_state)?;
        let inner = map.get::<(Arc<BaseProducer>, Arc<BaseConsumer>)>(self_)?;

        Ok(confluent::send(&inner.0, msg_key, msg_value, topic)
            .with_context(|| "failed to send message to a topic")?)
    }

    /// Subscribe to a topic
    fn pubsub_subscribe_to_topic(
        &mut self,
        self_: &Self::Pubsub,
        topic: Vec<&str>,
    ) -> Result<(), Error> {
        Uuid::parse_str(self_)
            .with_context(|| "internal error: failed to parse internal handle to this resource")?;

        let map = Map::lock(&mut self.host_state)?;
        let inner = map.get::<(Arc<BaseProducer>, Arc<BaseConsumer>)>(self_)?;

        Ok(
            confluent::subscribe(&inner.1, topic)
                .with_context(|| "failed to subscribe to topic")?,
        )
    }

    /// Receive/poll for messages
    fn pubsub_poll_for_message(
        &mut self,
        self_: &Self::Pubsub,
        timeout_in_secs: u64,
    ) -> Result<Message, Error> {
        Uuid::parse_str(self_)
            .with_context(|| "internal error: failed to parse internal handle to this resource")?;

        let map = Map::lock(&mut self.host_state)?;
        let inner = map.get::<(Arc<BaseProducer>, Arc<BaseConsumer>)>(self_)?;

        Ok(confluent::poll(&inner.1, timeout_in_secs)
            .map(|f| pubsub::Message {
                key: f.0,
                value: f.1,
            })
            .with_context(|| "failed to poll for message")?)
    }
}
