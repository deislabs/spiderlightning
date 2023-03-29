use anyhow::Result;
use async_trait::async_trait;

#[cfg(feature = "awsdynamodb")]
pub mod awsdynamodb;
#[cfg(feature = "azblob")]
pub mod azblob;
#[cfg(feature = "filesystem")]
pub mod filesystem;
#[cfg(feature = "redis")]
pub mod redis;
#[cfg(feature = "dapr")]
pub mod dapr;

#[async_trait]
pub trait KeyvalueImplementor {
    async fn get(&self, key: &str) -> Result<Vec<u8>>;
    async fn set(&self, key: &str, value: &[u8]) -> Result<()>;
    async fn keys(&self) -> Result<Vec<String>>;
    async fn delete(&self, key: &str) -> Result<()>;
}

impl std::fmt::Debug for dyn KeyvalueImplementor + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyvalueImplementor")
            .finish_non_exhaustive()
    }
}
