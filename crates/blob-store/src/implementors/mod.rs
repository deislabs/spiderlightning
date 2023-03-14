use anyhow::Result;
use async_trait::async_trait;

#[cfg(feature = "s3")]
pub mod aws_s3;

#[async_trait]
pub trait BlobStoreImplementor {
    async fn get(&self, key: &str) -> Result<Vec<u8>>;
    async fn set(&self, key: &str, value: &[u8]) -> Result<()>;
    async fn keys(&self) -> Result<Vec<String>>;
    async fn delete(&self, key: &str) -> Result<()>;
}

impl std::fmt::Debug for dyn BlobStoreImplementor + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobStoreImplementor")
            .finish_non_exhaustive()
    }
}
