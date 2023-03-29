use anyhow::Result;

use crate::container::DynR;
use async_trait::async_trait;
use std::fmt::Debug;

/// A stream of bytes that can be read from
#[async_trait]
pub trait ReadStreamImplementor {
    /// Read a number of bytes from the stream
    /// 
    /// Returns `None` if the stream has reached the end
    /// Otherwise returns a `Vec<u8>` of the bytes read
    async fn read(&self, size: u64) -> Result<Option<Vec<u8>>>;

    /// Returns the number of bytes available to read
    /// 
    /// TODO: This is not implemented for all implementors 
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
