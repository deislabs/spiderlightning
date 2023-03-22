use std::sync::Arc;

use async_trait::async_trait;

use crate::{blob_store::Error, container::{DynW, DynContainer}};

#[async_trait]
pub trait WriteStreamImplementor {
    async fn write(
        &self,
        data: &[u8],
    ) -> Result<(), Error>;
    async fn close(&self) -> Result<(), Error>;
}

impl std::fmt::Debug for dyn WriteStreamImplementor + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriteStreamImplementor").finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct WriteStreamInner {
    pub implementor: Arc<DynW>,
}

impl WriteStreamInner {
    pub async fn new(
        container_implementor: Arc<DynContainer>,
        name: &str,
    ) -> Self {
        Self {
            implementor: todo!()
        }
    }
}