use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use anyhow::{bail, Result};
use as_any::Downcast;
use slight_common::{BasicState, Capability, WasmtimeBuildable};
use slight_distributed_locking::DistributedLocking;
use slight_http::Http;
use slight_http_outbound::HttpOutbound;
use slight_keyvalue::Keyvalue;
use slight_messaging::Messaging;
use slight_runtime::{Builder, Ctx};
use slight_runtime_configs::Configs;
use spiderlightning::core::slightfile::{Capability as TomlCapability, TomlFile};
use wit_bindgen_wasmtime::wasmtime::Store;

const KEYVALUE_HOST_IMPLEMENTORS: [&str; 8] = [
    "kv.filesystem",
    "kv.azblob",
    "kv.awsdynamodb",
    "kv.redis",
    "keyvalue.filesystem",
    "keyvalue.azblob",
    "keyvalue.awsdynamodb",
    "keyvalue.redis",
];
const DISTRIBUTED_LOCKING_HOST_IMPLEMENTORS: [&str; 2] = ["lockd.etcd", "distributed_locking.etcd"];
const MESSAGING_HOST_IMPLEMENTORS: [&str; 8] = [
    "pubsub.confluent_apache_kafka",
    "pubsub.mosquitto",
    "messaging.confluent_apache_kafka",
    "messaging.mosquitto",
    "mq.azsbus",
    "mq.filesystem",
    "messaging.azsbus",
    "messaging.filesystem",
];
const CONFIGS_HOST_IMPLEMENTORS: [&str; 3] =
    ["configs.usersecrets", "configs.envvars", "configs.azapp"];

pub async fn handle_run(module: impl AsRef<Path>, toml_file_path: impl AsRef<Path>) -> Result<()> {
    let toml_file_contents =
        std::fs::read_to_string(&toml_file_path).expect("could not locate slightfile");
    let toml =
        toml::from_str::<TomlFile>(&toml_file_contents).expect("provided file is not a slightfile");

    tracing::info!("Starting slight");

    let host_builder = build_store_instance(&toml, &toml_file_path, &module).await?;
    let (mut store, instance) = host_builder.build().await;

    let caps = toml.capability.as_ref().unwrap();

    // looking for the http capability.
    let http_enabled;

    if toml.specversion == "0.1" {
        http_enabled = caps.iter().any(|cap| cap.name == "http");
    } else if toml.specversion == "0.2" {
        http_enabled = caps
            .iter()
            .any(|cap| cap.resource.as_ref().expect("missing resource field") == "http");
    } else {
        bail!("unsupported toml spec version");
    }

    if http_enabled {
        log::debug!("Http capability enabled");
        let guest_builder: Builder = build_store_instance(&toml, &toml_file_path, &module).await?;
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
    T: Capability,
{
    let err_msg = format!(
        "internal error: slight context does not contain key: {scheme_name}"
    );
    let err_msg2 = format!(
        "internal error: slight context contains key {scheme_name} but can't downcast"
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

async fn build_store_instance(
    toml: &TomlFile,
    toml_file_path: impl AsRef<Path>,
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

        if resource_type != "http" {
            maybe_add_named_capability_to_store(
                &toml.specversion,
                toml.secret_store.clone(),
                &mut capability_store,
                c.clone(),
                &toml_file_path,
            )?;
        }

        match resource_type {
            _ if KEYVALUE_HOST_IMPLEMENTORS.contains(&resource_type) => {
                if !linked_capabilities.contains("keyvalue") {
                    builder.link_capability::<Keyvalue>()?;
                    linked_capabilities.insert("keyvalue".to_string());
                }

                let resource = slight_keyvalue::Keyvalue::new(
                    resource_type.to_string(),
                    capability_store.clone(),
                );
                builder.add_to_builder("keyvalue".to_string(), resource);
            }
            _ if DISTRIBUTED_LOCKING_HOST_IMPLEMENTORS.contains(&resource_type) => {
                if !linked_capabilities.contains("distributed_locking") {
                    builder.link_capability::<DistributedLocking>()?;
                    linked_capabilities.insert("distributed_locking".to_string());
                }

                let resource = slight_distributed_locking::DistributedLocking::new(
                    resource_type.to_string(),
                    capability_store.clone(),
                );
                builder.add_to_builder("distributed_locking".to_string(), resource);
            }
            _ if MESSAGING_HOST_IMPLEMENTORS.contains(&resource_type) => {
                if !linked_capabilities.contains("messaging") {
                    builder.link_capability::<Messaging>()?;
                    linked_capabilities.insert("messaging".to_string());
                }

                let resource =
                    slight_messaging::Messaging::new(&c.name, capability_store.clone()).await;
                builder.add_to_builder("messaging".to_string(), resource);
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
                    let http = slight_http::Http::<Builder>::default();
                    builder
                        .link_capability::<Http<Builder>>()?
                        .add_to_builder("http".to_string(), http);
                    linked_capabilities.insert("http".to_string());
                } else {
                    bail!("the http capability was already linked");
                }
            }
            "http-outbound" => {
                if !linked_capabilities.contains("http-outbound") {
                    let http_outbound = HttpOutbound::new();
                    builder
                        .link_capability::<HttpOutbound>()?
                        .add_to_builder("http-outbound".to_string(), http_outbound);
                    linked_capabilities.insert("http-outbound".to_string());
                } else {
                    bail!("the http-outbound capability was already linked");
                }
            }
            _ => {
                bail!("invalid url: currently slight only supports 'configs.usersecrets', 'configs.envvars', 'keyvalue.filesystem', 'keyvalue.azblob', 'keyvalue.awsdynamodb', 'distributed_locking.etcd', 'messaging.confluent_apache_kafka', 'messaging.mosquitto', and 'http' schemes")
            }
        }
    }

    Ok(builder)
}

fn maybe_add_named_capability_to_store(
    specversion: &str,
    secret_store: Option<String>,
    capability_store: &mut HashMap<String, BasicState>,
    c: TomlCapability,
    toml_file_path: impl AsRef<Path>,
) -> Result<()> {
    if specversion == "0.1" {
        if let std::collections::hash_map::Entry::Vacant(e) = capability_store.entry(c.name.clone())
        {
            e.insert(BasicState::new(
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
