use anyhow::{Context, Result};
use azure_storage::core::prelude::*;
use azure_storage_blobs::prelude::*;
use crossbeam_channel::Sender;
use events_api::Event;
use futures::executor::block_on;
use proc_macro_utils::Resource;
use runtime::{
    impl_resource,
    resource::{
        get_table, Ctx, DataT, Linker, Map, Resource, BasicState, ResourceTables, RuntimeResource,
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

const SCHEME_NAME: &str = "azblobkv";

/// A Azure Blob Storage implementation for the kv interface
#[derive(Default, Clone, Resource)]
pub struct KvAzureBlob {
    inner: Option<Arc<ContainerClient>>,
    host_state: Option<BasicState>,
}

impl_resource!(
    KvAzureBlob,
    kv::KvTables<KvAzureBlob>,
    BasicState,
    SCHEME_NAME.to_string()
);

impl KvAzureBlob {
    /// Create a new `KvAzureBlob`
    fn new(storage_account_name: &str, storage_account_key: &str, container_name: &str) -> Self {
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
            inner,
            host_state: None,
        }
    }
}

impl kv::Kv for KvAzureBlob {
    type Kv = String;
    /// Construct a new `KvAzureBlob` from a container name. For example, a container name could be "my-container"
    fn kv_open(&mut self, name: &str) -> Result<Self::Kv, kv::Error> {
        let storage_account_name = String::from_utf8(configs::providers::get(
            &self.host_state.as_ref().unwrap().secret_store,
            "AZURE_STORAGE_ACCOUNT",
            &self.host_state.as_ref().unwrap().config_toml_file_path,
        )?)?;
        let storage_account_key = String::from_utf8(configs::providers::get(
            &self.host_state.as_ref().unwrap().secret_store,
            "AZURE_STORAGE_KEY",
            &self.host_state.as_ref().unwrap().config_toml_file_path,
        )?)?;

        let kv_azure_blob = KvAzureBlob::new(&storage_account_name, &storage_account_key, name);
        self.inner = kv_azure_blob.inner;

        let rd = Uuid::new_v4().to_string();
        let cloned = self.clone(); // have to clone here because of the mutable borrow below
        let mut map = Map::lock(&mut self.host_state.as_mut().unwrap().resource_map)?;
        map.set(rd.clone(), (Box::new(cloned), None));
        Ok(rd)
    }

    /// Output the value of a set key
    fn kv_get(&mut self, self_: &Self::Kv, key: &str) -> Result<kv::PayloadResult, kv::Error> {
        Uuid::parse_str(self_).with_context(|| {
            "internal kv::error: failed to parse internal handle to this resource"
        })?;

        let map = Map::lock(&mut self.host_state.as_mut().unwrap().resource_map)?;
        let inner = map.get::<Arc<ContainerClient>>(self_)?;
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
        value: kv::PayloadParam<'_>,
    ) -> Result<(), kv::Error> {
        Uuid::parse_str(self_).with_context(|| {
            "internal kv::error: failed to parse internal handle to this resource"
        })?;

        let map = Map::lock(&mut self.host_state.as_mut().unwrap().resource_map)?;
        let inner = map.get::<Arc<ContainerClient>>(self_)?;
        let blob_client = inner.as_blob_client(key);
        let value = Vec::from(value);
        block_on(azure::set(blob_client, value))
            .with_context(|| format!("failed to set value for key '{}'", key))?;
        Ok(())
    }

    /// Delete a key-value pair
    fn kv_delete(&mut self, self_: &Self::Kv, key: &str) -> Result<(), kv::Error> {
        Uuid::parse_str(self_).with_context(|| {
            "internal kv::error: failed to parse internal handle to this resource"
        })?;

        let map = Map::lock(&mut self.host_state.as_mut().unwrap().resource_map)?;
        let inner = map.get::<Arc<ContainerClient>>(self_)?;

        let blob_client = inner.as_blob_client(key);
        block_on(azure::delete(blob_client)).with_context(|| "failed to delete key's value")?;
        Ok(())
    }

    fn kv_watch(&mut self, self_: &Self::Kv, key: &str) -> Result<Observable, kv::Error> {
        Ok(Observable {
            rd: self_.to_string(),
            key: key.to_string(),
        })
    }
}
