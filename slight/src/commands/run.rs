use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use as_any::Downcast;
use slight_common::{BasicState, Resource};
use slight_events::{Events, EventsState};
use slight_events_api::StateTable;
use slight_http::Http;
use slight_kv::Kv;
use slight_lockd::{Lockd, LockdState};
use slight_mq::{Mq, MqState};
use slight_pubsub::{Pubsub, PubsubState};
use slight_runtime::{
    ctx::{SlightCtxBuilder, State},
    Builder, Ctx,
};
use slight_runtime_configs::{Configs, ConfigsState};
use spiderlightning::core::slightfile::TomlFile;
use wit_bindgen_wasmtime::wasmtime::Store;

const KV_HOST_IMPLEMENTORS: [&str; 3] = ["kv.filesystem", "kv.azblob", "kv.awsdynamodb"];
const MQ_HOST_IMPLEMENTORS: [&str; 2] = ["mq.filesystem", "mq.azsbus"];
const LOCKD_HOST_IMPLEMENTORS: [&str; 1] = ["lockd.etcd"];
const PUBSUB_HOST_IMPLEMENTORS: [&str; 2] = ["pubsub.confluent_apache_kafka", "pubsub.mosquitto"];
const CONFIGS_HOST_IMPLEMENTORS: [&str; 3] =
    ["configs.usersecrets", "configs.envvars", "configs.azapp"];

pub async fn handle_run(module: &str, toml: &TomlFile, toml_file_path: &str) -> Result<()> {
    tracing::info!("Starting slight");

    let resource_map = Arc::new(Mutex::new(StateTable::default()));

    let host_builder = build_store_instance(toml, toml_file_path, resource_map.clone(), module)?;
    let (mut store, instance) = host_builder.build()?;

    let caps = toml.capability.as_ref().unwrap();
    // looking for events capability.
    let events_enabled = caps.iter().any(|cap| cap.name == "events");

    // looking for http capability.
    let http_enabled = caps.iter().any(|cap| cap.name == "http");

    if events_enabled {
        log::debug!("Events capability enabled");
        let guest_builder =
            build_store_instance(toml, toml_file_path, resource_map.clone(), module)?;
        let (_store2, _instance2) = guest_builder.build()?;
        let event_handler_resource: &mut Events<Builder> = get_resource(&mut store, "events");
        event_handler_resource.update_state(slight_common::Builder::new(guest_builder))?;
    }

    if http_enabled {
        log::debug!("Http capability enabled");
        let guest_builder: Builder =
            build_store_instance(toml, toml_file_path, resource_map.clone(), module)?;
        let http_api_resource: &mut Http<Builder> = get_resource(&mut store, "http");
        http_api_resource.update_state(slight_common::Builder::new(guest_builder))?;
    }

    tracing::info!("Executing {}", module);
    instance
        .get_typed_func::<(), _, _>(&mut store, "_start")?
        .call(&mut store, ())?;

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
    toml_file_path: &str,
    resource_map: Arc<Mutex<StateTable>>,
    module: &str,
) -> Result<Builder> {
    let mut builder = Builder::new_default(module)?;
    let mut slight_builder = SlightCtxBuilder::default();
    builder.link_wasi()?;
    if toml.specversion.as_ref().unwrap() == "0.1" {
        for c in toml.capability.as_ref().unwrap() {
            let resource_type: &str = c.name.as_str();
            match resource_type {
                "events" => {
                    builder.link_capability::<Events<Builder>>(resource_type.to_string())?;
                    slight_builder =
                        slight_builder.add_state(State::Events(EventsState::<Builder>::new(
                            resource_map.clone(),
                        )))?;
                }
                _ if KV_HOST_IMPLEMENTORS.contains(&resource_type) => {
                    if let Some(ss) = &toml.secret_store {
                        builder.link_capability::<Kv>("kv".to_string())?;
                        slight_builder =
                            slight_builder.add_state(State::Kv(slight_kv::KvState::new(
                                resource_type.to_string(),
                                BasicState::new(resource_map.clone(), ss, toml_file_path),
                            )))?;
                    } else {
                        bail!("the kv capability requires a secret store of some type (i.e., envvars, or usersecrets) specified in your config file so it knows where to grab, say, the AZURE_STORAGE_ACCOUNT, and AZURE_STORAGE_KEY from.")
                    }
                }
                _ if MQ_HOST_IMPLEMENTORS.contains(&resource_type) => {
                    if let Some(ss) = &toml.secret_store {
                        builder.link_capability::<Mq>("mq".to_string())?;
                        slight_builder = slight_builder.add_state(State::Mq(MqState::new(
                            resource_type.to_string(),
                            BasicState::new(resource_map.clone(), ss, toml_file_path),
                        )))?;
                    } else {
                        bail!("the mq capability requires a secret store of some type (i.e., envvars, or usersecrets) specified in your config file so it knows where to grab the AZURE_SERVICE_BUS_NAMESPACE, AZURE_POLICY_NAME, and AZURE_POLICY_KEY from.")
                    }
                }
                _ if LOCKD_HOST_IMPLEMENTORS.contains(&resource_type) => {
                    if let Some(ss) = &toml.secret_store {
                        builder.link_capability::<Lockd>("lockd".to_string())?;
                        slight_builder =
                            slight_builder.add_state(State::Lockd(LockdState::new(
                                resource_type.to_string(),
                                BasicState::new(resource_map.clone(), ss, toml_file_path),
                            )))?;
                    } else {
                        bail!("the lockd capability requires a secret store of some type (i.e., envvars, or usersecrets) specified in your config file so it knows where to grab the ETCD_ENDPOINT.")
                    }
                }
                _ if PUBSUB_HOST_IMPLEMENTORS.contains(&resource_type) => {
                    if let Some(ss) = &toml.secret_store {
                        builder.link_capability::<Pubsub>("pubsub".to_string())?;
                        slight_builder =
                            slight_builder.add_state(State::PubSub(PubsubState::new(
                                resource_type.to_string(),
                                BasicState::new(resource_map.clone(), ss, toml_file_path),
                            )))?;
                    } else {
                        bail!("the pubsub capability requires a secret store of some type (i.e., envvars, or usersecrets) specified in your config file so it knows where to grab the AZURE_SERVICE_BUS_NAMESPACE, AZURE_POLICY_NAME, and AZURE_POLICY_KEY from.")
                    }
                }
                _ if CONFIGS_HOST_IMPLEMENTORS.contains(&resource_type) => {
                    builder.link_capability::<Configs>("configs".to_string())?;
                    slight_builder = slight_builder.add_state(State::RtCfg(ConfigsState::new(
                        resource_type.to_string(),
                        BasicState::new(resource_map.clone(), "", toml_file_path),
                    )))?;
                }
                "http" => {
                    builder.link_capability::<Http<Builder>>(resource_type.to_string())?;
                    slight_builder = slight_builder.add_state(State::Http(
                        slight_http::HttpState::<Builder>::new(resource_map.clone()),
                    ))?;
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
