use anyhow::{bail, Result};
use clap::Parser;
use kv_azure_blob::KvAzureBlob;
use kv_filesystem::KvFilesystem;
use lockd_etcd::LockdEtcd;
use mq_azure_servicebus::MqAzureServiceBus;
use mq_filesystem::MqFilesystem;
use pubsub_confluent_kafka::PubSubConfluentKafka;
use serde::Deserialize;

use runtime::Builder;
use url::Url;

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
    url: Option<String>,
}
/// The entry point for wasi-cloud CLI
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut builder = Builder::new_default()?;
    builder.link_wasi()?;
    let toml_file = std::fs::read_to_string(args.config)?;
    let toml: Config = toml::from_str(&toml_file)?;
    if toml.specversion.unwrap() == "0.1" {
        for c in toml.capability.unwrap() {
            match c.name.unwrap().as_str() {
            "azblob" => {
                builder.link_capability::<KvAzureBlob>(Url::parse(&c.url.unwrap()).unwrap())?;
            },
            "file" => {
                builder.link_capability::<KvFilesystem>(Url::parse(&c.url.unwrap()).unwrap())?;
            },
            "mq" => {
                builder.link_capability::<MqFilesystem>(Url::parse(&c.url.unwrap()).unwrap())?;
            },
            "azmq" => {
                builder.link_capability::<MqAzureServiceBus>(Url::parse(&c.url.unwrap()).unwrap())?;
            },
            "etcdlockd" => {
                builder.link_capability::<LockdEtcd>(Url::parse(&c.url.unwrap()).unwrap())?;
            },
            "ckpubsub" => {
                builder.link_capability::<PubSubConfluentKafka>(Url::parse(&c.url.unwrap()).unwrap())?;
            }
            _ => bail!("invalid url: currently wasi-cloud only supports 'file', 'azblob', 'mq', 'azmq', and 'ckpubsub' schemes"),
        }
        }
    } else {
        bail!("unsupported toml spec version");
    }

    let (mut store, instance) = builder.build(&args.module)?;

    instance
        .get_typed_func::<(i32, i32), i32, _>(&mut store, "main")?
        .call(&mut store, (0, 0))?;
    Ok(())
}
