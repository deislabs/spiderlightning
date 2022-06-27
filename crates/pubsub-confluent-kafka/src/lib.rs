use std::{env, sync::Arc};

use anyhow::{Context, Result};
use proc_macro_utils::{Resource, RuntimeResource};
use rdkafka::{consumer::BaseConsumer, producer::BaseProducer, ClientConfig};
use runtime::resource::{
    get, DataT, Linker, Map, Resource, ResourceMap, RuntimeContext, RuntimeResource,
};

use pubsub::*;
use uuid::Uuid;
wit_bindgen_wasmtime::export!("../../wit/pubsub.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);

mod confluent;

const SCHEME_NAME: &str = "ckpubsub";

#[derive(Default, Clone, Resource, RuntimeResource)]
pub struct PubSubConfluentKafka {
    inner: Option<(Arc<BaseProducer>, Arc<BaseConsumer>)>,
    resource_map: Option<ResourceMap>,
}

impl PubSubConfluentKafka {
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
            resource_map: None,
        }
    }
}

impl pubsub::Pubsub for PubSubConfluentKafka {
    fn get_pubsub(&mut self, name: &str) -> Result<ResourceDescriptorResult, Error> {
        let bootstap_servers = name;
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
            bootstap_servers,
            &security_protocol,
            &sasl_mechanisms,
            &sasl_username,
            &sasl_password,
            &group_id,
        );
        self.inner = ck_pubsub.inner;
        let uuid = Uuid::new_v4();
        let rd = uuid.to_string();
        let cloned = self.clone();
        let mut map = Map::lock(&mut self.resource_map)?;
        map.set(rd.clone(), Box::new(cloned));
        Ok(rd)
    }

    fn send_message_to_topic(
        &mut self,
        rd: ResourceDescriptorParam,
        msg_key: PayloadParam<'_>,
        msg_value: PayloadParam<'_>,
        topic: &str,
    ) -> Result<(), Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.resource_map)?;
        let inner = map.get::<(Arc<BaseProducer>, Arc<BaseConsumer>)>(rd)?;

        Ok(confluent::send(&inner.0, msg_key, msg_value, topic)
            .with_context(|| "failed to send message to a topic")?)
    }

    fn subscribe_to_topic(
        &mut self,
        rd: ResourceDescriptorParam,
        topic: Vec<&str>,
    ) -> Result<(), Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.resource_map)?;
        let inner = map.get::<(Arc<BaseProducer>, Arc<BaseConsumer>)>(rd)?;

        Ok(
            confluent::subscribe(&inner.1, topic)
                .with_context(|| "failed to subscribe to topic")?,
        )
    }

    fn poll_for_message(
        &mut self,
        rd: ResourceDescriptorParam,
        timeout_in_secs: u64,
    ) -> Result<pubsub::Message, Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.resource_map)?;
        let inner = map.get::<(Arc<BaseProducer>, Arc<BaseConsumer>)>(rd)?;

        Ok(confluent::poll(&inner.1, timeout_in_secs)
            .map(|f| pubsub::Message {
                key: f.0,
                value: f.1,
            })
            .with_context(|| "failed to poll for message")?)
    }
}
