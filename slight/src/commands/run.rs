use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::{bail, Result};
use as_any::Downcast;
use slight_common::{BasicState, Resource};
use slight_events::{Events, EventsState};
use slight_events_api::StateTable;
use slight_http::Http;
use slight_kv::Kv;
use slight_lockd::Lockd;
use slight_mq::Mq;
use slight_pubsub::Pubsub;
use slight_runtime::{
    ctx::{SlightCtxBuilder, State},
    Builder, Ctx,
};
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
    let (mut store, instance) = host_builder.build().await?;

    let caps = toml.capability.as_ref().unwrap();
    // looking for events capability.
    let events_enabled = caps.iter().any(|cap| cap.resource == "events");

    // looking for http capability.
    let http_enabled = caps.iter().any(|cap| cap.resource == "http");

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
        .get_mut()
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
    let mut builder = Builder::new_default(module)?;
    let mut slight_builder = SlightCtxBuilder::default();
    let mut linked_capabilities: HashSet<String> = HashSet::new();
    let mut capability_store: HashMap<String, BasicState> = HashMap::new();

    builder.link_wasi()?;
    if toml.specversion.as_ref().unwrap() == "0.2" {
        for c in toml.capability.as_ref().unwrap() {
            let resource_type: &str = c.resource.as_str();
            match resource_type {
                "events" => {
                    if !linked_capabilities.contains("events") {
                        builder.link_capability::<Events<Builder>>(resource_type.to_string())?;
                        linked_capabilities.insert("events".to_string());

                        slight_builder =
                            slight_builder.add_state(State::Events(
                                EventsState::<Builder>::new(resource_map.clone()),
                            ))?;
                    } else {
                        bail!("the events capability was already linked");
                    }
                }
                _ if KV_HOST_IMPLEMENTORS.contains(&resource_type) => {
                    if !linked_capabilities.contains("kv") {
                        builder.link_capability::<Kv>("kv".to_string())?;
                        linked_capabilities.insert("kv".to_string());
                    }

                    maybe_add_named_capability_to_store(
                        &mut capability_store,
                        c.clone(),
                        resource_map.clone(),
                        &toml_file_path,
                    )?;

                    slight_builder = slight_builder
                        .add_state(State::Kv(slight_kv::KvState::new(capability_store.clone())))?;
                }
                _ if MQ_HOST_IMPLEMENTORS.contains(&resource_type) => {
                    if !linked_capabilities.contains("mq") {
                        builder.link_capability::<Mq>("mq".to_string())?;
                        linked_capabilities.insert("mq".to_string());
                    }

                    maybe_add_named_capability_to_store(
                        &mut capability_store,
                        c.clone(),
                        resource_map.clone(),
                        &toml_file_path,
                    )?;

                    slight_builder = slight_builder
                        .add_state(State::Mq(slight_mq::MqState::new(capability_store.clone())))?;
                }
                _ if LOCKD_HOST_IMPLEMENTORS.contains(&resource_type) => {
                    if !linked_capabilities.contains("lockd") {
                        builder.link_capability::<Lockd>("lockd".to_string())?;
                        linked_capabilities.insert("lockd".to_string());
                    }

                    maybe_add_named_capability_to_store(
                        &mut capability_store,
                        c.clone(),
                        resource_map.clone(),
                        &toml_file_path,
                    )?;

                    slight_builder = slight_builder.add_state(State::Lockd(
                        slight_lockd::LockdState::new(capability_store.clone()),
                    ))?;
                }
                _ if PUBSUB_HOST_IMPLEMENTORS.contains(&resource_type) => {
                    if !linked_capabilities.contains("pubsub") {
                        builder.link_capability::<Pubsub>("pubsub".to_string())?;
                        linked_capabilities.insert("pubsub".to_string());
                    }

                    maybe_add_named_capability_to_store(
                        &mut capability_store,
                        c.clone(),
                        resource_map.clone(),
                        &toml_file_path,
                    )?;

                    slight_builder = slight_builder.add_state(State::PubSub(
                        slight_pubsub::PubsubState::new(capability_store.clone()),
                    ))?;
                }
                _ if CONFIGS_HOST_IMPLEMENTORS.contains(&resource_type) => {
                    if !linked_capabilities.contains("configs") {
                        builder.link_capability::<Configs>("configs".to_string())?;
                        linked_capabilities.insert("configs".to_string());
                    }

                    maybe_add_named_capability_to_store(
                        &mut capability_store,
                        c.clone(),
                        resource_map.clone(),
                        &toml_file_path,
                    )?;

                    slight_builder = slight_builder.add_state(State::RtCfg(
                        slight_runtime_configs::ConfigsState::new(capability_store.clone()),
                    ))?;
                }
                "http" => {
                    if !linked_capabilities.contains("http") {
                        builder.link_capability::<Http<Builder>>(resource_type.to_string())?;
                        linked_capabilities.insert("http".to_string());

                        slight_builder =
                            slight_builder.add_state(State::Http(slight_http::HttpState::<
                                Builder,
                            >::new(
                                resource_map.clone()
                            )))?;
                    } else {
                        bail!("the http capability was already linked");
                    }
                }
                _ => {
                    bail!("invalid url: currently slight only supports 'configs.usersecrets', 'configs.envvars', 'events', 'kv.filesystem', 'kv.azblob', 'kv.awsdynamodb', 'mq.filesystem', 'mq.azsbus', 'lockd.etcd', 'pubsub.confluent_apache_kafka', and 'http' schemes")
                }
            }
        }
    } else {
        bail!("unsupported toml spec version");
    }
    builder = builder.add_slight_states(slight_builder);
    Ok(builder)
}

fn maybe_add_named_capability_to_store(
    capability_store: &mut HashMap<String, BasicState>,
    c: Capability,
    resource_map: Arc<Mutex<StateTable>>,
    toml_file_path: impl AsRef<Path>,
) -> Result<()> {
    if let std::collections::hash_map::Entry::Vacant(e) = capability_store.entry(c.name.clone()) {
        e.insert(BasicState::new(
            resource_map,
            c.resource.clone(),
            c.name.clone(),
            c.configs,
            toml_file_path,
        ));
    } else {
        bail!("cannot add capabilities of the same name");
    }

    Ok(())
}
