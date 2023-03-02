mod ctx;
pub mod resource;

use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
};

use anyhow::Result;
use async_trait::async_trait;
use ctx::SlightCtxBuilder;
use resource::{get_host_state, HttpData};
use slight_common::{CapabilityBuilder, WasmtimeBuildable, WasmtimeLinkable};
use wasi_cap_std_sync::{ambient_authority, Dir, WasiCtxBuilder};
use wasi_common::pipe::{ReadPipe, WritePipe};
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

/// Input and output redirects to be used for the running module
#[derive(Clone, Default)]
pub struct IORedirects {
    pub stdout_path: Option<PathBuf>,
    pub stderr_path: Option<PathBuf>,
    pub stdin_path: Option<PathBuf>,
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
    io_redirects: IORedirects,
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
            io_redirects: IORedirects::default(),
        })
    }

    /// Set the I/O redirects for the module
    pub fn set_io(mut self, io_redirects: IORedirects) -> Self {
        self.io_redirects = io_redirects;
        self
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

fn maybe_open_stdio(pipe_path: &Path) -> Option<File> {
    if pipe_path.as_os_str().is_empty() {
        None
    } else {
        Some(
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(pipe_path)
                .unwrap_or_else(|_| {
                    panic!(
                        "could not open pipe: {path}",
                        path = pipe_path.to_str().unwrap()
                    )
                }),
        )
    }
}

#[async_trait]
impl WasmtimeBuildable for Builder {
    type Ctx = Ctx;

    /// Instantiate the guest module.
    async fn build(self) -> (Store<Self::Ctx>, Instance) {
        let wasi = build_wasi_context(self.io_redirects).unwrap();
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

fn build_wasi_context(io_redirects: IORedirects) -> Result<WasiCtx> {
    let mut ctx: WasiCtxBuilder = WasiCtxBuilder::new();
    ctx = add_io_redirects_to_wasi_context(ctx, io_redirects)?;
    Ok(ctx
        .inherit_args()?
        .preopened_dir(Dir::open_ambient_dir(".", ambient_authority())?, ".")?
        .build())
}

/// add_io_redirects_to_wasi_context inherits existing stdio and overrides stdio as available.
fn add_io_redirects_to_wasi_context(
    mut ctx: WasiCtxBuilder,
    io_redirects: IORedirects,
) -> Result<WasiCtxBuilder> {
    ctx = ctx.inherit_stdio();
    if let Some(stdout_path) = io_redirects.stdout_path {
        if let Some(stdout_file) = maybe_open_stdio(&stdout_path) {
            ctx = ctx.stdout(Box::new(WritePipe::new(stdout_file)));
        }
    }

    if let Some(stderr_path) = io_redirects.stderr_path {
        if let Some(stderr_file) = maybe_open_stdio(&stderr_path) {
            ctx = ctx.stderr(Box::new(WritePipe::new(stderr_file)));
        }
    }

    if let Some(stdin_path) = io_redirects.stdin_path {
        if let Some(stdin_file) = maybe_open_stdio(&stdin_path) {
            ctx = ctx.stdin(Box::new(ReadPipe::new(stdin_file)));
        }
    }
    Ok(ctx)
}

// TODO (Joe): expose the wasmtime config as a capability?
pub fn default_config() -> Result<Config> {
    let mut config = Config::new();
    config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    config.wasm_multi_memory(true);
    config.async_support(true);
    Ok(config)
}

#[cfg(test)]
mod unittest {
    // use std::collections::HashMap;
    use std::fs::File;
    use std::path::PathBuf;

    // use slight_common::WasmtimeBuildable;
    // use slight_keyvalue::Keyvalue;
    use tempfile::tempdir;

    // use crate::Builder;

    // TODO(DJ): re-enable this test --currently broken
    // #[tokio::test]
    // async fn test_builder_build() -> anyhow::Result<()> {
    //     let module = "./test/keyvalue-test.wasm";
    //     assert!(std::path::Path::new(module).exists());
    //     let mut builder = Builder::from_module(module)?;
    //     let keyvalue =
    //         slight_keyvalue::Keyvalue::new("keyvalue.filesystem".to_string(), HashMap::default());
    //
    //     builder
    //         .link_wasi()?
    //         .link_capability::<Keyvalue>()?
    //         .add_to_builder("keyvalue".to_string(), keyvalue);
    //
    //     let (_, _) = builder.build().await;
    //     Ok(())
    // }

    #[test]
    fn test_maybe_open_stdio_with_existing_file() -> anyhow::Result<()> {
        let tmp_dir = tempdir()?;
        let existing_file_path = tmp_dir.path().join("testpath");
        let empty_file_path = PathBuf::new();
        let _ = File::create(&existing_file_path)?;
        assert!(crate::maybe_open_stdio(&existing_file_path).is_some());
        assert!(crate::maybe_open_stdio(&empty_file_path).is_none());
        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_maybe_open_stdio_with_missing_file() {
        let missing_file_path = PathBuf::from("missing");
        let _ = crate::maybe_open_stdio(&missing_file_path);
    }
}
