use anyhow::Result;
use azure_storage_blobs::prelude::BlobClient;
use bytes::Bytes;
use std::sync::Arc;

/// get the value given blob_client
pub async fn get(blob_client: Arc<BlobClient>) -> Result<Vec<u8>> {
    let res = blob_client
        .get()
        .execute()
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(Bytes::from(res.data.to_vec()).to_vec())
}

/// set the value given blob_client and value
pub async fn set(blob_client: Arc<BlobClient>, value: Vec<u8>) -> Result<()> {
    blob_client
        .put_block_blob(value)
        .content_type("text/plain")
        .execute()
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(())
}

/// delete the value given blob_client
pub async fn delete(blob_client: Arc<BlobClient>) -> Result<()> {
    blob_client
        .delete()
        .execute()
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(())
}
