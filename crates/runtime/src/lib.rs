pub mod ctx;
pub mod resource;

use anyhow::Result;
use async_trait::async_trait;
use ctx::{SlightCtx, SlightCtxBuilder};
use resource::{EventsData, HttpData, Linkable};
use slight_common::Buildable;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_common::WasiCtx;
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};

/// Runtime Context for the wasm module
pub type Ctx = RuntimeContext;

/// A wasmtime runtime context to be passed to a wasm module.
///
/// This context contains the following resources:
///    - `wasi`: a wasi context
///    - `slight`: a slight context
///    - `events_state`: events handler's data
///    - `http_state`: http handler's data
///
/// The runtime context will be used inside of the `Builder`
/// to build a `Store` and `Instance` for the wasm module.
#[derive(Default)]
pub struct RuntimeContext {
    pub wasi: Option<WasiCtx>,
    pub slight: SlightCtx,
    pub events_state: EventsData,
    pub http_state: HttpData,
}

impl slight_common::Ctx for RuntimeContext {
    fn get_http_state_mut(&mut self) -> &mut HttpData {
        &mut self.http_state
    }

    fn get_events_state_mut(&mut self) -> &mut EventsData {
        &mut self.events_state
    }
}

/// A wasmtime-based runtime builder.
///
/// It knows how to build a `Store` and `Instance` for a wasm module, given
/// a `RuntimeContext`, and a `SlightCtxBuilder`.
#[derive(Clone)]
pub struct Builder {
    linker: Linker<Ctx>,
    engine: Engine,
    module: Module,
    states_builder: SlightCtxBuilder<Self>,
}

impl Builder {
    /// Create a new runtime builder.
    pub fn new_default(module: &str) -> Result<Self> {
        let engine = Engine::new(&default_config()?)?;
        let mut linker = Linker::new(&engine);
        linker.allow_shadowing(true);
        let states_builder = SlightCtxBuilder::<Self>::default();
        let module = Module::from_file(&engine, module)?;

        Ok(Self {
            linker,
            engine,
            module,
            states_builder,
        })
    }

    /// Link wasi to the wasmtime::Linker
    pub fn link_wasi(&mut self) -> Result<&mut Self> {
        wasmtime_wasi::add_to_linker(&mut self.linker, |cx: &mut Ctx| cx.wasi.as_mut().unwrap())?;
        Ok(self)
    }

    /// Link a host capability to the wasmtime::Linker
    pub fn link_capability<T: Linkable>(&mut self, name: String) -> Result<&mut Self> {
        tracing::log::info!("Adding capability: {}", &name);
        // self.store
        //     .data_mut()
        //     .data
        //     .insert(name, T::build_data(state)?);
        T::add_to_linker(&mut self.linker)?;
        // self.states.insert(name, Box::new(state));
        Ok(self)
    }

    /// Add slight states to the RuntimeContext
    pub fn add_slight_states(mut self, state_builder: SlightCtxBuilder<Self>) -> Self {
        self.states_builder = state_builder;
        self
    }

    /// Instantiate the guest module.
    pub async fn build(&self) -> Result<(Store<Ctx>, Instance)> {
        let wasi = default_wasi()?;

        let mut ctx = RuntimeContext {
            wasi: Some(wasi),
            slight: SlightCtx::default(),
            events_state: EventsData::default(),
            http_state: HttpData::default(),
        };

        ctx.slight = self.states_builder.clone().build();

        let mut store = Store::new(&self.engine, ctx);
        let instance = self
            .linker
            .instantiate_async(&mut store, &self.module)
            .await?;
        Ok((store, instance))
    }
}

#[async_trait]
impl Buildable for Builder {
    type Ctx = Ctx;

    async fn build(&self) -> (Store<Self::Ctx>, Instance) {
        self.build().await.unwrap()
    }
}

// TODO (Joe): expose the wasmtime config as a capability?
pub fn default_config() -> Result<Config> {
    let mut config = Config::new();
    config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    config.wasm_multi_memory(true);
    config.async_support(true);
    Ok(config)
}

// TODO (Joe): expose the wasmtime wasi context as a capability?
pub fn default_wasi() -> Result<WasiCtx> {
    let ctx: WasiCtxBuilder = WasiCtxBuilder::new().inherit_stdio().inherit_args()?;
    Ok(ctx.build())
}
