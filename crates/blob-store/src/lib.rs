mod container;
mod data_blob;
mod data_blob_writer;
mod implementors;
mod read_stream;
mod write_stream;
use std::{collections::HashMap, fmt::Debug};

use async_trait::async_trait;

use slight_common::{impl_resource, BasicState};

use blob_store::*;
wit_bindgen_wasmtime::export!({paths: ["../../wit/blob-store.wit"], async: *});
wit_error_rs::impl_error!(blob_store::Error);
wit_error_rs::impl_from!(anyhow::Error, blob_store::Error::UnexpectedError);

#[derive(Clone, Default)]
pub struct BlobStore {
    implementor: String,
    capability_store: HashMap<String, BasicState>,
}

impl BlobStore {
    pub fn new(implementor: String, keyvalue_store: HashMap<String, BasicState>) -> Self {
        Self {
            implementor,
            capability_store: keyvalue_store,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum BlobStoreImplementors {
    #[cfg(feature = "aws_s3")]
    S3,
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

impl_resource!(
    BlobStore,
    blob_store::BlobStoreTables<BlobStore>,
    blob_store::add_to_linker,
    "blobstore".to_string()
);

#[async_trait]
impl blob_store::BlobStore for BlobStore {
    type Container = ();
    type DataBlob = ();
    type DataBlobWriter = ();
    type ReadStream = ();
    type WriteStream = ();
    async fn container_name(&mut self, self_: &Self::Container) -> Result<String, Error> {
        todo!()
    }
    async fn container_info(
        &mut self,
        self_: &Self::Container,
    ) -> Result<ContainerMetadata, Error> {
        todo!()
    }
    async fn container_read_object(
        &mut self,
        self_: &Self::Container,
        name: ObjectNameParam<'_>,
    ) -> Result<Self::ReadStream, Error> {
        todo!()
    }
    async fn container_write_object(
        &mut self,
        self_: &Self::Container,
        name: ObjectNameParam<'_>,
    ) -> Result<Self::WriteStream, Error> {
        todo!()
    }
    async fn container_get_data(
        &mut self,
        self_: &Self::Container,
        name: ObjectNameParam<'_>,
        start: u64,
        end: u64,
    ) -> Result<Self::DataBlob, Error> {
        todo!()
    }
    async fn container_write_data(
        &mut self,
        self_: &Self::Container,
        name: ObjectNameParam<'_>,
        data: &Self::DataBlob,
    ) -> Result<(), Error> {
        todo!()
    }
    async fn container_list_objects(
        &mut self,
        self_: &Self::Container,
        name: ObjectNameParam<'_>,
    ) -> Result<Vec<ObjectNameResult>, Error> {
        todo!()
    }
    async fn container_delete_object(
        &mut self,
        self_: &Self::Container,
        name: ObjectNameParam<'_>,
    ) -> Result<(), Error> {
        todo!()
    }
    async fn container_delete_objects(
        &mut self,
        self_: &Self::Container,
        names: Vec<ObjectNameParam<'_>>,
    ) -> Result<(), Error> {
        todo!()
    }
    async fn container_has_object(
        &mut self,
        self_: &Self::Container,
        name: ObjectNameParam<'_>,
    ) -> Result<bool, Error> {
        todo!()
    }
    async fn container_object_info(
        &mut self,
        self_: &Self::Container,
        name: ObjectNameParam<'_>,
    ) -> Result<ObjectMetadata, Error> {
        todo!()
    }
    async fn container_clear(&mut self, self_: &Self::Container) -> Result<(), Error> {
        todo!()
    }
    async fn write_stream_write(
        &mut self,
        self_: &Self::WriteStream,
        data: &[u8],
    ) -> Result<(), Error> {
        todo!()
    }
    async fn write_stream_close(&mut self, self_: &Self::WriteStream) -> Result<(), Error> {
        todo!()
    }
    async fn read_stream_read_into(
        &mut self,
        self_: &Self::ReadStream,
        ref_: &[u8],
    ) -> Result<Option<u64>, Error> {
        todo!()
    }
    async fn read_stream_available(&mut self, self_: &Self::ReadStream) -> Result<u64, Error> {
        todo!()
    }
    async fn data_blob_create(&mut self, self_: &Self::DataBlob) -> Self::DataBlobWriter {
        todo!()
    }
    async fn data_blob_read(&mut self, self_: &Self::DataBlob) -> Result<Self::ReadStream, Error> {
        todo!()
    }
    async fn data_blob_size(&mut self, self_: &Self::DataBlob) -> Result<u64, Error> {
        todo!()
    }
    async fn data_blob_writer_write(
        &mut self,
        self_: &Self::DataBlobWriter,
        data: &[u8],
    ) -> Result<(), Error> {
        todo!()
    }
    async fn data_blob_writer_finalize(
        &mut self,
        self_: &Self::DataBlobWriter,
    ) -> Result<Self::DataBlob, Error> {
        todo!()
    }
}
