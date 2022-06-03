use std::env;

use anyhow::Result;
use rdkafka::{consumer::BaseConsumer, producer::BaseProducer, ClientConfig};
use runtime::resource::{get, Context, DataT, HostResource, Linker, Resource, ResourceTables};
use url::{Position, Url};

use pubsub::*;
wit_bindgen_wasmtime::export!("../../wit/pubsub.wit");

mod confluent;

#[derive(Default)]
pub struct PubSubConfluentKafka {
    producer: Option<BaseProducer>,
    consumer: Option<BaseConsumer>,
}

impl Resource for PubSubConfluentKafka {
    fn from_url(url: Url) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let bootstap_servers = &url[Position::AfterPassword..];
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

        Ok(Self::new(
            bootstap_servers,
            &security_protocol,
            &sasl_mechanisms,
            &sasl_username,
            &sasl_password,
            &group_id,
        ))
    }
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
            producer: Some(producer),
            consumer: Some(consumer),
        }
    }
}

impl<T> ResourceTables<dyn Resource> for PubsubTables<T> where T: Pubsub + 'static {}

impl HostResource for PubSubConfluentKafka {
    fn add_to_linker(linker: &mut Linker<Context<DataT>>) -> Result<()> {
        crate::add_to_linker(linker, get::<Self, crate::PubsubTables<Self>>)
    }

    fn build_data(url: Url) -> Result<DataT> {
        let mq_azure_servicebus = Self::from_url(url)?;
        Ok((
            Box::new(mq_azure_servicebus),
            Box::new(crate::PubsubTables::<Self>::default()),
        ))
    }
}

impl pubsub::Pubsub for PubSubConfluentKafka {
    type ResourceDescriptor = u64;

    fn get_pubsub(&mut self) -> Result<Self::ResourceDescriptor, Error> {
        Ok(0)
    }

    fn send_message_to_topic(
        &mut self,
        rd: &Self::ResourceDescriptor,
        msg_key: PayloadParam<'_>,
        msg_value: PayloadParam<'_>,
        topic: &str,
    ) -> Result<(), Error> {
        if *rd != 0 {
            return Err(Error::DescriptorError);
        }

        confluent::send(self.producer.as_ref().unwrap(), msg_key, msg_value, topic)
            .map_err(|_| Error::IoError)
    }

    fn subscribe_to_topic(
        &mut self,
        rd: &Self::ResourceDescriptor,
        topic: Vec<&str>,
    ) -> Result<(), Error> {
        if *rd != 0 {
            return Err(Error::DescriptorError);
        }

        confluent::subscribe(self.consumer.as_ref().unwrap(), topic).map_err(|_| Error::OtherError)
    }

    fn poll_for_message(
        &mut self,
        rd: &Self::ResourceDescriptor,
        timeout_in_secs: u64,
    ) -> Result<pubsub::Message, Error> {
        if *rd != 0 {
            return Err(Error::DescriptorError);
        }
        
        confluent::poll(self.consumer.as_ref().unwrap(), timeout_in_secs)
            .map_err(|_| Error::OtherError)
            .map(|f| pubsub::Message {
                key: f.0,
                value: f.1,
            })
    }
}
