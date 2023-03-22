use std::sync::Arc;

use async_trait::async_trait;
use slight_common::BasicState;

use crate::{
    blob_store::{ContainerMetadata, Error, ObjectMetadata, ObjectNameParam, ObjectNameResult},
    read_stream::ReadStreamImplementor,
    write_stream::WriteStreamImplementor,
    BlobStoreImplementors,
};

pub(crate) type DynW = dyn WriteStreamImplementor + Send + Sync;
pub(crate) type DynR = dyn ReadStreamImplementor + Send + Sync;
pub(crate) type DynContainer = dyn ContainerImplementor + Send + Sync;

#[async_trait]
pub trait ContainerImplementor {
    async fn name(&self) -> Result<String, Error>;
    async fn info(&self) -> Result<ContainerMetadata, Error>;
    async fn list_objects(&self, name: ObjectNameParam<'_>)
        -> Result<Vec<ObjectNameResult>, Error>;
    async fn delete_object(&self, name: ObjectNameParam<'_>) -> Result<(), Error>;
    async fn delete_objects(&self, names: Vec<ObjectNameParam<'_>>) -> Result<(), Error>;
    async fn has_object(&self, name: ObjectNameParam<'_>) -> Result<bool, Error>;
    async fn object_info(&self, name: ObjectNameParam<'_>) -> Result<ObjectMetadata, Error>;
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
    ) -> Self {
        Self {
            implementor: match blobstore_implementor {
                #[cfg(feature = "aws_s3")]
                BlobStoreImplementors::S3 => {
                    todo!()
                }
                BlobStoreImplementors::None => {
                    panic!("No implementor specified")
                }
            },
        }
    }
}
