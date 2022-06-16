use std::{env, sync::Arc};

use anyhow::Result;
use proc_macro_utils::{Resource, RuntimeResource};
use rdkafka::{consumer::BaseConsumer, producer::BaseProducer, ClientConfig};
use runtime::resource::{
    get, Context as RuntimeContext, DataT, Linker, Resource, ResourceMap, RuntimeResource,
};

use pubsub::*;
use uuid::Uuid;
wit_bindgen_wasmtime::export!("../../wit/pubsub.wit");

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
            .expect("failed to create producer");

        // basic consumer
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstap_servers)
            .set("security.protocol", security_protocol)
            .set("sasl.mechanisms", sasl_mechanisms)
            .set("sasl.username", sasl_username)
            .set("sasl.password", sasl_password)
            .set("group.id", group_id)
            .create()
            .expect("failed to create client");

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
            .expect("failed to read CK_SECURITY_PROTOCOL environment variable");
        let sasl_mechanisms = env::var("CK_SASL_MECHANISMS")
            .expect("failed to read CK_SASL_MECHANISMS environment variable");
        let sasl_username = env::var("CK_SASL_USERNAME")
            .expect("failed to read CK_SASL_USERNAME environment variable");
        let sasl_password = env::var("CK_SASL_PASSWORD")
            .expect("failed to read CK_SASL_PASSWORD environment variable");
        let group_id =
            env::var("CK_GROUP_ID").expect("failed to read CK_GROUP_ID environment variable");

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
        let mut map = self
            .resource_map
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        map.set(rd.clone(), Box::new(cloned))?;
        Ok(rd)
    }

    fn send_message_to_topic(
        &mut self,
        rd: ResourceDescriptorParam,
        msg_key: PayloadParam<'_>,
        msg_value: PayloadParam<'_>,
        topic: &str,
    ) -> Result<(), Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }

        let map = self
            .resource_map
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        let inner = map.get::<(Arc<BaseProducer>, Arc<BaseConsumer>)>(rd)?;

        confluent::send(&inner.0, msg_key, msg_value, topic).map_err(|_| Error::IoError)
    }

    fn subscribe_to_topic(
        &mut self,
        rd: ResourceDescriptorParam,
        topic: Vec<&str>,
    ) -> Result<(), Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }
        let map = self
            .resource_map
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        let inner = map.get::<(Arc<BaseProducer>, Arc<BaseConsumer>)>(rd)?;

        confluent::subscribe(&inner.1, topic).map_err(|_| Error::OtherError)
    }

    fn poll_for_message(
        &mut self,
        rd: ResourceDescriptorParam,
        timeout_in_secs: u64,
    ) -> Result<pubsub::Message, Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }
        let map = self
            .resource_map
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        let inner = map.get::<(Arc<BaseProducer>, Arc<BaseConsumer>)>(rd)?;

        confluent::poll(&inner.1, timeout_in_secs)
            .map_err(|_| Error::OtherError)
            .map(|f| pubsub::Message {
                key: f.0,
                value: f.1,
            })
    }
}

impl From<anyhow::Error> for Error {
    fn from(_: anyhow::Error) -> Self {
        Self::OtherError
    }
}
