use anyhow::Result;

use async_trait::async_trait;

use crate::container::DynW;

#[async_trait]
pub trait WriteStreamImplementor {
    async fn write(&self, data: &[u8]) -> Result<()>;
    async fn close(&self) -> Result<()>;
}

impl std::fmt::Debug for dyn WriteStreamImplementor + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriteStreamImplementor")
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct WriteStreamInner {
    pub implementor: Box<DynW>,
}

impl WriteStreamInner {
    pub async fn new(implementor: Box<DynW>) -> Self {
        Self { implementor }
    }
}
