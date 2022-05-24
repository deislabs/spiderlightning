pub mod resource;
use anyhow::Result;
use resource::{Resource, State};
use url::Url;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_common::{StringArrayError, WasiCtx};
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::*;

wit_bindgen_wasmtime::import!("../../wit/config.wit");

/// A wasmtime runtime context to be passed to a wasm module.
#[derive(Default)]
pub struct Context<T> 
{
    pub wasi: Option<WasiCtx>,
    pub config_data: config::ConfigData,
    pub data: Option<T>,
}

pub struct Runtime<T> 
{
    pub store: Store<Context<T>>,
    pub linker: Linker<Context<T>>,
    pub module: Module,
}

/// A wasmtime-based runtime builder.
pub struct Builder<T> 
{
    linker: Linker<Context<T>>,
    store: Store<Context<T>>,
    engine: Engine,
    pub config: Option<Vec<(String, String)>>,
}

impl<T> Builder<T> 
{
    /// Create a new runtime builder.
    pub fn new_default() -> Result<Self> {
        let wasi = default_wasi()?;
        let engine = Engine::new(&default_config()?)?;
        let linker = Linker::new(&engine);
        let ctx = Context {
            wasi: Some(wasi),
            config_data: config::ConfigData::default(),
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

    pub fn link_wasi(&mut self) -> Result<&mut Self> {
        wasmtime_wasi::add_to_linker(&mut self.linker, |cx: &mut Context<_>| {
            cx.wasi.as_mut().unwrap()
        })?;
        Ok(self)
    }

    pub fn link_config(&mut self) -> Result<&mut Self> {
        config::Config::add_to_linker(&mut self.linker, |cx: &mut Context<_>| &mut cx.config_data)?;
        Ok(self)
    }

    pub fn build_config(
        &mut self, 
        config: &str, 
        // link_state: impl Fn(Url, &mut Linker<Context<T>>, &mut Store<Context<T>>) -> Result<()>,
    ) -> Result<&mut Self> {
        let module = Module::from_file(&self.engine, config)?;
        let instance = self.linker.instantiate(&mut self.store, &module)?;
        let config = config::Config::new(&mut self.store, &instance, |ctx| &mut ctx.config_data)?;
        let config = config.get_capability(&mut self.store).unwrap()?;
        // let url = &config
        //     .iter()
        //     .find(|(name, _)| name == "url")
        //     .expect("url is required in the capability configuration")
        //     .1;
        // let parsed = Url::parse(url)?;
        // // self.store.data_mut().data = Some(build_state(parsed, &mut self.linker)?);
        // link_state(parsed, &mut self.linker, &mut self.store)?;
        // Ok(self)
        self.config = Some(config);
        Ok(self)
    }

    pub fn link_capability<U>(&mut self, url: Url) -> Result<&mut Self> 
    where
        U: Resource<State = T>,
    {
        U::add_to_linker(&mut self.linker)?;
        self.store.data_mut().data = Some(U::build_state(url)?);
        Ok(self)
    }

    pub fn build(mut self, module: &str) -> Result<(Store<Context<T>>, Instance)> {
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

impl From<config::Error> for anyhow::Error {
    fn from(_: config::Error) -> Self {
        anyhow::anyhow!("config error")
    }
}
