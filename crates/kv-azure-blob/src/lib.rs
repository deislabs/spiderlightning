use anyhow::{Context, Result};
use azure_storage::core::prelude::*;
use azure_storage_blobs::prelude::*;
use futures::executor::block_on;
use runtime::resource::{
    get, Context as RuntimeContext, DataT, HostResource, Linker, Resource,
    ResourceMap,
};
use std::sync::Arc;
use uuid::Uuid;

use kv::*;

pub mod azure;

wit_bindgen_wasmtime::export!("../../wit/kv.wit");

const SCHEME_NAME: &str = "azblob";

/// A Azure Blob Storage binding for kv interface.
#[derive(Default, Clone)]
pub struct KvAzureBlob {
    inner: Option<Arc<ContainerClient>>,
    resource_map: Option<ResourceMap>,
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
        Self {
            inner,
            resource_map: None,
        }
    }
}

impl Resource for KvAzureBlob {
    fn add_resource_map(&mut self, resource_map: ResourceMap) -> Result<()> {
        self.resource_map = Some(resource_map);
        Ok(())
    }

    fn get_inner(&self) -> &dyn std::any::Any {
        let inner = self.inner.as_ref().unwrap();
        inner
    }
}

impl HostResource for KvAzureBlob {
    fn add_to_linker(linker: &mut Linker<RuntimeContext<DataT>>) -> Result<()> {
        crate::add_to_linker(linker, |cx| get::<Self>(cx, SCHEME_NAME.to_string()))
    }

    fn build_data() -> Result<DataT> {
        let kv_azure_blob = Self::default();
        Ok(Box::new(kv_azure_blob))
    }
}

impl kv::Kv for KvAzureBlob {
    /// Construct a new KvAzureBlob from container name. For example, A container name could be "my-container".
    fn get_kv(&mut self, name: &str) -> Result<ResourceDescriptorResult, Error> {
        // get environment var STORAGE_ACCOUNT_NAME
        let storage_account_name = std::env::var("AZURE_STORAGE_ACCOUNT")
            .context("AZURE_STORAGE_ACCOUNT environment variable not found")?;
        // get environment var STORAGE_ACCOUNT_KEY
        let storage_account_key = std::env::var("AZURE_STORAGE_KEY")
            .context("AZURE_STORAGE_KEY environment variable not found")?;

        let kv_azure_blob = KvAzureBlob::new(&storage_account_name, &storage_account_key, name);
        self.inner = kv_azure_blob.inner;

        let uuid = Uuid::new_v4();
        let rd = uuid.to_string();
        let cloned = self.clone();
        let mut map = self
            .resource_map
            .as_mut()
            .ok_or(anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        map.set(rd.clone(), Box::new(cloned))?;
        Ok(rd)
    }

    /// Output the value of a set key.
    /// If key has not been set, return empty.
    fn get(&mut self, rd: ResourceDescriptorParam, key: &str) -> Result<PayloadResult, Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }

        let map = self
            .resource_map
            .as_mut()
            .ok_or(anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        let inner = map.get::<Arc<ContainerClient>>(rd)?;

        let blob_client = inner.as_blob_client(key);
        let res = block_on(azure::get(blob_client))?;
        Ok(res)
    }

    /// Create a key-value pair.
    fn set(
        &mut self,
        rd: ResourceDescriptorParam,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }

        let map = self
            .resource_map
            .as_mut()
            .ok_or(anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        let inner = map.get::<Arc<ContainerClient>>(rd)?;

        let blob_client = inner.as_blob_client(key);
        let value = Vec::from(value);
        block_on(azure::set(blob_client, value))?;
        Ok(())
    }

    /// Delete a key-value pair.
    fn delete(&mut self, rd: ResourceDescriptorParam, key: &str) -> Result<(), Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }

        let map = self
            .resource_map
            .as_mut()
            .ok_or(anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        let inner = map.get::<Arc<ContainerClient>>(rd)?;

        let blob_client = inner.as_blob_client(key);
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
