use anyhow::Result;

use async_trait::async_trait;

use crate::container::DynW;

/// A stream of bytes that can be written to
#[async_trait]
pub trait WriteStreamImplementor {
    /// Write a number of bytes to the stream
    ///
    /// This is a blocking operation that write the data byte array
    /// to the blob.
    async fn write(&self, data: &[u8]) -> Result<()>;

    /// Close the stream
    ///
    /// TODO: Not used
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
