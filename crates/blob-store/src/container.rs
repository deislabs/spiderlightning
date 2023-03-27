use anyhow::Result;

use std::sync::Arc;

use async_trait::async_trait;
use slight_common::BasicState;

use crate::{
    blob_store::{ContainerMetadata, ObjectMetadata, ObjectNameParam, ObjectNameResult},
    implementors::{aws_s3::S3Container, azblob::AzBlobContainer},
    read_stream::{ReadStreamImplementor, ReadStreamInner},
    write_stream::{WriteStreamImplementor, WriteStreamInner},
    BlobStoreImplementors,
};

pub(crate) type DynW = dyn WriteStreamImplementor + Send + Sync;
pub(crate) type DynR = dyn ReadStreamImplementor + Send + Sync;
pub(crate) type DynContainer = dyn ContainerImplementor + Send + Sync;

#[async_trait]
pub trait ContainerImplementor {
    async fn name(&self) -> Result<String>;
    async fn info(&self) -> Result<ContainerMetadata>;
    async fn list_objects(&self) -> Result<Vec<ObjectNameResult>>;
    async fn delete_object(&self, name: ObjectNameParam<'_>) -> Result<()>;
    async fn delete_objects(&self, names: Vec<ObjectNameParam<'_>>) -> Result<()>;
    async fn has_object(&self, name: ObjectNameParam<'_>) -> Result<bool>;
    async fn object_info(&self, name: ObjectNameParam<'_>) -> Result<ObjectMetadata>;
    async fn read_object(&self, name: ObjectNameParam<'_>) -> Result<ReadStreamInner>;
    async fn write_object(&self, name: ObjectNameParam<'_>) -> Result<WriteStreamInner>;
}

impl std::fmt::Debug for DynContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContainerImplementor")
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct ContainerInner {
    pub implementor: Arc<DynContainer>,
}

impl ContainerInner {
    pub(crate) async fn new(
        blobstore_implementor: BlobStoreImplementors,
        slight_state: &BasicState,
        name: &str,
    ) -> Result<Self> {
        let container = Self {
            implementor: match blobstore_implementor {
                #[cfg(feature = "aws_s3")]
                BlobStoreImplementors::S3 => Arc::new(S3Container::new(slight_state, name).await?),
                #[cfg(feature = "azblob")]
                BlobStoreImplementors::AzBlob => {
                    Arc::new(AzBlobContainer::new(slight_state, name).await?)
                }
                BlobStoreImplementors::None => {
                    panic!("No implementor specified")
                }
            },
        };
        Ok(container)
    }
}
