use anyhow::{Context, Result};
use async_trait::async_trait;
use azure_storage::prelude::*;
use azure_storage_blobs::{container::operations::BlobItem, prelude::*};
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;
use tracing::log;

use crate::providers::azure;

use super::KeyvalueImplementor;

/// This is the underlying struct behind the `AzBlob` variant of the `KeyvalueImplementor` enum.
///
/// It provides a property that pertains solely to the azblob implementation
/// of this capability:
///     - `container_client`
///
/// As per its' usage in `KeyvalueImplementor`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
pub struct AzBlobImplementor {
    container_client: ContainerClient,
}

impl AzBlobImplementor {
    pub async fn new(slight_state: &BasicState, name: &str) -> Self {
        let storage_account_name = get_from_state("AZURE_STORAGE_ACCOUNT", slight_state)
            .await
            .unwrap();
        let storage_account_key = get_from_state("AZURE_STORAGE_KEY", slight_state)
            .await
            .unwrap();

        let storage_credentials =
            StorageCredentials::Key(storage_account_name.clone(), storage_account_key);
        let service_client = BlobServiceClient::new(storage_account_name, storage_credentials);

        let container_client = service_client.container_client(name);
        Self { container_client }
    }
}

#[async_trait]
impl KeyvalueImplementor for AzBlobImplementor {
    async fn get(&self, key: &str) -> Result<Vec<u8>> {
        let blob_client = self.container_client.blob_client(key);
        let res = azure::get(blob_client)
            .await
            .with_context(|| format!("failed to get value for key {}", key))?;
        Ok(res)
    }

    async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let blob_client = self.container_client.blob_client(key);
        let value = Vec::from(value);
        azure::set(blob_client, value)
            .await
            .with_context(|| format!("failed to set value for key '{}'", key))?;
        Ok(())
    }

    async fn keys(&self) -> Result<Vec<String>> {
        let blobs = azure::list_blobs(self.container_client.clone())
            .await
            .with_context(|| "failed to list blobs")?;
        log::debug!("found blobs: {:?}", blobs);
        let keys = blobs
            .iter()
            .map(|blob| match blob {
                BlobItem::Blob(b) => b.name.clone(),
                BlobItem::BlobPrefix(b) => b.name.clone(),
            })
            .collect::<Vec<String>>();
        Ok(keys)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let blob_client = self.container_client.blob_client(key);
        azure::delete(blob_client)
            .await
            .with_context(|| "failed to delete key's value")?;
        Ok(())
    }
}
