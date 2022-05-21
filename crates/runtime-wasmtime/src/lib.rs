use anyhow::{bail, Result};
use as_any::Downcast;
use capability::{DataT, Resource};
use kv_azure_blob::{kv::KvTables as KvAzureBlobTables, KvAzureBlob};
use kv_filesystem::{kv::KvTables as KvFileSystemTables, KvFilesystem};
use mq_filesystem::{mq::MqTables, MqFilesystem};
use url::Url;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_common::{StringArrayError, WasiCtx};
use wasmtime::{AsContextMut, Config, Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::*;

wit_bindgen_wasmtime::import!("../../wit/config.wit");

/// A wasmtime runtime context to be passed to a wasm module.
#[derive(Default)]
pub struct Context<T> {
    pub wasi: Option<WasiCtx>,
    pub config_data: config::ConfigData,
    pub data: Option<T>,
}

pub struct Runtime {
    pub store: Store<Context<DataT>>,
    pub linker: Linker<Context<DataT>>,
    pub module: Module,
}

/// A wasmtime-based runtime builder.
pub struct Builder {
    linker: Linker<Context<DataT>>,
    store: Store<Context<DataT>>,
    engine: Engine,
}

impl Builder {
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

    pub fn link_capability_with_config(&mut self, config: &str) -> Result<&mut Self> {
        let module = Module::from_file(&self.engine, config)?;
        let instance = self.linker.instantiate(&mut self.store, &module)?;
        let config = config::Config::new(&mut self.store, &instance, |ctx| &mut ctx.config_data)?;
        let config = config.get_capability(&mut self.store).unwrap()?;
        let (resource, resource_tables) = load_config(config, &mut self.linker)?;
        self.store.data_mut().data = Some((resource, resource_tables));
        Ok(self)
    }

    pub fn build(mut self, module: &str) -> Result<(Store<Context<DataT>>, Instance)> {
        // let module = Module::from_file(&self.engine, module)?;
        // Ok(Runtime {
        //     store: self.store,
        //     linker: self.linker,
        //     module,
        // })
        let module = Module::from_file(&self.engine, module)?;
        let instance = self.linker.instantiate(&mut self.store, &module)?;
        Ok((self.store, instance))
    }
}

/// Load capability will return a trait object Resource and its Tables
pub fn load_config(
    config: Vec<(String, String)>,
    linker: &mut Linker<Context<DataT>>,
) -> Result<DataT> {
    let url = &config
        .iter()
        .find(|(name, _)| name == "url")
        .expect("url is required in the capability configuration")
        .1;
    let parsed = Url::parse(url)?;

    // TODO (Joe): we should have designed a better way to dynamic load capability. Maybe a
    // plugin model like terraform. see [here](https://www.terraform.io/plugin)?

    match parsed.scheme() {
        "azblob" => {
            kv_azure_blob::add_to_linker(linker, get::<KvAzureBlob, KvAzureBlobTables<KvAzureBlob>>)?;
            let kv_azure_blob = KvAzureBlob::from_url(parsed)?;
            Ok((
                Box::new(kv_azure_blob),
                Box::new(KvAzureBlobTables::<KvAzureBlob>::default()),
            ))
        },
        "file" => {
            kv_filesystem::add_to_linker(linker, get::<KvFilesystem, KvFileSystemTables<KvFilesystem>>)?;
            let kv_filesystem = KvFilesystem::from_url(parsed)?;
            Ok((
                Box::new(kv_filesystem),
                Box::new(KvFileSystemTables::<KvFilesystem>::default()),
            ))
        },
        "mq" => {
            mq_filesystem::add_to_linker(linker, get::<MqFilesystem, MqTables<MqFilesystem>>)?;
            let mq_filesystem = MqFilesystem::from_url(parsed)?;
            Ok((
                Box::new(mq_filesystem),
                Box::new(MqTables::<MqFilesystem>::default()),
            ))
        },
        scheme => bail!(
            "invalid scheme: {}, currently wasi-cloud only supports 'file', 'azblob', and 'mq' schemes",
            scheme
        ),
    }
}

fn get<T, TTables>(cx: &mut Context<DataT>) -> (&mut T, &mut TTables)
where
    T: 'static,
    TTables: 'static,
{
    let data = cx
        .data
        .as_mut()
        .expect("internal error: Runtime context data is None");
    let resource = data.0.as_mut().downcast_mut::<T>().unwrap();
    let resource_tables = data.1.as_mut().downcast_mut::<TTables>().unwrap();
    (resource, resource_tables)
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
