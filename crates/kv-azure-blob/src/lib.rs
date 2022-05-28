use anyhow::Result;
use azure_storage::core::prelude::*;
use azure_storage_blobs::prelude::*;
use futures::executor::block_on;
use runtime::resource::{Addressable};
use std::sync::Arc;
use url::Url;

pub mod azure;
/// A Azure Blob Storage binding for kv interface.
#[derive(Default, Debug)]
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

    /// Output the value of a set key.
    /// If key has not been set, return empty.
    pub fn get(&self, key: &str) -> Result<Vec<u8>> {
        let blob_client = self.inner.as_ref().unwrap().as_blob_client(key);
        match block_on(azure::get(blob_client)) {
            Ok(value) => Ok(value),
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }

    /// Create a key-value pair.
    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let blob_client = self.inner.as_ref().unwrap().as_blob_client(key);
        let value = Vec::from(value);
        match block_on(azure::set(blob_client, value)) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }

    /// Delete a key-value pair.
    pub fn delete(&self, key: &str) -> Result<()> {
        let blob_client = self.inner.as_ref().unwrap().as_blob_client(key);
        match block_on(azure::delete(blob_client)) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }
}

impl Addressable for KvAzureBlob {
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
        Ok(KvAzureBlob::new(
            &storage_account_name,
            &storage_account_key,
            container_name,
        ))
    }
}
