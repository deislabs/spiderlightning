use anyhow::{Context, Result};
use azure_storage::core::prelude::*;
use azure_storage_blobs::prelude::*;
use crossbeam_channel::Sender;
use events_api::Event;
use futures::executor::block_on;
use runtime::{
    impl_resource,
    resource::{
        BasicState,
        Watch,
    },
};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use kv::*;

pub mod azure;

wit_bindgen_wasmtime::export!("../../wit/kv.wit");
wit_error_rs::impl_error!(kv::Error);
wit_error_rs::impl_from!(anyhow::Error, kv::Error::ErrorWithDescription);
wit_error_rs::impl_from!(std::string::FromUtf8Error, kv::Error::ErrorWithDescription);

const SCHEME_NAME: &str = "kv.azblob";

/// A Azure Blob Storage implementation for the kv interface
#[derive(Default, Clone)]
pub struct KvAzureBlob {
    host_state: BasicState,
}

impl_resource!(
    KvAzureBlob,
    kv::KvTables<KvAzureBlob>,
    BasicState,
    SCHEME_NAME.to_string()
);

#[derive(Default, Clone, Debug)]
pub struct KvAzureBlobInner {
    container_client: Option<Arc<ContainerClient>>,
    rd: String,
}

impl Watch for KvAzureBlobInner {}

impl KvAzureBlobInner {
    pub fn new(
        storage_account_name: &str,
        storage_account_key: &str,
        container_name: &str,
        rd: String,
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
        Self {
            container_client: inner,
            rd,
        }
    }
}

impl kv::Kv for KvAzureBlob {
    type Kv = KvAzureBlobInner;
    /// Construct a new `KvAzureBlob` from a container name. For example, a container name could be "my-container"
    fn kv_open(&mut self, name: &str) -> Result<Self::Kv, kv::Error> {
        let storage_account_name = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "AZURE_STORAGE_ACCOUNT",
            &self.host_state.config_toml_file_path,
        )?)?;
        let storage_account_key = String::from_utf8(runtime_configs::providers::get(
            &self.host_state.secret_store,
            "AZURE_STORAGE_KEY",
            &self.host_state.config_toml_file_path,
        )?)?;

        let rd = Uuid::new_v4().to_string();
        let kv_azure_blob_guest = KvAzureBlobInner::new(
            &storage_account_name,
            &storage_account_key,
            name,
            rd.clone(),
        );

        self.host_state
            .resource_map
            .lock()
            .unwrap()
            .set(rd, Box::new(kv_azure_blob_guest.clone()));
        Ok(kv_azure_blob_guest)
    }

    /// Output the value of a set key
    fn kv_get(&mut self, self_: &Self::Kv, key: &str) -> Result<PayloadResult, Error> {
        let inner = self_.container_client.as_ref().unwrap();
        let blob_client = inner.as_blob_client(key);
        let res = block_on(azure::get(blob_client))
            .with_context(|| format!("failed to get value for key {}", key))?;
        Ok(res)
    }

    /// Create a key-value pair
    fn kv_set(
        &mut self,
        self_: &Self::Kv,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        let inner = self_.container_client.as_ref().unwrap();

        let blob_client = inner.as_blob_client(key);
        let value = Vec::from(value);
        block_on(azure::set(blob_client, value))
            .with_context(|| format!("failed to set value for key '{}'", key))?;
        Ok(())
    }

    /// Delete a key-value pair
    fn kv_delete(&mut self, self_: &Self::Kv, key: &str) -> Result<(), Error> {
        let inner = self_.container_client.as_ref().unwrap();
        let blob_client = inner.as_blob_client(key);
        block_on(azure::delete(blob_client)).with_context(|| "failed to delete key's value")?;
        Ok(())
    }

    /// Watch for changes to a key-value pair
    fn kv_watch(&mut self, self_: &Self::Kv, key: &str) -> Result<Observable, Error> {
        Ok(Observable {
            rd: self_.rd.clone(),
            key: key.to_string(),
        })
    }
}
