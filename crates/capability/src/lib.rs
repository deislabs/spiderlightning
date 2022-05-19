use anyhow::{bail, Result};
use as_any::{AsAny, Downcast};
use azure_blob::{blob::BlobTables as AzureBlobTables, AzureBlob};
use blob_filesystem::{blob::BlobTables as BlobFileSystemTables, BlobFilesystem};
use url::Url;
use wasi_common::WasiCtx;
use wasmtime::Linker;

wit_bindgen_wasmtime::import!("../../wit/config.wit");

type DataT = (Box<dyn Resource>, Box<dyn ResourceTables<dyn Resource>>);

/// A trait for wit-bindgen resource tables. see [here](https://github.com/bytecodealliance/wit-bindgen/blob/main/crates/wasmtime/src/table.rs) for more details:
pub trait ResourceTables<T: ?Sized>: AsAny {}

/// A trait for wit-bindgen resource.
pub trait Resource: AsAny {
    /// Given a resource url, return a resource.
    fn from_url(url: Url) -> Result<Self>
    where
        Self: Sized;
}

impl<T> ResourceTables<dyn Resource> for AzureBlobTables<T> where T: azure_blob::blob::Blob + 'static
{}

impl Resource for BlobFilesystem {
    fn from_url(url: Url) -> Result<Self> {
        let path = url.to_file_path();
        match path {
            Ok(path) => {
                let path = path.to_str().unwrap_or(".").to_string();
                Ok(BlobFilesystem::new(path))
            }
            Err(_) => bail!("invalid url: {}", url),
        }
    }
}

impl<T> ResourceTables<dyn Resource> for BlobFileSystemTables<T> where
    T: blob_filesystem::blob::Blob + 'static
{
}

impl Resource for AzureBlob {
    fn from_url(url: Url) -> Result<Self> {
        // get environment var STORAGE_ACCOUNT_NAME
        let storage_account_name = std::env::var("AZURE_STORAGE_ACCOUNT")?;
        // get environment var STORAGE_ACCOUNT_KEY
        let storage_account_key = std::env::var("AZURE_STORAGE_KEY")?;

        // container name from the domain of url. For example, if url is
        // "azblob://my-container, then the domain is "my-container".
        let container_name = url
            .domain()
            .expect("container name is required in the capability configuration");
        Ok(AzureBlob::new(
            &storage_account_name,
            &storage_account_key,
            container_name,
        ))
    }
}

/// A wasmtime runtime context to be passed to a wasm module.
pub struct Context<T> {
    pub wasi: WasiCtx,
    pub config_data: config::ConfigData,
    pub data: Option<T>,
}

/// Load capability will return a trait object Resource and its Tables
pub fn load_capability(
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

    if parsed.scheme() == "azblob" {
        azure_blob::add_to_linker(linker, |cx: &mut Context<DataT>| {
            let data = cx
                .data
                .as_mut()
                .expect("internal error: Runtime context data is None");
            let resource = data.0.as_mut().downcast_mut::<AzureBlob>().unwrap();
            let resource_tables = data
                .1
                .as_mut()
                .downcast_mut::<AzureBlobTables<AzureBlob>>()
                .unwrap();
            (resource, resource_tables)
        })?;
        let azure_blob = AzureBlob::from_url(parsed)?;
        Ok((
            Box::new(azure_blob),
            Box::new(AzureBlobTables::<AzureBlob>::default()),
        ))
    } else if parsed.scheme() == "file" {
        blob_filesystem::add_to_linker(linker, |cx: &mut Context<DataT>| {
            let data = cx
                .data
                .as_mut()
                .expect("internal error: Runtime context data is None");
            let resource = data.0.as_mut().downcast_mut::<BlobFilesystem>().unwrap();
            let resource_tables = data
                .1
                .as_mut()
                .downcast_mut::<BlobFileSystemTables<BlobFilesystem>>()
                .unwrap();
            (resource, resource_tables)
        })?;
        let blob_filesystem = BlobFilesystem::from_url(parsed)?;
        Ok((
            Box::new(blob_filesystem),
            Box::new(BlobFileSystemTables::<BlobFilesystem>::default()),
        ))
    } else {
        bail!(
            "invalid url: {}, currently wasi-cloud only supports file blob and azblob",
            url
        )
    }
}

impl From<config::Error> for anyhow::Error {
    fn from(_: config::Error) -> Self {
        anyhow::anyhow!("config error")
    }
}
