

use anyhow::{bail, Result};
use async_trait::async_trait;
use azure_storage::prelude::*;
use azure_storage_blobs::{
    container::{operations::BlobItem, Container},
    prelude::*,
};
use futures::StreamExt;
use slight_common::BasicState;

use slight_runtime_configs::get_from_state;
use tracing::info;

use crate::{
    blob_store::{ContainerMetadata, ObjectMetadata, ObjectNameParam, ObjectNameResult},
    container::ContainerImplementor,
    read_stream::{ReadStreamImplementor, ReadStreamInner},
    write_stream::{WriteStreamImplementor, WriteStreamInner},
};

pub const AZBLOB_CAPABILITY_NAME: &str = "blobstore.azblob";

/// A container maps to a bucket in azure blob storage
#[derive(Debug, Clone)]
pub struct AzBlobContainer {
    client: ContainerClient,
}

#[derive(Debug)]
pub struct AzBlobReadStream {
    blob_client: BlobClient,
}

#[derive(Debug, Clone)]
pub struct AzBlobWriteStream {
    client: BlobClient,
}

impl AzBlobContainer {
    pub async fn new(slight_state: &BasicState, name: &str) -> Result<Self> {
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
        if container_client.exists().await? {
            Ok(Self {
                client: container_client,
            })
        } else {
            bail!(format!("container {name} not found"))
        }
    }
}

#[async_trait]
impl ContainerImplementor for AzBlobContainer {
    async fn name(&self) -> Result<String> {
        Ok(self.client.container_name().to_owned())
    }
    async fn info(&self) -> Result<ContainerMetadata> {
        let properties = self.client.get_properties().await?;
        Ok(properties.container.into())
    }
    async fn list_objects(&self) -> Result<Vec<ObjectNameResult>> {
        let mut stream = self.client.list_blobs().into_stream();
        let mut results = vec![];
        while let Some(value) = stream.next().await {
            let value = value?;
            results.push(value);
        }

        let mut result = vec![];
        for list_blob in results {
            for blob in list_blob.blobs.items {
                let blob_name = match blob {
                    BlobItem::Blob(b) => b.name.clone(),
                    BlobItem::BlobPrefix(b) => b.name.clone(),
                };
                result.push(blob_name)
            }
        }
        Ok(result)
    }
    async fn delete_object(&self, name: ObjectNameParam<'_>) -> Result<()> {
        self.client
            .blob_client(name)
            .delete()
            .delete_snapshots_method(DeleteSnapshotsMethod::Include)
            .into_future()
            .await?;
        Ok(())
    }
    async fn delete_objects(&self, _names: Vec<ObjectNameParam<'_>>) -> Result<()> {
        // TODO: there isn't an API in azure blob storage to do this directly
        // if we are going to delete a lot of objects, we should use a batch delete
        // otherwise, we run into issues with deleting half the objects and then
        // failing
        //
        //
        // followed up on https://github.com/Azure/azure-sdk-for-rust/issues/1249
        todo!()
    }
    async fn has_object(&self, name: ObjectNameParam<'_>) -> Result<bool> {
        let res = self.client.blob_client(name).exists().await?;
        Ok(res)
    }
    async fn object_info(&self, name: ObjectNameParam<'_>) -> Result<ObjectMetadata> {
        let blob = self.client.blob_client(name).get_properties().await?.blob;
        Ok(ObjectMetadata {
            name: blob.name,
            container: self.name().await?,
            created_at: blob.properties.creation_time.unix_timestamp() as u64,
            size: blob.properties.content_length,
        })
    }
    async fn read_object(&self, name: ObjectNameParam<'_>) -> Result<ReadStreamInner> {
        let client = self.client.blob_client(name);
        if client.exists().await? {
            info!("found blob {name}");
            let read_stream_inner =
                ReadStreamInner::new(Box::new(AzBlobReadStream::new(client.clone()).await)).await;
            Ok(read_stream_inner)
        } else {
            bail!(format!("blob {name} not found"))
        }
    }
    async fn write_object(&self, name: ObjectNameParam<'_>) -> Result<WriteStreamInner> {
        // unlike read-object, there is no need for write-object to check if the object exists
        // this is because the write-stream will create the object if it doesn't exist or
        // overwrite it if it does
        let write_stream_inner = WriteStreamInner::new(Box::new(
            AzBlobWriteStream::new(self.client.blob_client(name).clone()).await,
        ))
        .await;
        Ok(write_stream_inner)
    }
}

impl AzBlobReadStream {
    pub async fn new(blob_client: BlobClient) -> Self {
        Self { blob_client }
    }
}

impl AzBlobWriteStream {
    pub async fn new(client: BlobClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ReadStreamImplementor for AzBlobReadStream {
    async fn read(&self, size: u64) -> Result<Option<Vec<u8>>> {
        let mut size = size as usize;
        let mut stream = self.blob_client.get().chunk_size(128u64).into_stream();
        let mut result = vec![];
        // The stream is composed of individual calls to the get blob endpoint
        while let Some(value) = stream.next().await {
            let mut body = value?.data;
            // For each response, we stream the body instead of collecting it all into one large allocation.
            // We use take to limit the number of bytes read from the body
            while let Some(value) = (&mut body).take(size).next().await {
                let value = value?;
                result.extend(&value);
                // reduce the size by the number of bytes read
                size -= value.len();
                // if size is zero, we break out of the loop
                if size == 0 {
                    break;
                }
            }
        }
        Ok(Some(result))
    }
    async fn available(&self) -> Result<u64> {
        todo!()
    }
}

#[async_trait]
impl WriteStreamImplementor for AzBlobWriteStream {
    async fn write(&self, data: &[u8]) -> Result<()> {
        let exists = self.client.exists().await?;
        if !exists {
            self.client.put_append_blob().into_future().await?;
        }
        self.client
            .append_block(data.to_vec())
            .into_future()
            .await?;
        Ok(())
    }
    async fn close(&self) -> Result<()> {
        todo!()
    }
}

impl From<Container> for ContainerMetadata {
    fn from(container: Container) -> Self {
        // TODO: the last-modified timestamp is not the same as created_at
        // there is no other APIs exposed by azure blob storage to get
        // the container's creation time
        Self {
            name: container.name,
            created_at: container.last_modified.unix_timestamp() as u64,
        }
    }
}
