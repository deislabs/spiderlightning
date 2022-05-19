use azure_storage_blobs::prelude::BlobClient;
use bytes::Bytes;
use std::{error::Error, result, sync::Arc};

pub type Result<T> = result::Result<T, Box<dyn Error + Send + Sync>>;

pub async fn get(blob_client: Arc<BlobClient>) -> Result<Vec<u8>> {
    let res = blob_client.get().execute().await?;
    Ok(Bytes::from(res.data.to_vec()).to_vec())
}

pub async fn set(blob_client: Arc<BlobClient>, value: Vec<u8>) -> Result<()> {
    blob_client
        .put_block_blob(value)
        .content_type("text/plain")
        .execute()
        .await?;
    Ok(())
}

pub async fn delete(blob_client: Arc<BlobClient>) -> Result<()> {
    blob_client.delete().execute().await?;
    Ok(())
}
