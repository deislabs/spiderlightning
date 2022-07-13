use anyhow::{bail, Result};
use as_any::Downcast;
use events::Events;
use events_api::event_handler::EventHandler;
use kv_azure_blob::KvAzureBlob;
use kv_filesystem::KvFilesystem;
use lockd_etcd::LockdEtcd;
use mq_azure_servicebus::MqAzureServiceBus;
use mq_filesystem::MqFilesystem;
use pubsub_confluent_kafka::PubSubConfluentKafka;
use runtime::{resource::Map, Builder};
use std::sync::{Arc, Mutex};

use crate::wc_config::TomlFile;

pub fn handle_run(module: &str, toml: &TomlFile) -> Result<()> {
    let resource_map = Arc::new(Mutex::new(Map::default()));

    let mut host_builder = Builder::new_default()?;
    let mut guest_builder = Builder::new_default()?;
    host_builder.link_wasi()?;
    guest_builder.link_wasi()?;
    let mut events_enabled = false;
    if toml.specversion.as_ref().unwrap() == "0.1" {
        for c in toml.capability.as_ref().unwrap() {
            let resource_type: &str = c.name.as_str();
            match resource_type {
            "events" => {
                events_enabled = true;
                host_builder.link_capability::<Events>(resource_type.to_string(), resource_map.clone())?;
                guest_builder.link_capability::<Events>(resource_type.to_string(), resource_map.clone())?;
            },
            "azblobkv" => {
                host_builder.link_capability::<KvAzureBlob>(resource_type.to_string(), resource_map.clone())?;
                guest_builder.link_capability::<KvAzureBlob>(resource_type.to_string(), resource_map.clone())?;
            },
            "filekv" => {
                host_builder.link_capability::<KvFilesystem>(resource_type.to_string(), resource_map.clone())?;
                guest_builder.link_capability::<KvFilesystem>(resource_type.to_string(), resource_map.clone())?;
            },
            "filemq" => {
                host_builder.link_capability::<MqFilesystem>(resource_type.to_string(), resource_map.clone())?;
                guest_builder.link_capability::<MqFilesystem>(resource_type.to_string(), resource_map.clone())?;
            },
            "azsbusmq" => {
                host_builder.link_capability::<MqAzureServiceBus>(resource_type.to_string(), resource_map.clone())?;
                guest_builder.link_capability::<MqAzureServiceBus>(resource_type.to_string(), resource_map.clone())?;
            },
            "etcdlockd" => {
                host_builder.link_capability::<LockdEtcd>(resource_type.to_string(), resource_map.clone())?;
                guest_builder.link_capability::<LockdEtcd>(resource_type.to_string(), resource_map.clone())?;
            },
            "ckpubsub" => {
                host_builder.link_capability::<PubSubConfluentKafka>(resource_type.to_string(), resource_map.clone())?;
                guest_builder.link_capability::<PubSubConfluentKafka>(resource_type.to_string(), resource_map.clone())?;
            }
            _ => bail!("invalid url: currently wasi-cloud only supports 'events', 'filekv', 'azblobkv', 'filemq', 'azsbusmq', 'etcdlockd', and 'ckpubsub' schemes"),
        }
        }
    } else {
        bail!("unsupported toml spec version");
    }
    let (_, mut store, _) = host_builder.build(module)?;
    let (_, mut store2, instance2) = guest_builder.build(module)?;
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
    Ok(())
}
