use std::sync::Arc;

use anyhow::{Context, Result};
use azure_storage::clients::StorageAccountClient;
use azure_storage_blobs::prelude::{AsBlobClient, AsContainerClient, ContainerClient};
use futures::executor::block_on;
use runtime::resource::BasicState;

use crate::clouds::azure;

/// This is the underlying struct behind the `AzBlob` variant of the `KvProvider` enum.
///
/// It provides a properties that pertains solely to the azblob implementation
/// of this capability:
///     - `container_client`, and
///
/// As per its' usage in `KvProvider`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
pub struct AzBlobProvider {
    container_client: Option<Arc<ContainerClient>>,
}

impl AzBlobProvider {
    pub fn new(slight_state: &BasicState, name: &str) -> Self {
        let storage_account_name = String::from_utf8(
            runtime_configs::providers::get(
                &slight_state.secret_store,
                "AZURE_STORAGE_ACCOUNT",
                &slight_state.config_toml_file_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'AZURE_STORAGE_ACCOUNT' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();
        let storage_account_key = String::from_utf8(
            runtime_configs::providers::get(
                &slight_state.secret_store,
                "AZURE_STORAGE_KEY",
                &slight_state.config_toml_file_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'AZURE_STORAGE_KEY' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();

        let http_client = azure_core::new_http_client();
        let container_client = Some(
            StorageAccountClient::new_access_key(
                http_client.clone(),
                storage_account_name,
                storage_account_key,
            )
            .as_container_client(name),
        );
        Self { container_client }
    }

    pub fn get(&self, key: &str) -> Result<Vec<u8>> {
        let inner = self.container_client.as_ref().unwrap();
        let blob_client = inner.as_blob_client(key);
        let res = block_on(azure::get(blob_client))
            .with_context(|| format!("failed to get value for key {}", key))?;
        Ok(res)
    }

    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let inner = self.container_client.as_ref().unwrap();

        let blob_client = inner.as_blob_client(key);
        let value = Vec::from(value);
        block_on(azure::set(blob_client, value))
            .with_context(|| format!("failed to set value for key '{}'", key))?;
        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        let inner = &self.container_client.as_ref().unwrap();
        let blob_client = inner.as_blob_client(key);
        block_on(azure::delete(blob_client)).with_context(|| "failed to delete key's value")?;
        Ok(())
    }
}
