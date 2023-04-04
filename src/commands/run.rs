use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use as_any::Downcast;
#[cfg(feature = "blob-store")]
use slight_blob_store::{BlobStore, BLOB_STORE_SCHEME_NAME};
use slight_common::{BasicState, Capability, Ctx as _, WasmtimeBuildable};
#[cfg(feature = "distributed-locking")]
use slight_distributed_locking::DistributedLocking;
use slight_file::{
    Capability as TomlCapability, Resource, SecretStoreResource, SlightFile, SlightFileBuilder,
    SpecVersion,
};
#[cfg(feature = "http-client")]
use slight_http_client::HttpClient;

#[cfg(feature = "http-server")]
use slight_http_server::{HttpServer, HttpServerInit};

#[cfg(feature = "keyvalue")]
use slight_keyvalue::Keyvalue;
#[cfg(feature = "messaging")]
use slight_messaging::Messaging;
use slight_runtime::{Builder, Ctx};
#[cfg(feature = "runtime-configs")]
use slight_runtime_configs::Configs;
#[cfg(feature = "sql")]
use slight_sql::Sql;
use wit_bindgen_wasmtime::wasmtime::Store;

pub type IORedirects = slight_runtime::IORedirects;

#[derive(Clone, Default)]
pub struct RunArgs {
    pub module: PathBuf,
    pub slightfile: PathBuf,
    pub io_redirects: Option<IORedirects>,
}

pub async fn handle_run(args: RunArgs) -> Result<()> {
    let toml = SlightFileBuilder::new()
        .path(args.slightfile.clone())?
        .build()?;
    let http_enabled = toml.has_http_cap();
    tracing::info!("Starting slight");

    let mut host_builder =
        build_store_instance(toml.as_ref(), &args.slightfile, &args.module).await?;
    if let Some(io_redirects) = args.io_redirects.clone() {
        tracing::info!("slight io redirects were specified");
        host_builder = host_builder.set_io(io_redirects);
    }
    let (mut store, instance) = host_builder.build().await;

    // looking for the http capability.
    if cfg!(feature = "http-server") && http_enabled {
        log::debug!("Http capability enabled");
        update_http_states(
            toml.as_ref(),
            args.slightfile,
            args.module,
            &mut store,
            args.io_redirects,
        )
        .await?;

        // invoke on_server_init
        let http_server =
            HttpServerInit::new(&mut store, &instance, |ctx| ctx.get_http_server_mut())?;

        let res = http_server.on_server_init(&mut store).await?;
        match res {
            Ok(_) => {}
            Err(e) => bail!(e),
        }

        log::info!("waiting for http to finish...");
        close_http_server(store).await;
    } else {
        instance
            .get_typed_func::<(), _>(&mut store, "_start")?
            .call_async(&mut store, ())
            .await?;
    }
    Ok(())
}

#[cfg(not(feature = "http-server"))]
async fn update_http_states(
    _toml: &SlightFile,
    _toml_file_path: impl AsRef<Path>,
    _module: impl AsRef<Path>,
    _store: &mut Store<slight_runtime::RuntimeContext>,
) -> Result<(), anyhow::Error> {
    log::debug!("http-server feature is not enabled");
    Ok(())
}

#[cfg(feature = "http-server")]
async fn update_http_states(
    toml: &SlightFile,
    toml_file_path: impl AsRef<Path>,
    module: impl AsRef<Path>,
    store: &mut Store<slight_runtime::RuntimeContext>,
    maybe_stdio: Option<IORedirects>,
) -> Result<(), anyhow::Error> {
    let mut guest_builder: Builder = build_store_instance(toml, &toml_file_path, &module).await?;
    if let Some(ioredirects) = maybe_stdio {
        tracing::info!("setting HTTP guest builder io redirects");
        guest_builder = guest_builder.set_io(ioredirects);
    }
    let http_api_resource: &mut HttpServer<Builder> = get_resource(store, "http");
    http_api_resource.update_state(slight_common::Builder::new(guest_builder))?;
    Ok(())
}

#[cfg(not(feature = "http-server"))]
async fn close_http_server(_store: Store<slight_runtime::RuntimeContext>) {
    log::debug!("http-server feature is not enabled");
}

#[cfg(feature = "http-server")]
async fn close_http_server(mut store: Store<slight_runtime::RuntimeContext>) {
    shutdown_signal().await;
    let http_api_resource: &mut HttpServer<Builder> = get_resource(&mut store, "http");
    http_api_resource.close();
}

fn get_resource<'a, T>(store: &'a mut Store<Ctx>, scheme_name: &'a str) -> &'a mut T
where
    T: Capability,
{
    let err_msg = format!("internal error: slight context does not contain key: {scheme_name}");
    let err_msg2 =
        format!("internal error: slight context contains key {scheme_name} but can't downcast");
    store
        .data_mut()
        .slight
        .get_mut(scheme_name)
        .expect(&err_msg)
        .0
        .as_mut()
        .downcast_mut::<T>()
        .expect(&err_msg2)
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

async fn build_store_instance(
    toml: &SlightFile,
    toml_file_path: impl AsRef<Path>,
    module: impl AsRef<Path>,
) -> Result<Builder> {
    let mut builder = Builder::from_module(module)?;
    let mut linked_capabilities: HashSet<String> = HashSet::new();
    let mut capability_store: HashMap<String, BasicState> = HashMap::new();

    builder.link_wasi()?;
    for c in toml.capability.as_ref().unwrap() {
        let resource_type = c.resource();

        match resource_type {
            Resource::HttpServer(_) => {}
            _ => maybe_add_named_capability_to_store(
                toml.specversion,
                toml.secret_store.clone(),
                &mut capability_store,
                c.clone(),
                &toml_file_path,
            )?,
        }

        match resource_type {
            #[cfg(feature = "blob-store")]
            Resource::Blob(_) => {
                if !linked_capabilities.contains(BLOB_STORE_SCHEME_NAME) {
                    builder.link_capability::<BlobStore>()?;
                    linked_capabilities.insert(BLOB_STORE_SCHEME_NAME.to_string());
                }
                let resource = slight_blob_store::BlobStore::new(
                    resource_type.to_string(),
                    capability_store.clone(),
                );
                builder.add_to_builder(BLOB_STORE_SCHEME_NAME.to_string(), resource);
            }
            #[cfg(feature = "keyvalue")]
            Resource::Keyvalue(_) => {
                if !linked_capabilities.contains("keyvalue") {
                    builder.link_capability::<Keyvalue>()?;
                    linked_capabilities.insert("keyvalue".to_string());
                }

                let resource = slight_keyvalue::Keyvalue::new(
                    resource_type.to_string(),
                    capability_store.clone(),
                );
                builder.add_to_builder("keyvalue".to_string(), resource);
            }
            #[cfg(feature = "distributed-locking")]
            Resource::DistributedLocking(_) => {
                if !linked_capabilities.contains("distributed_locking") {
                    builder.link_capability::<DistributedLocking>()?;
                    linked_capabilities.insert("distributed_locking".to_string());
                }

                let resource = slight_distributed_locking::DistributedLocking::new(
                    resource_type.to_string(),
                    capability_store.clone(),
                );
                builder.add_to_builder("distributed_locking".to_string(), resource);
            }
            #[cfg(feature = "messaging")]
            Resource::Messaging(_) => {
                if !linked_capabilities.contains("messaging") {
                    builder.link_capability::<Messaging>()?;
                    linked_capabilities.insert("messaging".to_string());
                }

                let resource = slight_messaging::Messaging::new(
                    &c.name().to_string(),
                    capability_store.clone(),
                )
                .await?;
                builder.add_to_builder("messaging".to_string(), resource);
            }
            #[cfg(feature = "runtime-configs")]
            Resource::Configs(_) => {
                if !linked_capabilities.contains("configs") {
                    builder.link_capability::<Configs>()?;
                    linked_capabilities.insert("configs".to_string());
                }

                let resource = slight_runtime_configs::Configs::new(
                    resource_type.to_string(),
                    capability_store.clone(),
                );
                builder.add_to_builder("configs".to_string(), resource);
            }
            #[cfg(feature = "sql")]
            Resource::Sql(_) => {
                if !linked_capabilities.contains("sql") {
                    builder.link_capability::<Sql>()?;
                    linked_capabilities.insert("sql".to_string());
                }

                let resource =
                    slight_sql::Sql::new(resource_type.to_string(), capability_store.clone());
                builder.add_to_builder("sql".to_string(), resource);
            }
            #[cfg(feature = "http-server")]
            Resource::HttpServer(_) => {
                if !linked_capabilities.contains("http") {
                    let http = slight_http_server::HttpServer::<Builder>::default();
                    builder
                        .link_capability::<HttpServer<Builder>>()?
                        .add_to_builder("http".to_string(), http);
                    linked_capabilities.insert("http".to_string());
                } else {
                    bail!("the http capability was already linked");
                }
            }
            #[cfg(feature = "http-client")]
            Resource::HttpClient(_) => {
                if !linked_capabilities.contains("http-client") {
                    let http_client = HttpClient::new();
                    builder
                        .link_capability::<HttpClient>()?
                        .add_to_builder("http-client".to_string(), http_client);
                    linked_capabilities.insert("http-client".to_string());
                } else {
                    bail!("the http-client capability was already linked");
                }
            }
        }
    }

    Ok(builder)
}

fn maybe_add_named_capability_to_store(
    specversion: SpecVersion,
    secret_store: Option<SecretStoreResource>,
    capability_store: &mut HashMap<String, BasicState>,
    c: TomlCapability,
    toml_file_path: impl AsRef<Path>,
) -> Result<()> {
    match specversion {
        SpecVersion::V1 => {
            if let std::collections::hash_map::Entry::Vacant(e) =
                capability_store.entry(c.name().to_string())
            {
                e.insert(BasicState::new(
                    secret_store,
                    c.resource(),
                    c.name().to_string(),
                    c.configs(),
                    toml_file_path,
                ));
            } else {
                bail!("cannot add capabilities of the same name");
            }
        }
        SpecVersion::V2 => {
            if let std::collections::hash_map::Entry::Vacant(e) =
                capability_store.entry(c.name().to_string())
            {
                let resource = c.resource();

                e.insert(BasicState::new(
                    None,
                    resource,
                    c.name().to_string(),
                    c.configs(),
                    toml_file_path,
                ));
            } else {
                bail!("cannot add capabilities of the same name");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod unittest {
    use crate::commands::run::{handle_run, RunArgs};
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use slight_runtime::IORedirects;
    use std::fs::File;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_handle_run_with_io() -> anyhow::Result<()> {
        let module = "./src/commands/test/io-test.wasm";
        assert!(Path::new(module).exists());
        let slightfile = "./src/commands/test/slightfile.toml";
        assert!(Path::new(slightfile).exists());

        let tmp_dir = tempdir()?;
        let stdin_path = tmp_dir.path().join("stdin");
        let stdout_path = tmp_dir.path().join("stdout");
        let stderr_path = tmp_dir.path().join("stderr");

        let canary: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        fs::write(&stdin_path, &canary).await?;
        let _ = File::create(&stdout_path)?;
        let _ = File::create(&stderr_path)?;

        let args = RunArgs {
            module: PathBuf::from(module),
            slightfile: PathBuf::from(slightfile),
            io_redirects: Some(IORedirects {
                stdin_path: Some(PathBuf::from(&stdin_path)),
                stdout_path: Some(PathBuf::from(&stdout_path)),
                stderr_path: Some(PathBuf::from(&stderr_path)),
            }),
        };

        handle_run(args).await?;
        let stdout_output = fs::read_to_string(&stdout_path).await?;
        assert_eq!(stdout_output, canary);
        let stderr_output = fs::read_to_string(&stderr_path).await?;
        assert_eq!(stderr_output, format!("error: {canary}"));
        Ok(())
    }
}
