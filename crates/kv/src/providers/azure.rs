use anyhow::Result;
use azure_storage_blobs::prelude::{Blob, BlobClient, ContainerClient, DeleteSnapshotsMethod};
use futures::stream::StreamExt;

/// Get the value given a `blob_client`
pub async fn get(blob_client: BlobClient) -> Result<Vec<u8>> {
    let mut stream = blob_client.get().chunk_size(128u64).into_stream();
    let mut result = vec![];
    // The stream is composed of individual calls to the get blob endpoint
    while let Some(value) = stream.next().await {
        let mut body = value?.data;
        // For each response, we stream the body instead of collecting it all into one large allocation.
        while let Some(value) = body.next().await {
            let value = value?;
            println!("received {:?} bytes", value.len());
            result.extend(&value);
        }
    }
    Ok(result)
}

/// Set the value given a `blob_client` and `value`
pub async fn set(blob_client: BlobClient, value: Vec<u8>) -> Result<()> {
    blob_client
        .put_block_blob(value)
        .content_type("text/plain")
        .into_future()
        .await?;
    Ok(())
}

/// Delete the `value` given a `blob_client`
pub async fn delete(blob_client: BlobClient) -> Result<()> {
    blob_client
        .delete()
        .delete_snapshots_method(DeleteSnapshotsMethod::Include)
        .into_future()
        .await?;
    Ok(())
}

pub async fn list_blobs(container_client: ContainerClient) -> Result<Vec<Blob>> {
    let mut stream = container_client.list_blobs().into_stream();
    let mut results = vec![];
    while let Some(value) = stream.next().await {
        let value = value?;
        results.push(value);
    }

    let mut result = vec![];
    for list_blob in results {
        for blob in list_blob.blobs.blobs {
            result.push(blob);
        }
    }
    Ok(result)
}
