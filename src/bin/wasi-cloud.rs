use anyhow::Result;
use clap::Parser;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_common::{StringArrayError, WasiCtx};
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::*;

use kv_filesystem::kv::KvTables;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    module: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let wasi = default_wasi()?;
    let engine = Engine::new(&default_config()?)?;
    let module = Module::from_file(&engine, args.module)?;
    let mut linker = Linker::new(&engine);

    let ctx = Context {
        wasi,
        data: (
            kv_filesystem::KvFilesystem::new(".".to_string()),
            KvTables::<kv_filesystem::KvFilesystem>::default(),
        ),
    };

    wasmtime_wasi::add_to_linker(&mut linker, |cx: &mut Context<_>| &mut cx.wasi)?;
    kv_filesystem::add_to_linker(
        &mut linker,
        |cx: &mut Context<(kv_filesystem::KvFilesystem, KvTables<kv_filesystem::KvFilesystem>)>| (&mut cx.data.0, &mut cx.data.1),
    )?;

    let mut store = Store::new(&engine, ctx);
    let instance = linker.instantiate(&mut store, &module)?;
    instance
        .get_typed_func::<(i32, i32), i32, _>(&mut store, "main")?
        .call(&mut store, (0, 0))?;
    Ok(())
}

pub fn default_config() -> Result<Config> {
    let mut config = Config::new();
    config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    config.wasm_multi_memory(true);
    config.wasm_module_linking(true);
    Ok(config)
}

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

struct Context<T> {
    wasi: WasiCtx,
    data: T,
}
