use anyhow::Result;

use crate::container::DynR;
use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait]
pub trait ReadStreamImplementor {
    async fn read(&self, size: u64) -> Result<Option<Vec<u8>>>;
    async fn available(&self) -> Result<u64>;
}

impl Debug for dyn ReadStreamImplementor + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadStreamImplementor")
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct ReadStreamInner {
    pub implementor: Box<DynR>,
}

impl ReadStreamInner {
    pub async fn new(implementor: Box<DynR>) -> Self {
        Self { implementor }
    }
}
