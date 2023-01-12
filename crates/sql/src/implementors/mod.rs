use anyhow::Result;
use async_trait::async_trait;

use crate::sql::RowItem;

#[cfg(feature = "postgres")]
pub mod postgres;

#[async_trait]
pub trait SqlImplementor {
    async fn query(&self, query: &str) -> Result<Vec<RowItem>>;
    async fn exec(&self, query: &str) -> Result<()>;
}

impl std::fmt::Debug for dyn SqlImplementor + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqlImplementor").finish_non_exhaustive()
    }
}
