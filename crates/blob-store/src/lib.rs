mod container;
mod implementors;
mod read_stream;
mod write_stream;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use async_trait::async_trait;

use container::ContainerInner;
use read_stream::ReadStreamInner;
use slight_common::{impl_resource, BasicState};

use blob_store::*;
use write_stream::WriteStreamInner;
wit_bindgen_wasmtime::export!({paths: ["../../wit/blob-store.wit"], async: *});
wit_error_rs::impl_error!(blob_store::Error);
wit_error_rs::impl_from!(anyhow::Error, blob_store::Error::UnexpectedError);

#[derive(Clone, Default)]
pub struct BlobStore {
    implementor: BlobStoreImplementors,
    capability_store: HashMap<String, BasicState>,
}

impl BlobStore {
    pub fn new(implementor: String, keyvalue_store: HashMap<String, BasicState>) -> Self {
        Self {
            implementor: implementor.as_str().into(),
            capability_store: keyvalue_store,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Default)]
pub enum BlobStoreImplementors {
    #[cfg(feature = "aws_s3")]
    S3,
    #[default]
    None,
}

impl From<&str> for BlobStoreImplementors {
    fn from(s: &str) -> Self {
        match s {
            #[cfg(feature = "aws_s3")]
            "blobstore.aws_s3" => Self::S3,
            p => panic!(
                "failed to match provided name (i.e., '{p}') to any known host implementations"
            ),
        }
    }
}

impl Display for BlobStoreImplementors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "aws_s3")]
            Self::S3 => write!(f, "blobstore.aws_s3"),
            Self::None => panic!("No implementor specified"),
        }
    }
}

impl_resource!(
    BlobStore,
    blob_store::BlobStoreTables<BlobStore>,
    blob_store::add_to_linker,
    "blobstore".to_string()
);

impl BlobStore {
    fn fetch_state(&mut self, name: &str) -> BasicState {
        let s: String = self.implementor.to_string();
        let state = if let Some(r) = self.capability_store.get(name) {
            r.clone()
        } else if let Some(r) = self.capability_store.get(&s) {
            r.clone()
        } else {
            panic!(
                "could not find capability under name '{}' for implementor '{}'",
                name, &s
            );
        };

        state
    }
}

#[async_trait]
impl blob_store::BlobStore for BlobStore {
    type Container = ContainerInner;
    type ReadStream = ReadStreamInner;
    type WriteStream = WriteStreamInner;

    async fn container_open(&mut self, name: &str) -> Result<Self::Container, Error> {
        let state = self.fetch_state(name);
        tracing::log::info!("Opening implementor {}", &state.implementor);
        let inner = Self::Container::new(state.implementor.as_str().into(), &state, name).await?;

        Ok(inner)
    }

    async fn container_name(&mut self, self_: &Self::Container) -> Result<String, Error> {
        Ok(self_.implementor.name().await?)
    }
    async fn container_info(
        &mut self,
        self_: &Self::Container,
    ) -> Result<ContainerMetadata, Error> {
        Ok(self_.implementor.info().await?)
    }
    async fn container_read_object(
        &mut self,
        self_: &Self::Container,
        name: ObjectNameParam<'_>,
    ) -> Result<Self::ReadStream, Error> {
        let read_stream = self_.implementor.read_object(name).await?;
        Ok(read_stream)
    }
    async fn container_write_object(
        &mut self,
        self_: &Self::Container,
        name: ObjectNameParam<'_>,
    ) -> Result<Self::WriteStream, Error> {
        let write_stream = self_.implementor.write_object(name).await?;
        Ok(write_stream)
    }
    async fn container_list_objects(
        &mut self,
        self_: &Self::Container,
        name: ObjectNameParam<'_>,
    ) -> Result<Vec<ObjectNameResult>, Error> {
        Ok(self_.implementor.list_objects(name).await?)
    }
    async fn container_delete_object(
        &mut self,
        self_: &Self::Container,
        name: ObjectNameParam<'_>,
    ) -> Result<(), Error> {
        Ok(self_.implementor.delete_object(name).await?)
    }
    async fn container_delete_objects(
        &mut self,
        self_: &Self::Container,
        names: Vec<ObjectNameParam<'_>>,
    ) -> Result<(), Error> {
        Ok(self_.implementor.delete_objects(names).await?)
    }
    async fn container_has_object(
        &mut self,
        self_: &Self::Container,
        name: ObjectNameParam<'_>,
    ) -> Result<bool, Error> {
        Ok(self_.implementor.has_object(name).await?)
    }
    async fn container_object_info(
        &mut self,
        self_: &Self::Container,
        name: ObjectNameParam<'_>,
    ) -> Result<ObjectMetadata, Error> {
        Ok(self_.implementor.object_info(name).await?)
    }
    async fn container_clear(&mut self, _self_: &Self::Container) -> Result<(), Error> {
        todo!()
    }
    async fn write_stream_write(
        &mut self,
        self_: &Self::WriteStream,
        data: &[u8],
    ) -> Result<(), Error> {
        Ok(self_.implementor.write(data).await?)
    }
    async fn write_stream_close(&mut self, self_: &Self::WriteStream) -> Result<(), Error> {
        Ok(self_.implementor.close().await?)
    }
    async fn read_stream_read(
        &mut self,
        self_: &Self::ReadStream,
        size: u64,
    ) -> Result<Option<Vec<u8>>, Error> {
        Ok(self_.implementor.read(size).await?)
    }
    async fn read_stream_available(&mut self, self_: &Self::ReadStream) -> Result<u64, Error> {
        Ok(self_.implementor.available().await?)
    }
}
