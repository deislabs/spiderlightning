use anyhow::{bail, Result};
use clap::Parser;
use kv_azure_blob::KvAzureBlob;
use kv_filesystem::KvFilesystem;
use mq_azure_servicebus::MqAzureServiceBus;
use mq_filesystem::MqFilesystem;

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

/// The entry point for wasi-cloud CLI
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut builder = Builder::new_default()?;
    builder.link_wasi()?;
    let url = Url::parse(&args.config)?;
    match url.scheme() {
        "azblob" => {
            builder.link_capability::<KvAzureBlob>(url)?;
        },
        "file" => {
            builder.link_capability::<KvFilesystem>(url)?;
        },
        "mq" => {
            builder.link_capability::<MqFilesystem>(url)?;
        },
        "azmq" => {
            builder.link_capability::<MqAzureServiceBus>(url)?;
        },
        _ => bail!("invalid url: {}, currently wasi-cloud only supports 'file', 'azblob', and 'mq' schemes", url),
    }
    let (mut store, instance) = builder.build(&args.module)?;

    instance
        .get_typed_func::<(i32, i32), i32, _>(&mut store, "main")?
        .call(&mut store, (0, 0))?;
    Ok(())
}
