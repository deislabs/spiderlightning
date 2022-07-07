use anyhow::{Context, Result};
use azure_storage::core::prelude::*;
use azure_storage_blobs::prelude::*;
use crossbeam_channel::Sender;
use futures::executor::block_on;
use proc_macro_utils::{Resource, RuntimeResource};
use runtime::resource::{
    get, Ctx, DataT, Linker, Map, Resource, ResourceMap, RuntimeResource,
};
use events_api::Event;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use kv::*;

pub mod azure;

wit_bindgen_wasmtime::export!("../../wit/kv.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);

const SCHEME_NAME: &str = "azblobkv";

/// A Azure Blob Storage implementation for the kv interface
#[derive(Default, Clone, Resource, RuntimeResource)]
pub struct KvAzureBlob {
    inner: Option<Arc<ContainerClient>>,
    resource_map: Option<ResourceMap>,
}

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
            resource_map: None,
        }
    }
}

impl kv::Kv for KvAzureBlob {
    /// Construct a new `KvAzureBlob` from a container name. For example, a container name could be "my-container"
    fn get_kv(&mut self, name: &str) -> Result<ResourceDescriptorResult, Error> {
        let storage_account_name = std::env::var("AZURE_STORAGE_ACCOUNT")
            .with_context(|| "failed to read AZURE_STORAGE_ACCOUNT environment variable")?;
        let storage_account_key = std::env::var("AZURE_STORAGE_KEY")
            .with_context(|| "failed to read AZURE_STORAGE_KEY environment variable")?;

        let kv_azure_blob = KvAzureBlob::new(&storage_account_name, &storage_account_key, name);
        self.inner = kv_azure_blob.inner;

        let rd = Uuid::new_v4().to_string();
        let cloned = self.clone(); // have to clone here because of the mutable borrow below
        let mut map = Map::lock(&mut self.resource_map)?;
        map.set(rd.clone(), (Box::new(cloned), None));
        Ok(rd)
    }

    /// Output the value of a set key
    fn get(&mut self, rd: ResourceDescriptorParam, key: &str) -> Result<PayloadResult, Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.resource_map)?;
        let inner = map.get::<Arc<ContainerClient>>(rd)?;
        let blob_client = inner.as_blob_client(key);
        let res = block_on(azure::get(blob_client))
            .with_context(|| format!("failed to get value for key {}", key))?;
        Ok(res)
    }

    /// Create a key-value pair
    fn set(
        &mut self,
        rd: ResourceDescriptorParam,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.resource_map)?;
        let inner = map.get::<Arc<ContainerClient>>(rd)?;
        let blob_client = inner.as_blob_client(key);
        let value = Vec::from(value);
        block_on(azure::set(blob_client, value))
            .with_context(|| format!("failed to set value for key {}", key))?;
        Ok(())
    }

    /// Delete a key-value pair
    fn delete(&mut self, rd: ResourceDescriptorParam, key: &str) -> Result<(), Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.resource_map)?;
        let inner = map.get::<Arc<ContainerClient>>(rd)?;

        let blob_client = inner.as_blob_client(key);
        block_on(azure::delete(blob_client)).with_context(|| "failed to delete key's value")?;
        Ok(())
    }

    fn watch(&mut self, rd: ResourceDescriptorParam, key: &str) -> Result<Observable, Error> {
        Ok(Observable {
            rd: rd.to_string(),
            key: key.to_string(),
        })
    }
}
