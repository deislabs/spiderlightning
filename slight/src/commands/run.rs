use anyhow::{bail, Result};
use as_any::Downcast;
use events::{Events, EventsState};
use events_api::event_handler::EventHandler;
use kv::{Kv, KvState};
use lockd::{Lockd, LockdState};
use mq::{Mq, MqState};
use pubsub::{Pubsub, PubsubState};
use runtime::{
    resource::{BasicState, StateTable},
    Builder,
};
use runtime_configs::{Configs, ConfigsState};
use std::sync::{Arc, Mutex};

use spiderlightning::core::slightfile::TomlFile;

const KV_HOST_IMPLEMENTORS: [&str; 2] = ["kv.filesystem", "kv.azblob"];
const MQ_HOST_IMPLEMENTORS: [&str; 2] = ["mq.filesystem", "mq.azsbus"];
const LOCKD_HOST_IMPLEMENTORS: [&str; 1] = ["lockd.etcd"];
const PUBSUB_HOST_IMPLEMENTORS: [&str; 1] = ["pubsub.confluent_apache_kafka"];
const CONFIGS_HOST_IMPLEMENTORS: [&str; 2] = ["configs.usersecrets", "configs.envvars"];

pub fn handle_run(module: &str, toml: &TomlFile, toml_file_path: &str) -> Result<()> {
    tracing::info!("Starting slight");

    let resource_map = Arc::new(Mutex::new(StateTable::default()));

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
                host_builder.link_capability::<Events>(resource_type.to_string(), EventsState::new(resource_map.clone()))?;
                guest_builder.link_capability::<Events>(resource_type.to_string(), EventsState::new(resource_map.clone()))?;
            },
            _ if KV_HOST_IMPLEMENTORS.contains(&resource_type) => {
                if let Some(ss) = &toml.secret_store {
                    host_builder.link_capability::<Kv>("kv".to_string(), KvState::new(resource_type.to_string(), BasicState::new(resource_map.clone(), ss, toml_file_path)))?;
                    guest_builder.link_capability::<Kv>("kv".to_string(), KvState::new(resource_type.to_string(), BasicState::new(resource_map.clone(), ss, toml_file_path)))?;
                } else {
                    bail!("the kv capability requires a secret store of some type (i.e., envvars, or usersecrets) specified in your config file so it knows where to grab, say, the AZURE_STORAGE_ACCOUNT, and AZURE_STORAGE_KEY from.")
                }
            },
            _ if MQ_HOST_IMPLEMENTORS.contains(&resource_type) => {
                if let Some(ss) = &toml.secret_store {
                    host_builder.link_capability::<Mq>("mq".to_string(), MqState::new(resource_type.to_string(), BasicState::new(resource_map.clone(), ss, toml_file_path)))?;
                    guest_builder.link_capability::<Mq>("mq".to_string(), MqState::new(resource_type.to_string(), BasicState::new(resource_map.clone(), ss, toml_file_path)))?;
                } else {
                    bail!("the mq capability requires a secret store of some type (i.e., envvars, or usersecrets) specified in your config file so it knows where to grab the AZURE_SERVICE_BUS_NAMESPACE, AZURE_POLICY_NAME, and AZURE_POLICY_KEY from.")
                }
            }
            _ if LOCKD_HOST_IMPLEMENTORS.contains(&resource_type) => {
                if let Some(ss) = &toml.secret_store {
                    host_builder.link_capability::<Lockd>("lockd".to_string(), LockdState::new(resource_type.to_string(), BasicState::new(resource_map.clone(), ss, toml_file_path)))?;
                    guest_builder.link_capability::<Lockd>("lockd".to_string(),LockdState::new(resource_type.to_string(), BasicState::new(resource_map.clone(), ss, toml_file_path)))?;
                } else {
                    bail!("the lockd capability requires a secret store of some type (i.e., envvars, or usersecrets) specified in your config file so it knows where to grab the ETCD_ENDPOINT.")
                }
            },
            _ if PUBSUB_HOST_IMPLEMENTORS.contains(&resource_type) => {
                if let Some(ss) = &toml.secret_store {
                    host_builder.link_capability::<Pubsub>("pubsub".to_string(), PubsubState::new(resource_type.to_string(), BasicState::new(resource_map.clone(), ss, toml_file_path)))?;
                    guest_builder.link_capability::<Pubsub>("pubsub".to_string(), PubsubState::new(resource_type.to_string(), BasicState::new(resource_map.clone(), ss, toml_file_path)))?;
                } else {
                    bail!("the pubsub capability requires a secret store of some type (i.e., envvars, or usersecrets) specified in your config file so it knows where to grab the CK_SECURITY_PROTOCOL, CK_SASL_MECHANISMS, CK_SASL_USERNAME, CK_SASL_PASSWORD, and CK_GROUP_ID.")
                }
            },
            _ if CONFIGS_HOST_IMPLEMENTORS.contains(&resource_type) => {
                host_builder.link_capability::<Configs>(
                    "configs".to_string(),
                    ConfigsState::new(resource_type.to_string(), BasicState::new(resource_map.clone(), "", toml_file_path)))?;
                guest_builder.link_capability::<Configs>(
                    "configs".to_string(),
                    ConfigsState::new(resource_type.to_string(), BasicState::new(resource_map.clone(), "", toml_file_path)))?;
            }
            _ => bail!("invalid url: currently slight only supports 'configs.usersecrets', 'configs.envvars', 'events', 'kv.filesystem', 'kv.azblob', 'mq.filesystem', 'mq.azsbus', 'lockd.etcd', and 'pubsub.confluent_apache_kafka' schemes"),
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
