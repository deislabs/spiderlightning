use anyhow::Result;
use clap::Parser;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_common::{StringArrayError, WasiCtx};
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::*;

use capability::{config, Context};

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

    let wasi = default_wasi()?;
    let engine = Engine::new(&default_config()?)?;    
    let config_module = Module::from_file(&engine, args.config)?;
    let mut linker = Linker::new(&engine);
    let ctx = Context {
        wasi,
        config_data: config::ConfigData::default(),
        data: None,
    };

    let mut store = Store::new(&engine, ctx);
    wasmtime_wasi::add_to_linker(&mut linker, |cx: &mut Context<_>| &mut cx.wasi)?;

    // First instantiate the config wasm module.
    let (config, _) =
        config::Config::instantiate(&mut store, &config_module, &mut linker, |ctx| {
            &mut ctx.config_data
        })?;

    // Then get the capability from config module and modify wasmtime store.
    let config = config.get_capability(&mut store).unwrap()?;
    let (resource, resource_tables) = capability::load_capability(config, &mut linker)?;
    store.data_mut().data = Some((resource, resource_tables));

    // Finally instantiate the application wasm module.
    let module = Module::from_file(&engine, args.module)?;
    let instance = linker.instantiate(&mut store, &module)?;
    instance
        .get_typed_func::<(i32, i32), i32, _>(&mut store, "main")?
        .call(&mut store, (0, 0))?;
    Ok(())
}

// TODO (Joe): expose the wasmtime config as a capability?
pub fn default_config() -> Result<Config> {
    let mut config = Config::new();
    config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    config.wasm_multi_memory(true);
    config.wasm_module_linking(true);
    Ok(config)
}

// TODO (Joe): expose the wasmtime wasi context as a capability?
pub fn default_wasi() -> Result<WasiCtx, StringArrayError> {
    let mut ctx: WasiCtxBuilder = WasiCtxBuilder::new().inherit_stdio().inherit_args()?;
    ctx = ctx
        .preopened_dir(
            Dir::open_ambient_dir("./target", ambient_authority()).unwrap(),
            "cache",
        )
        .unwrap();

    Ok(ctx.build())
}
