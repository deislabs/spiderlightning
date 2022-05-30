pub mod resource;
use anyhow::Result;
use resource::{DataT, HostResource};
use url::Url;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_common::{StringArrayError, WasiCtx};
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::*;

/// A wasmtime runtime context to be passed to a wasm module.
#[derive(Default)]
pub struct Context<T> {
    pub wasi: Option<WasiCtx>,
    pub data: Option<T>,
}

/// A wasmtime-based runtime builder.
pub struct Builder {
    linker: Linker<Context<DataT>>,
    store: Store<Context<DataT>>,
    engine: Engine,
    pub config: Option<Vec<(String, String)>>,
}

impl Builder {
    /// Create a new runtime builder.
    pub fn new_default() -> Result<Self> {
        let wasi = default_wasi()?;
        let engine = Engine::new(&default_config()?)?;
        let linker = Linker::new(&engine);
        let ctx = Context {
            wasi: Some(wasi),
            data: None,
        };

        let store = Store::new(&engine, ctx);
        Ok(Self {
            linker,
            store,
            engine,
            config: None,
        })
    }

    /// Link wasi to the wasmtime::Linker
    pub fn link_wasi(&mut self) -> Result<&mut Self> {
        wasmtime_wasi::add_to_linker(&mut self.linker, |cx: &mut Context<_>| {
            cx.wasi.as_mut().unwrap()
        })?;
        Ok(self)
    }

    /// Link a host capability to the wasmtime::Linker
    pub fn link_capability<T: HostResource>(&mut self, url: Url) -> Result<&mut Self> {
        T::add_to_linker(&mut self.linker)?;
        self.store.data_mut().data = Some(T::build_data(url)?);
        Ok(self)
    }

    /// Instantiate the guest module.
    pub fn build(mut self, module: &str) -> Result<(Store<Context<DataT>>, Instance)> {
        let module = Module::from_file(&self.engine, module)?;
        let instance = self.linker.instantiate(&mut self.store, &module)?;
        Ok((self.store, instance))
    }
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
