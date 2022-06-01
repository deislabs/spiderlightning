use anyhow::{bail, Result};
use kv_azure_blob::KvAzureBlob;
use kv_filesystem::KvFilesystem;
use runtime::resource::{get, Context, DataT, HostResource, Linker, Resource, ResourceTables, Addressable};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};
use url::Url;

use kv::*;

wit_bindgen_wasmtime::export!("../../wit/kv.wit");

#[derive(Debug)]
pub enum ResourceDescriptor {
    AzureBlob(KvAzureBlob),
    Filesystem(KvFilesystem),
}

pub struct KvDispatcher {}

impl kv::Kv for KvDispatcher {
    type ResourceDescriptor = ResourceDescriptor;

    fn get_kv(&mut self, url: &str) -> Result<Self::ResourceDescriptor, Error> {
        let parsed = Url::parse(url).unwrap();
        match parsed.scheme() {
            "azblob" => Ok(ResourceDescriptor::AzureBlob(KvAzureBlob::from_url(parsed)?)),
            "file" => Ok(ResourceDescriptor::Filesystem(KvFilesystem::from_url(parsed)?)),
            _ => {
                println!("invalid url: {}, currently wasi-cloud kv interface only supports 'file', 'azblob'", parsed);
                Err(Error::OtherError)
            }
        }
    }

    /// Output the value of a set key.
    /// If key has not been set, return empty.
    fn get(&mut self, rd: &Self::ResourceDescriptor, key: &str) -> Result<PayloadResult, Error> {
        match rd {
            ResourceDescriptor::AzureBlob(kv_azure_blob) => {
                let res = kv_azure_blob.get(key)?;
                Ok(res)
            }
            ResourceDescriptor::Filesystem(kv_filesystem) => {
                let res = kv_filesystem.get(key)?;
                Ok(res)
            }
        }
    }

    /// Create a key-value pair.
    fn set(
        &mut self,
        rd: &Self::ResourceDescriptor,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        match rd {
            ResourceDescriptor::AzureBlob(kv_azure_blob) => {
                kv_azure_blob.set(key, value)?;
                Ok(())
            }
            ResourceDescriptor::Filesystem(kv_filesystem) => {
                kv_filesystem.set(key, value)?;
                Ok(())
            }
        }
    }

    /// Delete a key-value pair.
    fn delete(&mut self, rd: &Self::ResourceDescriptor, key: &str) -> Result<(), Error> {
        match rd {
            ResourceDescriptor::AzureBlob(kv_azure_blob) => {
                kv_azure_blob.delete(key)?;
                Ok(())
            }
            ResourceDescriptor::Filesystem(kv_filesystem) => {
                kv_filesystem.delete(key)?;
                Ok(())
            }
        }
    }
}

impl Resource for KvDispatcher {}

impl<T> ResourceTables<dyn Resource> for KvTables<T> where T: Kv + 'static {}

impl HostResource for KvDispatcher {
    fn add_to_linker(linker: &mut Linker<Context<DataT>>) -> Result<()> {
        crate::add_to_linker(linker, get::<Self, KvTables<Self>>)
    }

    fn build_data() -> Result<DataT> {
        let kv_dispatcher = KvDispatcher {};
        Ok((
            Box::new(kv_dispatcher),
            Box::new(KvTables::<Self>::default()),
        ))
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::OtherError
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError
    }
}
