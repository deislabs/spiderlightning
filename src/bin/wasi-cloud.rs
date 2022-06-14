use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use clap::Parser;
use kv_azure_blob::KvAzureBlob;
use kv_filesystem::KvFilesystem;
// use lockd_etcd::LockdEtcd;
use mq_azure_servicebus::MqAzureServiceBus;
use mq_filesystem::MqFilesystem;
use serde::Deserialize;
use runtime::{resource::Map, Builder};

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

    let mut builder = Builder::new_default()?;
    builder.link_wasi()?;
    let toml_file = std::fs::read_to_string(args.config)?;
    let toml: Config = toml::from_str(&toml_file)?;
    if toml.specversion.unwrap() == "0.1" {
        for c in toml.capability.unwrap() {
            let resource_type: &str = c.name.as_ref().unwrap();
            match resource_type {
            "azblob" => {
                builder.link_capability::<KvAzureBlob>(resource_type.to_string())?;
            },
            "file" => {
                builder.link_capability::<KvFilesystem>(resource_type.to_string())?;
            },
            "mq" => {
                builder.link_capability::<MqFilesystem>(resource_type.to_string())?;
            },
            "azmq" => {
                builder.link_capability::<MqAzureServiceBus>(resource_type.to_string())?;
            },
            // "etcdlockd" => {
            //     builder.link_capability::<LockdEtcd>(url)?;
            // },
            // "ckpubsub" => {
            //     builder.link_capability::<PubSubConfluentKafka>(url)?;
            // }
            _ => bail!("invalid url: currently wasi-cloud only supports 'file', 'azblob', 'mq', 'azmq', and 'ckpubsub' schemes"),
        }
        }
    } else {
        bail!("unsupported toml spec version");
    }

    builder.link_resource_map(resource_map)?;
    let (mut store, instance) = builder.build(&args.module)?;

    instance
        .get_typed_func::<(i32, i32), i32, _>(&mut store, "main")?
        .call(&mut store, (0, 0))?;
    Ok(())
}
