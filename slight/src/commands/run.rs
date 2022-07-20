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
use runtime::{
    resource::{BasicState, Map},
    Builder,
};
use runtime_configs::{Configs, ConfigsState};
use std::sync::{Arc, Mutex};

use spiderlightning::core::slightfile::TomlFile;

pub fn handle_run(module: &str, toml: &TomlFile, toml_file_path: &str) -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    tracing::info!("Starting slight");

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
                if let Some(ss) = &toml.secret_store {
                    host_builder.link_capability::<KvAzureBlob>(resource_type.to_string(), BasicState::new(resource_map.clone(), &ss, toml_file_path))?;
                    guest_builder.link_capability::<KvAzureBlob>(resource_type.to_string(), BasicState::new(resource_map.clone(), &ss, toml_file_path))?;
                } else {
                    bail!("the azblobkv capability requires a secret store of some type (i.e., envvars, or usersecrets) specified in your config file so it knows where to grab the AZURE_STORAGE_ACCOUNT, and AZURE_STORAGE_KEY from.")
                }
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
                if let Some(ss) = &toml.secret_store {
                    host_builder.link_capability::<MqAzureServiceBus>(resource_type.to_string(), BasicState::new(resource_map.clone(), &ss, toml_file_path))?;
                    guest_builder.link_capability::<MqAzureServiceBus>(resource_type.to_string(), BasicState::new(resource_map.clone(), &ss, toml_file_path))?;
                } else {
                    bail!("the azsbusmq capability requires a secret store of some type (i.e., envvars, or usersecrets) specified in your config file so it knows where to grab the AZURE_SERVICE_BUS_NAMESPACE, AZURE_POLICY_NAME, and AZURE_POLICY_KEY from.")
                }
            },
            "etcdlockd" => {
                if let Some(ss) = &toml.secret_store {
                    host_builder.link_capability::<LockdEtcd>(resource_type.to_string(), BasicState::new(resource_map.clone(), &ss, toml_file_path))?;
                    guest_builder.link_capability::<LockdEtcd>(resource_type.to_string(), BasicState::new(resource_map.clone(), &ss, toml_file_path))?;
                } else {
                    bail!("the etcdlockd capability requires a secret store of some type (i.e., envvars, or usersecrets) specified in your config file so it knows where to grab the ETCD_ENDPOINT.")
                }
            },
            "ckpubsub" => {
                if let Some(ss) = &toml.secret_store {
                    host_builder.link_capability::<PubSubConfluentKafka>(resource_type.to_string(), BasicState::new(resource_map.clone(), &ss, toml_file_path))?;
                    guest_builder.link_capability::<PubSubConfluentKafka>(resource_type.to_string(), BasicState::new(resource_map.clone(), &ss, toml_file_path))?;
                } else {
                    bail!("the ckpubsub capability requires a secret store of some type (i.e., envvars, or usersecrets) specified in your config file so it knows where to grab the CK_SECURITY_PROTOCOL, CK_SASL_MECHANISMS, CK_SASL_USERNAME, CK_SASL_PASSWORD, and CK_GROUP_ID.")
                }
            },
            "usersecrets_configs" | "envvars_configs" => {
                host_builder.link_capability::<Configs>(
                    "configs".to_string(),
                    ConfigsState::new(resource_map.clone(), resource_type, toml_file_path),
                )?;
                guest_builder.link_capability::<Configs>(
                    "configs".to_string(),
                    ConfigsState::new(resource_map.clone(), resource_type, toml_file_path),
                )?;
            }
            _ => bail!("invalid url: currently slight only supports 'usersecrets_configs', 'envvars_configs', 'events', 'filekv', 'azblobkv', 'filemq', 'azsbusmq', 'etcdlockd', and 'ckpubsub' schemes"),
        }
        }
    } else {
        bail!("unsupported toml spec version");
    }

    let (_, mut store, instance) = host_builder.build(module)?;
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
    tracing::info!("Executing {}", module);
    instance
        .get_typed_func::<(), _, _>(&mut store, "_start")?
        .call(&mut store, ())?;
    Ok(())
}
