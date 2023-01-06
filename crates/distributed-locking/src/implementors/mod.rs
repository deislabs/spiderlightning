use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;

#[cfg(feature = "etcd")]
pub mod etcd;

#[async_trait]
pub trait DistributedLockingImplementor {
    async fn lock(&self, lock_name: &[u8]) -> Result<Vec<u8>>;
    async fn lock_with_time_to_live(
        &self,
        lock_name: &[u8],
        time_to_live_in_secs: i64,
    ) -> Result<Vec<u8>>;
    async fn unlock(&self, lock_key: &[u8]) -> Result<()>;
}

impl Debug for dyn DistributedLockingImplementor + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DistributedLockingImplementor")
            .finish_non_exhaustive()
    }
}
