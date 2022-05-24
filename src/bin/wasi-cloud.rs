use anyhow::{Result, bail};
use clap::Parser;
use runtime::Builder;
use kv_filesystem::KvFilesystem;
use mq_filesystem::MqFilesystem;
use kv_azure_blob::KvAzureBlob;
use runtime::resource::Resource;
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
    let mut builder = builder
        .link_wasi()?
        .link_config()?
        .build_config(&args.config)?;
    let config = builder.config.unwrap();
    let url = &config
            .iter()
            .find(|(name, _)| name == "url")
            .expect("url is required in the capability configuration")
            .1;
    let parsed = Url::parse(url)?;
    match parsed.scheme() {
        "azblob" => {
            builder = builder.link_capability::<KvAzureBlob>(parsed)?;
        },
        "file" => {
            builder = builder.link_capability::<KvFilesystem>(parsed)?;
        },
        "mq" => {
            builder = builder.link_capability::<MqFilesystem>(parsed)?;
        },
        _ => bail!("invalid url: {}, currently wasi-cloud only supports 'file', 'azblob', and 'mq' schemes", parsed),
    }
        // .link_capability_with_config(&args.config, |url, linker, store| {
        //     // TODO (Joe): we should have designed a better way to dynamic load capability. Maybe a
        //     // plugin model like terraform. see [here](https://www.terraform.io/plugin)?
        //     match url.scheme() {
        //         "azblob" => {
        //             let kv = KvAzureBlob::from_url(url)?;
        //             KvAzureBlob::add_to_linker(linker)?;
        //             store.data_mut().data = Some(kv.build_state(url).unwrap());
        //             Ok(())
        //         },
        //         "file" => {
        //             let kv = KvFilesystem::from_url(url)?;
        //             KvFilesystem::add_to_linker(linker)?;
        //             store.data_mut().data = Some(kv.build_state(url).unwrap());
        //             Ok(())
        //         },
        //         "mq" => {
        //             let mq = MqFilesystem::from_url(url)?;
        //             MqFilesystem::add_to_linker(linker)?;
        //             store.data_mut().data = Some(mq.build_state(url).unwrap());
        //             Ok(())
        //         },
        //         _ => bail!("invalid url: {}, currently wasi-cloud only supports 'file', 'azblob', and 'mq' schemes"),
        //     }
        // })?;
    let (mut store, instance) = builder.build(&args.module)?;

    instance
        .get_typed_func::<(i32, i32), i32, _>(&mut store, "main")?
        .call(&mut store, (0, 0))?;
    Ok(())
}

