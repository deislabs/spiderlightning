use anyhow::Result;
use async_trait::async_trait;

#[cfg(feature = "apache_kafka")]
pub mod apache_kafka;
#[cfg(feature = "azsbus")]
pub mod azsbus;
#[cfg(feature = "filesystem")]
pub mod filesystem;
#[cfg(feature = "mosquitto")]
pub mod mosquitto;

#[async_trait]
pub trait PubImplementor {
    async fn publish(&self, msg: &[u8], topic: &str) -> Result<()>;
}

impl std::fmt::Debug for dyn PubImplementor + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PubImplementor").finish_non_exhaustive()
    }
}

#[async_trait]
pub trait SubImplementor {
    async fn subscribe(&self, topic: &str) -> Result<String>;
    async fn receive(&self, sub_tok: &str) -> Result<Vec<u8>>;
}

impl std::fmt::Debug for dyn SubImplementor + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubImplementor").finish_non_exhaustive()
    }
}
