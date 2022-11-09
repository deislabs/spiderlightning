pub mod resource;

use std::{collections::HashMap, path::Path};

use anyhow::Result;
use async_trait::async_trait;
// use ctx::{SlightCtx, SlightCtxBuilder};
use resource::{get_host_state, EventsData, HttpData};
use slight_common::{HostState, WasmtimeBuildable, WasmtimeLinkable};
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

    fn get_host_state<T: 'static, TTable: 'static>(
        &mut self,
        resource_key: String,
    ) -> (&mut T, &mut TTable) {
        get_host_state(self, resource_key)
    }
}

/// A runtime context for slight capabilities.
///
/// It is a wrapper around a HashMap of HostState, which are
/// generated bindings for the capabilities, and are linked to
/// the `wasmtime::Linker`.
///
/// The `SlightCtx` cannot be created directly, but it can be
/// constructed using the `SlightCtxBuilder`.
///
/// The `SlightCtx` is not cloneable, but the `SlightCtxBuilder` is.
#[derive(Default)]
pub struct SlightCtx(HashMap<String, HostState>);

impl SlightCtx {
    /// Get a reference to the inner HashMap.
    pub fn get_ref(&self) -> &HashMap<String, HostState> {
        &self.0
    }

    /// Get a mutable reference to the inner HashMap.
    pub fn get_mut(&mut self) -> &mut HashMap<String, HostState> {
        &mut self.0
    }
}

pub trait GetCxFn: FnOnce(&mut SlightCtx) + GetCxFnClone + Send + Sync + 'static {}

impl<T: FnOnce(&mut SlightCtx) + Send + Sync + Clone + 'static> GetCxFn for T {}

pub trait GetCxFnClone {
    fn clone_box(&self) -> Box<dyn GetCxFn>;
}

impl<T> GetCxFnClone for T
where
    T: 'static + Clone + GetCxFn,
{
    fn clone_box(&self) -> Box<dyn GetCxFn> {
        dbg!("clone_box");
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn GetCxFn> {
    fn clone(&self) -> Box<dyn GetCxFn> {
        dbg!("clone");
        (**self).clone_box()
    }
}

#[derive(Clone, Default)]
pub struct StateBuilder {
    pub get_cx_fns: Vec<Box<dyn GetCxFn>>,
}

impl StateBuilder {
    pub fn build(self) -> SlightCtx {
        dbg!("build");
        let mut ctx = SlightCtx::default();
        for f in self.get_cx_fns {
            dbg!("applying closure");
            f(&mut ctx);
        }
        ctx
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
    // states_builder: SlightCtxBuilder<Self>,
    state_builder: StateBuilder,
}

impl Builder {
    /// Create a new runtime builder.
    pub fn new_default(module: impl AsRef<Path>) -> Result<Self> {
        let engine = Engine::new(&default_config()?)?;
        let mut linker = Linker::new(&engine);
        linker.allow_shadowing(true);
        let module = Module::from_file(&engine, module)?;

        Ok(Self {
            linker,
            engine,
            module,
            // states_builder: SlightCtxBuilder::default(),
            state_builder: StateBuilder::default(),
        })
    }

    /// Link wasi to the wasmtime::Linker
    pub fn link_wasi(&mut self) -> Result<&mut Self> {
        wasmtime_wasi::add_to_linker(&mut self.linker, |cx: &mut Ctx| cx.wasi.as_mut().unwrap())?;
        Ok(self)
    }

    /// Link a host capability to the wasmtime::Linker
    pub fn link_capability<T: WasmtimeLinkable>(&mut self, name: String) -> Result<&mut Self> {
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
    // pub fn add_slight_states(mut self, state_builder: SlightCtxBuilder<Self>) -> Self {
    //     self.states_builder = state_builder;
    //     self
    // }

    pub fn add_to_builder(&mut self, get_cx: impl GetCxFn) -> &mut Self {
        self.state_builder.get_cx_fns.push(Box::new(get_cx));
        self
    }
}

#[async_trait]
impl WasmtimeBuildable for Builder {
    type Ctx = Ctx;

    /// Instantiate the guest module.
    async fn build(self) -> (Store<Self::Ctx>, Instance) {
        let wasi = default_wasi().unwrap();

        let mut ctx = RuntimeContext {
            wasi: Some(wasi),
            slight: SlightCtx::default(),
            events_state: EventsData::default(),
            http_state: HttpData::default(),
        };

        // ctx.slight = self.states_builder.clone().build();
        ctx.slight = self.state_builder.build();

        let mut store = Store::new(&self.engine, ctx);
        let instance = self
            .linker
            .instantiate_async(&mut store, &self.module)
            .await
            .unwrap();
        (store, instance)
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
