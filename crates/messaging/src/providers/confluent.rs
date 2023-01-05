use anyhow::{bail, Result};
#[cfg(feature = "apache_kafka")]
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    producer::{BaseProducer, BaseRecord},
    Message,
};

pub fn publish(
    producer: &BaseProducer,
    msg_key: &[u8],
    msg_value: &[u8],
    topic: &str,
) -> Result<()> {
    producer
        .send(BaseRecord::to(topic).key(msg_key).payload(msg_value))
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(())
}

pub fn subscribe(consumer: &StreamConsumer, topic: Vec<&str>) -> Result<()> {
    consumer.subscribe(&topic)?;
    Ok(())
}

pub async fn receive(consumer: &StreamConsumer) -> Result<Vec<u8>> {
    match consumer.recv().await {
        Err(e) => bail!(e),
        Ok(m) => Ok(m.payload().unwrap().to_vec()),
    }
}
