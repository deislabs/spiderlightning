use anyhow::Result;
use clap::Parser;
// use wasi_cap_std_sync::WasiCtxBuilder;
// use wasi_common::{StringArrayError, WasiCtx};
// use wasmtime::{Config, Engine, Linker, Module, Store};
// use wasmtime_wasi::*;

use runtime::Builder;

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
    builder
        .link_wasi()?
        .link_config()?
        .link_capability_with_config(&args.config)?;
    let (mut store, instance) = builder.build(&args.module)?;

    instance
        .get_typed_func::<(i32, i32), i32, _>(&mut store, "main")?
        .call(&mut store, (0, 0))?;
    Ok(())
}
