use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use clap::Parser;
use kv_azure_blob::KvAzureBlob;
use kv_filesystem::KvFilesystem;
// use lockd_etcd::LockdEtcd;
use mq_azure_servicebus::MqAzureServiceBus;
use mq_filesystem::MqFilesystem;
// use pubsub_confluent_kafka::PubSubConfluentKafka;

use runtime::{resource::Map, Builder};
use url::Url;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    module: String,
    #[clap(short, long)]
    config: String,
}

/// The entry point for wasi-cloud CLI
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let resource_map = Arc::new(Mutex::new(Map::default()));

    let mut builder = Builder::new_default()?;
    builder.link_wasi()?;
    let url = Url::parse(&args.config)?;
    match url.scheme() {
        s@"azblob" => {
            builder.link_capability::<KvAzureBlob>(s.to_string())?;
        },
        s@"file" => {
            builder.link_capability::<KvFilesystem>(s.to_string())?;
        },
        s@"mq" => {
            builder.link_capability::<MqFilesystem>(s.to_string())?;
        },
        s@"azmq" => {
            builder.link_capability::<MqAzureServiceBus>(s.to_string())?;
        },
        // "etcdlockd" => {
        //     builder.link_capability::<LockdEtcd>(url)?;
        // },
        // "ckpubsub" => {
        //     builder.link_capability::<PubSubConfluentKafka>(url)?;
        // }
        _ => bail!("invalid url: {}, currently wasi-cloud only supports 'file', 'azblob', 'mq', 'azmq', and 'ckpubsub' schemes", url),
    }
    builder.link_resource_map(resource_map)?;
    let (mut store, instance) = builder.build(&args.module)?;

    instance
        .get_typed_func::<(i32, i32), i32, _>(&mut store, "main")?
        .call(&mut store, (0, 0))?;
    Ok(())
}
