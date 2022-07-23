use anyhow::{Context, Result};
use azure_storage::core::prelude::*;
use azure_storage_blobs::prelude::*;
use crossbeam_channel::Sender;
use events_api::Event;
use futures::executor::block_on;
use proc_macro_utils::{Resource, Watch};
use runtime::{
    impl_resource,
    resource::{
        get_table, BasicState, Ctx, HostState, Linker, Resource, ResourceBuilder, ResourceTables,
        Watch,
    },
};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use state_store::*;

pub mod azure;

wit_bindgen_wasmtime::export!("../../wit/state_store.wit");
wit_error_rs::impl_error!(state_store::Error);
wit_error_rs::impl_from!(anyhow::Error, state_store::Error::ErrorWithDescription);
wit_error_rs::impl_from!(
    std::string::FromUtf8Error,
    state_store::Error::ErrorWithDescription
);

const SCHEME_NAME: &str = "state_store.azblob";

/// A Azure Blob Storage implementation for the state_store interface
#[derive(Default, Clone, Resource)]
pub struct StateStoreAzureBlob {
    host_state: BasicState,
}

impl_resource!(
    StateStoreAzureBlob,
    state_store::StateStoreTables<StateStoreAzureBlob>,
    BasicState,
    SCHEME_NAME.to_string()
);

#[derive(Default, Clone, Debug, Watch)]
pub struct StateStoreAzureBlobInner {
    container_client: Option<Arc<ContainerClient>>,
    rd: String,
}

impl StateStoreAzureBlobInner {
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

impl state_store::StateStore for StateStoreAzureBlob {
    type StateStore = StateStoreAzureBlobInner;
    /// Construct a new `StateStoreAzureBlob` from a container name. For example, a container name could be "my-container"
    fn state_store_open(&mut self, name: &str) -> Result<Self::StateStore, state_store::Error> {
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
        let state_store_azure_blob_guest = StateStoreAzureBlobInner::new(
            &storage_account_name,
            &storage_account_key,
            name,
            rd.clone(),
        );

        self.host_state
            .resource_map
            .lock()
            .unwrap()
            .set(rd, Box::new(state_store_azure_blob_guest.clone()));
        Ok(state_store_azure_blob_guest)
    }

    /// Output the value of a set key
    fn state_store_get(
        &mut self,
        self_: &Self::StateStore,
        key: &str,
    ) -> Result<PayloadResult, Error> {
        let inner = self_.container_client.as_ref().unwrap();
        let blob_client = inner.as_blob_client(key);
        let res = block_on(azure::get(blob_client))
            .with_context(|| format!("failed to get value for key {}", key))?;
        Ok(res)
    }

    /// Create a key-value pair
    fn state_store_set(
        &mut self,
        self_: &Self::StateStore,
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
    fn state_store_delete(&mut self, self_: &Self::StateStore, key: &str) -> Result<(), Error> {
        let inner = self_.container_client.as_ref().unwrap();
        let blob_client = inner.as_blob_client(key);
        block_on(azure::delete(blob_client)).with_context(|| "failed to delete key's value")?;
        Ok(())
    }

    /// Watch for changes to a key-value pair
    fn state_store_watch(
        &mut self,
        self_: &Self::StateStore,
        key: &str,
    ) -> Result<Observable, Error> {
        Ok(Observable {
            rd: self_.rd.clone(),
            key: key.to_string(),
        })
    }
}
