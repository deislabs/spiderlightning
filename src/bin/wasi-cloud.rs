use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use as_any::Downcast;
use clap::Parser;
use events::Events;
use kv_azure_blob::KvAzureBlob;
use kv_filesystem::KvFilesystem;
use lockd_etcd::LockdEtcd;
use mq_azure_servicebus::MqAzureServiceBus;
use mq_filesystem::MqFilesystem;
use pubsub_confluent_kafka::PubSubConfluentKafka;

use runtime::{
    resource::{event_handler::EventHandler, Map},
    Builder,
};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    module: String,
    #[clap(short, long)]
    config: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    specversion: Option<String>,
    capability: Option<Vec<CapabilityConfig>>,
}

#[derive(Debug, Deserialize)]
struct CapabilityConfig {
    name: Option<String>,
}
/// The entry point for wasi-cloud CLI
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let resource_map = Arc::new(Mutex::new(Map::default()));
    let toml_file = std::fs::read_to_string(args.config)?;
    let toml: Config = toml::from_str(&toml_file)?;

    let mut builder = Builder::new_default()?;
    let mut builder2 = Builder::new_default()?;
    builder.link_wasi()?;
    builder2.link_wasi()?;
    let mut events_enabled = false;
    if toml.specversion.unwrap() == "0.1" {
        for c in toml.capability.unwrap() {
            let resource_type: &str = c.name.as_ref().unwrap();
            match resource_type {
            "events" => {
                events_enabled = true;
                builder.link_capability::<Events>(resource_type.to_string())?;
                builder2.link_capability::<Events>(resource_type.to_string())?;
            },
            "azblobkv" => {
                builder.link_capability::<KvAzureBlob>(resource_type.to_string())?;
                builder2.link_capability::<KvAzureBlob>(resource_type.to_string())?;
            },
            "filekv" => {
                builder.link_capability::<KvFilesystem>(resource_type.to_string())?;
                builder2.link_capability::<KvFilesystem>(resource_type.to_string())?;
            },
            "filemq" => {
                builder.link_capability::<MqFilesystem>(resource_type.to_string())?;
                builder2.link_capability::<MqFilesystem>(resource_type.to_string())?;
            },
            "azsbusmq" => {
                builder.link_capability::<MqAzureServiceBus>(resource_type.to_string())?;
                builder2.link_capability::<MqAzureServiceBus>(resource_type.to_string())?;
            },
            "etcdlockd" => {
                builder.link_capability::<LockdEtcd>(resource_type.to_string())?;
                builder2.link_capability::<LockdEtcd>(resource_type.to_string())?;
            },
            "ckpubsub" => {
                builder.link_capability::<PubSubConfluentKafka>(resource_type.to_string())?;
                builder2.link_capability::<PubSubConfluentKafka>(resource_type.to_string())?;
            }
            _ => bail!("invalid url: currently wasi-cloud only supports 'events', 'filekv', 'azblobkv', 'filemq', 'azsbusmq', 'etcdlockd', and 'ckpubsub' schemes"),
        }
        }
    } else {
        bail!("unsupported toml spec version");
    }
    builder.link_resource_map(resource_map.clone())?;
    let (_, mut store, instance) = builder.build(&args.module)?;

    builder2.link_resource_map(resource_map)?;
    let (_, mut store2, instance2) = builder2.build(&args.module)?;
    if events_enabled {
        let event_handler = EventHandler::new(&mut store2, &instance2, |ctx| &mut ctx.state)?;
        store
            .data_mut()
            .data
            .get_mut("events")
            .expect("internal error: resource_map does not contain key events")
            .0
            .as_mut()
            .downcast_mut::<Events>()
            .expect("internal error: resource map contains key events but can't downcast to Events")
            .update_state(
                Arc::new(Mutex::new(store2)),
                Arc::new(Mutex::new(event_handler)),
            )?;
    }
    instance
        .get_typed_func::<(), _, _>(&mut store, "_start")?
        .call(&mut store, ())?;
    Ok(())
}
