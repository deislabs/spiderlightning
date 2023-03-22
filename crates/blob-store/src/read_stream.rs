use std::sync::Arc;

use async_trait::async_trait;

use crate::{blob_store::Error, container::{DynR, DynContainer}};

#[async_trait]
pub trait ReadStreamImplementor {
    async fn read_into(
        &self,
        ref_: &[u8],
    ) -> Result<Option<u64>, Error>;
    async fn available(&self) -> Result<u64, Error>;
}

impl std::fmt::Debug for dyn ReadStreamImplementor + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadStreamImplementor").finish_non_exhaustive()
    }
}


#[derive(Clone, Debug)]
pub struct ReadStreamInner {
    pub implementor: Arc<DynR>,
}

impl ReadStreamInner {
    pub async fn new(
        container_implementor: Arc<DynContainer>,
        name: &str,
    ) -> Self {
        Self {
            implementor: todo!(),
        }
    }
}