mod ctx;
pub mod resource;

use std::path::Path;

use anyhow::Result;
use async_trait::async_trait;
use ctx::SlightCtxBuilder;
use resource::{get_host_state, HttpData};
use slight_common::{CapabilityBuilder, WasmtimeBuildable, WasmtimeLinkable};
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_common::WasiCtx;
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};

pub use ctx::SlightCtx;
/// Runtime Context for the wasm module
pub type Ctx = RuntimeContext;

/// A wasmtime runtime context to be passed to a wasm module.
///
/// This context contains the following resources:
///    - `wasi`: a wasi context
///    - `slight`: a slight context
///    - `http_state`: http handler's data
///
/// The runtime context will be used inside of the `Builder`
/// to build a `Store` and `Instance` for the wasm module.
#[derive(Default)]
pub struct RuntimeContext {
    pub wasi: Option<WasiCtx>,
    pub slight: SlightCtx,
    pub http_state: HttpData,
}

impl slight_common::Ctx for RuntimeContext {
    fn get_http_state_mut(&mut self) -> &mut HttpData {
        &mut self.http_state
    }

    fn get_host_state<T: 'static, TTable: 'static>(
        &mut self,
        resource_key: String,
    ) -> (&mut T, &mut TTable) {
        get_host_state(self, resource_key)
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
    state_builder: SlightCtxBuilder,
}

impl Builder {
    /// Create a new runtime builder.
    pub fn from_module(module: impl AsRef<Path>) -> Result<Self> {
        let engine = Engine::new(&default_config()?)?;
        let mut linker = Linker::new(&engine);
        linker.allow_shadowing(true);
        let module = Module::from_file(&engine, module)?;

        Ok(Self {
            linker,
            engine,
            module,
            state_builder: SlightCtxBuilder::default(),
        })
    }

    /// Link wasi to the wasmtime::Linker
    pub fn link_wasi(&mut self) -> Result<&mut Self> {
        wasmtime_wasi::add_to_linker(&mut self.linker, |cx: &mut Ctx| cx.wasi.as_mut().unwrap())?;
        Ok(self)
    }

    /// Link a host capability to the wasmtime::Linker
    pub fn link_capability<T: WasmtimeLinkable>(&mut self) -> Result<&mut Self> {
        tracing::log::info!("Adding capability: {}", std::any::type_name::<T>());
        T::add_to_linker(&mut self.linker)?;
        Ok(self)
    }

    pub fn add_to_builder<T>(&mut self, name: String, resource: T) -> &mut Self
    where
        T: CapabilityBuilder + Send + Sync + Clone + 'static,
    {
        self.state_builder.add_to_builder(|ctx: &mut SlightCtx| {
            ctx.insert(name, T::build(resource).unwrap());
        });
        self
    }
}

#[async_trait]
impl WasmtimeBuildable for Builder {
    type Ctx = Ctx;

    /// Instantiate the guest module.
    async fn build(self) -> (Store<Self::Ctx>, Instance) {
        let wasi = default_wasi().unwrap();

        let ctx = RuntimeContext {
            wasi: Some(wasi),
            slight: self.state_builder.build(),
            http_state: HttpData::default(),
        };

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

#[cfg(test)]
mod unittest {
    use std::collections::HashMap;

    use slight_common::WasmtimeBuildable;
    use slight_kv::Kv;

    use crate::Builder;

    #[tokio::test]
    async fn test_builder_build() -> anyhow::Result<()> {
        let module = "./test/kv-test.wasm";
        assert!(std::path::Path::new(module).exists());
        let mut builder = Builder::from_module(module)?;
        let kv = slight_kv::Kv::new("kv.filesystem".to_string(), HashMap::default());

        builder
            .link_wasi()?
            .link_capability::<Kv>()?
            .add_to_builder("kv".to_string(), kv);

        let (_, _) = builder.build().await;
        Ok(())
    }
}
