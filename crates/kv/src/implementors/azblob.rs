use anyhow::{Context, Result};
use azure_storage::clients::StorageClient;
use azure_storage_blobs::prelude::{AsContainerClient, ContainerClient};
use futures::executor::block_on;
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;

use crate::providers::azure;

/// This is the underlying struct behind the `AzBlob` variant of the `KvImplementor` enum.
///
/// It provides a property that pertains solely to the azblob implementation
/// of this capability:
///     - `container_client`
///
/// As per its' usage in `KvImplementor`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
pub struct AzBlobImplementor {
    container_client: ContainerClient,
}

impl AzBlobImplementor {
    pub fn new(slight_state: &BasicState, name: &str) -> Self {
        let storage_account_name = get_from_state("AZURE_STORAGE_ACCOUNT", slight_state).unwrap();
        let storage_account_key = get_from_state("AZURE_STORAGE_KEY", slight_state).unwrap();

        let container_client =
            StorageClient::new_access_key(storage_account_name, storage_account_key)
                .container_client(name);
        Self { container_client }
    }

    pub fn get(&self, key: &str) -> Result<Vec<u8>> {
        let blob_client = self.container_client.blob_client(key);
        let res = block_on(azure::get(blob_client))
            .with_context(|| format!("failed to get value for key {}", key))?;
        Ok(res)
    }

    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let blob_client = self.container_client.blob_client(key);
        let value = Vec::from(value);
        block_on(azure::set(blob_client, value))
            .with_context(|| format!("failed to set value for key '{}'", key))?;
        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        let blob_client = self.container_client.blob_client(key);
        block_on(azure::delete(blob_client)).with_context(|| "failed to delete key's value")?;
        Ok(())
    }
}
