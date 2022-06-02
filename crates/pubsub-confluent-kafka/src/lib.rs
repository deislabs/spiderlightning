use std::env;

use anyhow::Result;
use rdkafka::{
    consumer::{BaseConsumer, Consumer},
    producer::{BaseProducer, BaseRecord},
    ClientConfig, Message,
};
use runtime::resource::{get, Context, DataT, HostResource, Linker, Resource, ResourceTables};
use url::{Position, Url};

use pubsub::*;
wit_bindgen_wasmtime::export!("../../wit/pubsub.wit");

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
        msg_key: Payload<'_>,
        msg_value: Payload<'_>,
        topic: &str,
    ) -> Result<(), Error> {
        if *rd != 0 {
            return Err(Error::DescriptorError);
        }
        // TO DO: move this to a module
        self.producer
            .as_ref()
            .unwrap()
            .send(BaseRecord::to(topic).key(msg_key).payload(msg_value))
            .expect("failed to send message");

        Ok(())
    }

    fn subscribe_to_topic(
        &mut self,
        rd: &Self::ResourceDescriptor,
        topic: Vec<&str>,
    ) -> Result<(), Error> {
        if *rd != 0 {
            return Err(Error::DescriptorError);
        }
        // TO DO: move this to a module
        self.consumer
            .as_ref()
            .unwrap()
            .subscribe(&topic)
            .expect("failed to subscribe to topic");

        Ok(())
    }

    // TO DO: I have to think about how we want to do streaming
    fn print_message_stream(&mut self, rd: &Self::ResourceDescriptor) -> Result<(), Error> {
        if *rd != 0 {
            return Err(Error::DescriptorError);
        }
        for message in self.consumer.as_ref().unwrap().iter() {
            let raw_message = message.expect("failed to read message");
            let key: &str = raw_message
                .key_view()
                .unwrap()
                .expect("failed to get message key");
            let value = std::str::from_utf8(raw_message.payload().unwrap());
            println!("{:#?} => {:#?}", key, value);
        }

        Ok(())
    }
}
