use anyhow::{bail, Result};
use as_any::{AsAny, Downcast};
use kv_azure_blob::{kv::KvTables as KvAzureBlobTables, KvAzureBlob};
use kv_filesystem::{kv::KvTables as KvFileSystemTables, KvFilesystem};
use url::Url;
use wasi_common::WasiCtx;
use wasmtime::Linker;
// use std::any::Any;

wit_bindgen_wasmtime::import!("../../wit/config.wit");

type DataT = (Box<dyn Resource>, Box<dyn ResourceTables<dyn Resource>>);

pub trait ResourceTables<T: ?Sized>: AsAny {}

pub trait Resource: AsAny {
    fn from_url(url: Url) -> Result<Self>
    where
        Self: Sized;
}

impl<T> ResourceTables<dyn Resource> for KvAzureBlobTables<T> where
    T: kv_azure_blob::kv::Kv + 'static
{
}

impl Resource for KvFilesystem {
    fn from_url(url: Url) -> Result<Self> {
        let path = url.to_file_path();
        match path {
            Ok(path) => {
                let path = path.to_str().unwrap();
                Ok(KvFilesystem::new(path.to_string()))
            }
            Err(_) => bail!("invalid url: {}", url),
        }
    }
}

impl<T> ResourceTables<dyn Resource> for KvFileSystemTables<T> where
    T: kv_filesystem::kv::Kv + 'static
{
}

impl Resource for KvAzureBlob {
    fn from_url(url: Url) -> Result<Self> {
        // get environment var STORAGE_ACCOUNT_NAME
        let storage_account_name = std::env::var("AZURE_STORAGE_ACCOUNT")?;
        // get environment var STORAGE_ACCOUNT_KEY
        let storage_account_key = std::env::var("AZURE_STORAGE_KEY")?;

        let container_name = url
            .domain()
            .expect("container name is required in the capabiloty configuration");
        Ok(KvAzureBlob::new(
            &storage_account_name,
            &storage_account_key,
            container_name,
        ))
    }
}

pub struct Context<T> {
    pub wasi: WasiCtx,
    pub config_data: config::ConfigData,
    pub data: Option<T>,
}

// load capability will return a trait object Resource and its Tables
pub fn load_capability(
    config: Vec<(String, String)>,
    linker: &mut Linker<Context<DataT>>,
) -> Result<DataT> {
    let url = &config
        .iter()
        .find(|(name, _)| name == "url")
        .expect("url is required in the capabiloty configuration")
        .1;
    let parsed = Url::parse(url)?;
    if parsed.scheme() == "azblob" {
        kv_azure_blob::add_to_linker(linker, |cx: &mut Context<DataT>| {
            let data = cx.data.as_mut().unwrap();
            let resource = data.0.as_mut().downcast_mut::<KvAzureBlob>().unwrap();
            let resource_tables = data
                .1
                .as_mut()
                .downcast_mut::<KvAzureBlobTables<KvAzureBlob>>()
                .unwrap();
            (resource, resource_tables)
        })?;
        let kv_azure_blob = KvAzureBlob::from_url(parsed)?;
        Ok((
            Box::new(kv_azure_blob),
            Box::new(KvAzureBlobTables::<KvAzureBlob>::default()),
        ))
    } else if parsed.scheme() == "file" {
        kv_filesystem::add_to_linker(linker, |cx: &mut Context<DataT>| {
            let data = cx.data.as_mut().unwrap();
            let resource = data.0.as_mut().downcast_mut::<KvFilesystem>().unwrap();
            let resource_tables = data
                .1
                .as_mut()
                .downcast_mut::<KvFileSystemTables<KvFilesystem>>()
                .unwrap();
            (resource, resource_tables)
        })?;
        let kv_filesystem = KvFilesystem::from_url(parsed)?;
        Ok((
            Box::new(kv_filesystem),
            Box::new(KvFileSystemTables::<KvFilesystem>::default()),
        ))
    } else {
        bail!("invalid url: {}", url)
    }
}

impl From<config::Error> for anyhow::Error {
    fn from(_: config::Error) -> Self {
        anyhow::anyhow!("config error")
    }
}
