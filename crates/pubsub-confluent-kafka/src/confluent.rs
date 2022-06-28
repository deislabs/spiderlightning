use std::time::Duration;

use anyhow::Result;
use rdkafka::{
    consumer::{BaseConsumer, Consumer},
    producer::{BaseProducer, BaseRecord},
    Message,
};

pub struct KafkaMessage(pub Option<Vec<u8>>, pub Option<Vec<u8>>);

pub fn send(producer: &BaseProducer, msg_key: &[u8], msg_value: &[u8], topic: &str) -> Result<()> {
    producer
        .send(BaseRecord::to(topic).key(msg_key).payload(msg_value))
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(())
}

pub fn subscribe(consumer: &BaseConsumer, topic: Vec<&str>) -> Result<()> {
    consumer.subscribe(&topic)?;

    Ok(())
}

pub fn poll(consumer: &BaseConsumer, timeout_in_secs: u64) -> Result<KafkaMessage> {
    let message = consumer
        .poll(Duration::from_secs(timeout_in_secs))
        .transpose()?;

    match message {
        Some(m) => Ok(KafkaMessage(
            m.key_view::<[u8]>()
                .transpose()
                .expect("failed to get message key view")
                .map(Vec::from),
            m.payload().map(Vec::from),
        )),
        None => Ok(KafkaMessage(None, None)),
    }
}
