use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::{bail, Result};
use as_any::Downcast;
use slight_common::{BasicState, Resource, WasmtimeBuildable};
use slight_events::Events;
use slight_events_api::StateTable;
use slight_http::Http;
use slight_kv::Kv;
use slight_lockd::Lockd;
use slight_mq::Mq;
use slight_pubsub::Pubsub;
use slight_runtime::{Builder, Ctx};
use slight_runtime_configs::Configs;
use spiderlightning::core::slightfile::{Capability, TomlFile};
use wit_bindgen_wasmtime::wasmtime::Store;

const KV_HOST_IMPLEMENTORS: [&str; 4] =
    ["kv.filesystem", "kv.azblob", "kv.awsdynamodb", "kv.redis"];
const MQ_HOST_IMPLEMENTORS: [&str; 2] = ["mq.filesystem", "mq.azsbus"];
const LOCKD_HOST_IMPLEMENTORS: [&str; 1] = ["lockd.etcd"];
const PUBSUB_HOST_IMPLEMENTORS: [&str; 2] = ["pubsub.confluent_apache_kafka", "pubsub.mosquitto"];
const CONFIGS_HOST_IMPLEMENTORS: [&str; 3] =
    ["configs.usersecrets", "configs.envvars", "configs.azapp"];

pub async fn handle_run(module: impl AsRef<Path>, toml_file_path: impl AsRef<Path>) -> Result<()> {
    let toml_file_contents = std::fs::read_to_string(&toml_file_path)?;
    let toml = toml::from_str::<TomlFile>(&toml_file_contents)?;

    tracing::info!("Starting slight");

    let resource_map = Arc::new(Mutex::new(StateTable::default()));

    let host_builder = build_store_instance(&toml, &toml_file_path, resource_map.clone(), &module)?;
    let (mut store, instance) = host_builder.build().await;

    let caps = toml.capability.as_ref().unwrap();

    // looking for the events and http capabilities.
    let events_enabled;
    let http_enabled;

    if toml.specversion == "0.1" {
        events_enabled = caps.iter().any(|cap| cap.name == "events");
        http_enabled = caps.iter().any(|cap| cap.name == "http");
    } else if toml.specversion == "0.2" {
        events_enabled = caps
            .iter()
            .any(|cap| cap.resource.as_ref().expect("missing resource field") == "events");
        http_enabled = caps
            .iter()
            .any(|cap| cap.resource.as_ref().expect("missing resource field") == "http");
    } else {
        bail!("unsupported toml spec version");
    }

    if events_enabled {
        log::debug!("Events capability enabled");
        let guest_builder =
            build_store_instance(&toml, &toml_file_path, resource_map.clone(), &module)?;
        let event_handler_resource: &mut Events<Builder> = get_resource(&mut store, "events");
        event_handler_resource.update_state(slight_common::Builder::new(guest_builder))?;
    }

    if http_enabled {
        log::debug!("Http capability enabled");
        let guest_builder: Builder =
            build_store_instance(&toml, &toml_file_path, resource_map.clone(), &module)?;
        let http_api_resource: &mut Http<Builder> = get_resource(&mut store, "http");
        http_api_resource.update_state(slight_common::Builder::new(guest_builder))?;
    }

    instance
        .get_typed_func::<(), _, _>(&mut store, "_start")?
        .call_async(&mut store, ())
        .await?;

    if http_enabled {
        log::info!("waiting for http to finish...");
        shutdown_signal().await;
        let http_api_resource: &mut Http<Builder> = get_resource(&mut store, "http");
        http_api_resource.close();
    }
    Ok(())
}

fn get_resource<'a, T>(store: &'a mut Store<Ctx>, scheme_name: &'a str) -> &'a mut T
where
    T: Resource,
{
    let err_msg = format!(
        "internal error: resource_map does not contain key: {}",
        scheme_name
    );
    let err_msg2 = format!(
        "internal error: resource map contains key {} but can't downcast",
        scheme_name
    );
    store
        .data_mut()
        .slight
        .get_mut(scheme_name)
        .expect(&err_msg)
        .0
        .as_mut()
        .downcast_mut::<T>()
        .expect(&err_msg2)
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

fn build_store_instance(
    toml: &TomlFile,
    toml_file_path: impl AsRef<Path>,
    resource_map: Arc<Mutex<StateTable>>,
    module: impl AsRef<Path>,
) -> Result<Builder> {
    let mut builder = Builder::from_module(module)?;
    let mut linked_capabilities: HashSet<String> = HashSet::new();
    let mut capability_store: HashMap<String, BasicState> = HashMap::new();

    builder.link_wasi()?;
    for c in toml.capability.as_ref().unwrap() {
        let resource_type = if toml.specversion == "0.1" {
            c.name.as_str()
        } else if toml.specversion == "0.2" {
            if let Some(r) = &c.resource {
                r.as_str()
            } else {
                bail!("missing resource field");
            }
        } else {
            bail!("unsupported toml spec version");
        };

        if resource_type != "events" && resource_type != "http" {
            maybe_add_named_capability_to_store(
                &toml.specversion,
                toml.secret_store.clone(),
                &mut capability_store,
                c.clone(),
                resource_map.clone(),
                &toml_file_path,
            )?;
        }

        match resource_type {
            "events" => {
                if !linked_capabilities.contains("events") {
                    let events = Events::<Builder>::new(resource_map.clone());
                    builder
                        .link_capability::<Events<Builder>>()?
                        .add_to_builder("events".to_string(), events);
                    linked_capabilities.insert("events".to_string());
                } else {
                    bail!("the events capability was already linked");
                }
            }
            _ if KV_HOST_IMPLEMENTORS.contains(&resource_type) => {
                if !linked_capabilities.contains("kv") {
                    builder.link_capability::<Kv>()?;
                    linked_capabilities.insert("kv".to_string());
                }

                let resource =
                    slight_kv::Kv::new(resource_type.to_string(), capability_store.clone());
                builder.add_to_builder("kv".to_string(), resource);
            }
            _ if MQ_HOST_IMPLEMENTORS.contains(&resource_type) => {
                if !linked_capabilities.contains("mq") {
                    builder.link_capability::<Mq>()?;
                    linked_capabilities.insert("mq".to_string());
                }

                let resource =
                    slight_mq::Mq::new(resource_type.to_string(), capability_store.clone());
                builder.add_to_builder("mq".to_string(), resource);
            }
            _ if LOCKD_HOST_IMPLEMENTORS.contains(&resource_type) => {
                if !linked_capabilities.contains("lockd") {
                    builder.link_capability::<Lockd>()?;
                    linked_capabilities.insert("lockd".to_string());
                }

                let resource =
                    slight_lockd::Lockd::new(resource_type.to_string(), capability_store.clone());
                builder.add_to_builder("lockd".to_string(), resource);
            }
            _ if PUBSUB_HOST_IMPLEMENTORS.contains(&resource_type) => {
                if !linked_capabilities.contains("pubsub") {
                    builder.link_capability::<Pubsub>()?;
                    linked_capabilities.insert("pubsub".to_string());
                }

                let resource =
                    slight_pubsub::Pubsub::new(resource_type.to_string(), capability_store.clone());
                builder.add_to_builder("pubsub".to_string(), resource);
            }
            _ if CONFIGS_HOST_IMPLEMENTORS.contains(&resource_type) => {
                if !linked_capabilities.contains("configs") {
                    builder.link_capability::<Configs>()?;
                    linked_capabilities.insert("configs".to_string());
                }

                let resource = slight_runtime_configs::Configs::new(
                    resource_type.to_string(),
                    capability_store.clone(),
                );
                builder.add_to_builder("configs".to_string(), resource);
            }
            "http" => {
                if !linked_capabilities.contains("http") {
                    let http = slight_http::Http::<Builder>::new(resource_map.clone());
                    builder
                        .link_capability::<Http<Builder>>()?
                        .add_to_builder("http".to_string(), http);
                    linked_capabilities.insert("http".to_string());
                } else {
                    bail!("the http capability was already linked");
                }
            }
            _ => {
                bail!("invalid url: currently slight only supports 'configs.usersecrets', 'configs.envvars', 'events', 'kv.filesystem', 'kv.azblob', 'kv.awsdynamodb', 'mq.filesystem', 'mq.azsbus', 'lockd.etcd', 'pubsub.confluent_apache_kafka', and 'http' schemes")
            }
        }
    }

    Ok(builder)
}

fn maybe_add_named_capability_to_store(
    specversion: &str,
    secret_store: Option<String>,
    capability_store: &mut HashMap<String, BasicState>,
    c: Capability,
    resource_map: Arc<Mutex<StateTable>>,
    toml_file_path: impl AsRef<Path>,
) -> Result<()> {
    if specversion == "0.1" {
        if let std::collections::hash_map::Entry::Vacant(e) = capability_store.entry(c.name.clone())
        {
            e.insert(BasicState::new(
                resource_map,
                secret_store,
                c.name.clone(),
                c.name.clone(),
                c.configs,
                toml_file_path,
            ));
        } else {
            bail!("cannot add capabilities of the same name");
        }
    } else if specversion == "0.2" {
        if let std::collections::hash_map::Entry::Vacant(e) = capability_store.entry(c.name.clone())
        {
            let resource = if let Some(r) = c.resource {
                r
            } else {
                bail!("missing resource field");
            };

            e.insert(BasicState::new(
                resource_map,
                None,
                resource,
                c.name.clone(),
                c.configs,
                toml_file_path,
            ));
        } else {
            bail!("cannot add capabilities of the same name");
        }
    } else {
        bail!("unsupported toml version");
    }

    Ok(())
}
