use anyhow::Result;
use azure_storage::core::prelude::*;
use azure_storage_blobs::prelude::*;
use runtime::resource::{Resource, Context, Linker};
use futures::executor::block_on;
use std::sync::Arc;
use url::Url;

use kv::*;

pub mod azure;

wit_bindgen_wasmtime::export!("../../wit/kv.wit");

/// A Azure Blob Storage binding for kv interface.
#[derive(Default)]
pub struct KvAzureBlob {
    inner: Option<Arc<ContainerClient>>,
}

impl KvAzureBlob {
    /// Create a new KvAzureBlob.
    pub fn new(
        storage_account_name: &str,
        storage_account_key: &str,
        container_name: &str,
    ) -> Self {
        let http_client = azure_core::new_http_client();
        let inner = Some(
            StorageAccountClient::new_access_key(
                http_client.clone(),
                storage_account_name,
                storage_account_key,
            )
            .as_container_client(container_name),
        );
        Self { inner }
    }
}

impl Resource for KvAzureBlob {
    type State = (Self, KvTables<Self>);

    fn from_url(url: Url) -> Result<Self>
    where
            Self: Sized {
        // get environment var STORAGE_ACCOUNT_NAME
        let storage_account_name = std::env::var("AZURE_STORAGE_ACCOUNT")?;
        // get environment var STORAGE_ACCOUNT_KEY
        let storage_account_key = std::env::var("AZURE_STORAGE_KEY")?;

        // container name from the domain of url. For example, if url is
        // "azblob://my-container, then the domain is "my-container".
        let container_name = url
            .domain()
            .expect("container name is required in the capability configuration");
        Ok(KvAzureBlob::new(
            &storage_account_name,
            &storage_account_key,
            container_name,
        ))
    }

    fn build_state(url: Url) -> Result<Self::State> {
        Ok((Self::from_url(url)?, Default::default()))
    }

    fn add_to_linker(
        linker: &mut Linker<Context<Self::State>>,
    ) -> Result<()> {
        kv::add_to_linker(linker, |ctx| {
            let (resource, resource_type) = Self::get_state(ctx);
            (resource, resource_type)
        })
    }
}


impl kv::Kv for KvAzureBlob {
    type ResourceDescriptor = u64;

    fn get_kv(&mut self) -> Result<Self::ResourceDescriptor, Error> {
        Ok(1)
    }

    /// Output the value of a set key.
    /// If key has not been set, return empty.
    fn get(&mut self, rd: &Self::ResourceDescriptor, key: &str) -> Result<PayloadResult, Error> {
        if *rd != 1 {
            return Err(Error::DescriptorError);
        }

        let blob_client = self.inner.as_ref().unwrap().as_blob_client(key);
        let res = block_on(azure::get(blob_client))?;
        Ok(res)
    }

    /// Create a key-value pair.
    fn set(
        &mut self,
        rd: &Self::ResourceDescriptor,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        if *rd != 1 {
            return Err(Error::DescriptorError);
        }

        let blob_client = self.inner.as_ref().unwrap().as_blob_client(key);
        let value = Vec::from(value);
        block_on(azure::set(blob_client, value))?;
        Ok(())
    }

    /// Delete a key-value pair.
    fn delete(&mut self, rd: &Self::ResourceDescriptor, key: &str) -> Result<(), Error> {
        if *rd != 1 {
            return Err(Error::DescriptorError);
        }
        let blob_client = self.inner.as_ref().unwrap().as_blob_client(key);
        block_on(azure::delete(blob_client))?;
        Ok(())
    }
}

impl From<anyhow::Error> for Error {
    fn from(_: anyhow::Error) -> Self {
        Self::OtherError
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(_: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::IoError
    }
}
