use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use aws_config::{from_env, meta::region::RegionProviderChain};
use aws_sdk_s3::{
    client::fluent_builders::GetObject,
    error::{GetObjectError, GetObjectErrorKind},
    model::{Bucket, Delete, ObjectIdentifier},
    types::ByteStream,
    Client,
};
use slight_common::BasicState;

use tracing::info;

use crate::{
    blob_store::{ContainerMetadata, ObjectMetadata, ObjectNameParam, ObjectNameResult},
    container::ContainerImplementor,
    read_stream::{ReadStreamImplementor, ReadStreamInner},
    write_stream::{WriteStreamImplementor, WriteStreamInner},
};

pub const S3_CAPABILITY_NAME: &str = "blobstore.aws_s3";

/// A container maps to a bucket in aws S3
#[derive(Debug, Clone)]
pub struct S3Container {
    client: Arc<Client>,
    bucket: Bucket,
}

/// A read stream maps to a GetObject request
///
/// To use this stream, you must call `send` on it.
#[derive(Debug)]
pub struct S3ReadStream {
    req: GetObject,
}

/// A write stream contains a S3 client, the bucket name and a key
#[derive(Debug, Clone)]
pub struct S3WriteStream {
    client: Arc<Client>,
    bucket: String,
    key: String,
}

impl S3Container {
    pub async fn new(_slight_state: &BasicState, name: &str) -> Result<Self> {
        let region = RegionProviderChain::default_provider().or_else("us-west-2");
        let config = from_env().region(region).load().await;
        let client = Arc::new(Client::new(&config));

        // perform list buckets, too costly?
        let resp = client.list_buckets().send().await?;
        let buckets = resp.buckets().unwrap_or_default();
        let bucket = buckets
            .iter()
            .find(|b| b.name().unwrap_or_default() == name)
            .ok_or(anyhow::anyhow!(format!("container {name} not found")))?
            .clone();

        Ok(Self { client, bucket })
    }
}

#[async_trait]
impl ContainerImplementor for S3Container {
    async fn name(&self) -> Result<String> {
        Ok(self.bucket.name().unwrap_or_default().to_string())
    }
    async fn info(&self) -> Result<ContainerMetadata> {
        Ok(ContainerMetadata::from(&self.bucket))
    }
    async fn list_objects(&self) -> Result<Vec<ObjectNameResult>> {
        let resp = self
            .client
            .list_objects_v2()
            .bucket(self.name().await?)
            .send()
            .await?;
        info!("{}", "received list objects response");
        let res = resp
            .contents()
            .unwrap_or_default()
            .iter()
            .map(|object| object.key().unwrap_or_default().to_string())
            .collect();
        Ok(res)
    }
    async fn delete_object(&self, name: ObjectNameParam<'_>) -> Result<()> {
        let _ = self
            .client
            .delete_object()
            .bucket(self.name().await?)
            .key(name)
            .send()
            .await?;
        info!("{}", format!("object {name} deleted"));
        Ok(())
    }
    async fn delete_objects(&self, names: Vec<ObjectNameParam<'_>>) -> Result<()> {
        let mut delete_objects: Vec<ObjectIdentifier> = vec![];
        for name in names {
            let obj_id = ObjectIdentifier::builder()
                .set_key(Some(name.to_owned()))
                .build();
            delete_objects.push(obj_id);
        }
        let delete = Delete::builder().set_objects(Some(delete_objects)).build();
        let _ = self
            .client
            .delete_objects()
            .bucket(self.name().await?)
            .delete(delete)
            .send()
            .await?;
        info!("{}", "objects deleted");
        Ok(())
    }
    async fn has_object(&self, name: ObjectNameParam<'_>) -> Result<bool> {
        let res = self
            .client
            .get_object()
            .bucket(self.name().await?)
            .key(name)
            .send()
            .await;
        let mut key_exists: bool = true;
        if let Err(err) = res {
            match err.into_service_error() {
                GetObjectError {
                    kind: GetObjectErrorKind::NoSuchKey(_),
                    ..
                } => key_exists = false,
                err => return Err(err.into()),
            }
        }
        Ok(key_exists)
    }
    async fn object_info(&self, name: ObjectNameParam<'_>) -> Result<ObjectMetadata> {
        let container = self.name().await?;
        let metadata = self
            .client
            .get_object_attributes()
            .bucket(container.clone())
            .key(name)
            .object_attributes(aws_sdk_s3::model::ObjectAttributes::ObjectSize)
            .send()
            .await?;
        let res = ObjectMetadata {
            name: name.to_owned(),
            container,
            created_at: metadata.last_modified().unwrap().as_secs_f64() as u64, // TODO: fix me
            size: metadata.object_size() as u64,
        };
        Ok(res)
    }
    async fn read_object(&self, name: ObjectNameParam<'_>) -> Result<ReadStreamInner> {
        let resp = self
            .client
            .get_object()
            .bucket(self.name().await?)
            .key(name);
        let read_stream_inner = ReadStreamInner::new(Box::new(S3ReadStream::new(resp).await)).await;
        Ok(read_stream_inner)
    }
    async fn write_object(&self, name: ObjectNameParam<'_>) -> Result<WriteStreamInner> {
        let write_stream_inner = WriteStreamInner::new(Box::new(
            S3WriteStream::new(self.client.clone(), self.bucket.name().unwrap(), name).await,
        ))
        .await;
        Ok(write_stream_inner)
    }
}

impl S3ReadStream {
    pub async fn new(req: GetObject) -> Self {
        Self { req }
    }
}

impl S3WriteStream {
    pub async fn new(client: Arc<Client>, bucket: &str, key: &str) -> Self {
        Self {
            client,
            bucket: bucket.into(),
            key: key.into(),
        }
    }
}

#[async_trait]
impl ReadStreamImplementor for S3ReadStream {
    async fn read(&self, size: u64) -> Result<Option<Vec<u8>>> {
        // In wasi-blob-store, `read` takes a mutable buffer as an argument.
        // I changed it to return a vector of bytes instead because as of right now,
        // wit-bindgen does not support generating mutable buffers.
        //
        // This is something we might want to go back and change in the future
        // when we transform wit-bindgen v0.2.0 to the newest component model syntax.
        //
        // TODO: change `read` to take a mutable buffer as an argument
        let resp = self.req.clone().send().await?;
        let content_length = resp.content_length() as u64;
        let stream: ByteStream = resp.body;
        if size == 0 {
            Ok(Some(vec![]))
        } else if size > content_length {
            Ok(Some(stream.collect().await?.to_vec()))
        } else {
            let mut res = stream.collect().await?.to_vec();
            res.truncate(size as usize);
            Ok(Some(res))
        }
    }
    async fn available(&self) -> Result<u64> {
        todo!()
    }
}

#[async_trait]
impl WriteStreamImplementor for S3WriteStream {
    async fn write(&self, data: &[u8]) -> Result<()> {
        // TODO: same comment from `read` applies here
        let _ = self
            .client
            .put_object()
            .bucket(self.bucket.clone())
            .key(self.key.clone())
            .body(ByteStream::from(data.to_vec()))
            .send()
            .await?;
        Ok(())
    }
    async fn close(&self) -> Result<()> {
        todo!()
    }
}

impl From<&Bucket> for ContainerMetadata {
    fn from(bucket: &Bucket) -> Self {
        let created_at = if let Some(creation_date) = bucket.creation_date() {
            creation_date.secs() as u64
        } else {
            0_u64
        };
        Self {
            name: bucket.name().unwrap_or_default().into(),
            created_at,
        }
    }
}
