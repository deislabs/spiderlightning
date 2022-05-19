use azure_storage::core::prelude::*;
use azure_storage_blobs::prelude::*;
use futures::executor::block_on;
use std::sync::Arc;

pub use blob::add_to_linker;
use blob::*;

pub mod azure;

wit_bindgen_wasmtime::export!("../../wit/blob.wit");

/// A Azure Blob Storage binding for blob interface.
#[derive(Default)]
pub struct AzureBlob {
    inner: Option<Arc<ContainerClient>>,
}

impl AzureBlob {
    /// Create a new AzureBlob.
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

impl blob::Blob for AzureBlob {
    type ResourceDescriptor = u64;

    fn get_blob(&mut self) -> Result<Self::ResourceDescriptor, Error> {
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
