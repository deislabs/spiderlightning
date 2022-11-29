use std::time::Duration;

use anyhow::Result;
use rdkafka::{
    consumer::{BaseConsumer, Consumer},
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

pub fn subscribe(consumer: &BaseConsumer, topic: Vec<&str>) -> Result<()> {
    consumer.subscribe(&topic)?;
    Ok(())
}

pub async fn receive(consumer: &BaseConsumer) -> Result<Vec<u8>> {
    let message = consumer
        .poll(Duration::from_millis(100))
        .transpose()
        .expect("failed to read message");

    match message {
        Some(m) => {
            consumer.commit_message(&m, rdkafka::consumer::CommitMode::Async).unwrap();
            Ok(m.payload().map(Vec::from).unwrap())
        },
        None => Ok(b"".to_vec()),
    }
}
