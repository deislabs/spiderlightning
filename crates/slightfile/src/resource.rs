use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// All the resources that slightfile supports. This is used in the
/// `Capability` section in slightfile to specify what resource a
/// capability is for.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Resource {
    #[serde(rename = "blobstore.aws_s3")]
    BlobstoreAwsS3,
    #[serde(rename = "blobstore.azblob")]
    BlobstoreAzblob,
    #[serde(rename = "keyvalue.awsdynamodb")]
    KeyvalueAwsDynamoDb,
    #[serde(rename = "keyvalue.azblob")]
    KeyvalueAzblob,
    #[serde(rename = "keyvalue.filesystem")]
    KeyvalueFilesystem,
    #[serde(rename = "keyvalue.redis")]
    KeyvalueRedis,
    #[serde(rename = "kv.awsdynamodb")]
    V1KeyvalueAwsDynamoDb,
    #[serde(rename = "kv.azblob")]
    V1KeyvalueAzblob,
    #[serde(rename = "kv.filesystem")]
    V1KeyvalueFilesystem,
    #[serde(rename = "kv.redis")]
    V1KeyvalueRedis,
    #[serde(rename = "keyvalue.dapr")]
    KeyvalueDapr,
    #[serde(rename = "messaging.azsbus")]
    MessagingAzsbus,
    #[serde(rename = "messaging.confluent_apache_kafka")]
    MessagingConfluentApacheKafka,
    #[serde(rename = "messaging.filesystem")]
    MessagingFilesystem,
    #[serde(rename = "messaging.mosquitto")]
    MessagingMosquitto,
    #[serde(rename = "mq.azsbus")]
    V1MessagingAzsbus,
    #[serde(rename = "mq.filesystem")]
    V1MessagingFilesystem,
    #[serde(rename = "messaging.nats")]
    MessagingNats,
    #[serde(rename = "http")] // TODO: change this to http-server and bump up slightfile version?
    HttpServer,
    #[serde(rename = "http-client")]
    HttpClient,
    #[serde(rename = "configs.azapp")]
    ConfigsAzapp,
    #[serde(rename = "configs.envvars")]
    ConfigsEnvvars,
    #[serde(rename = "configs.usersecrets")]
    ConfigsUsersecrets,
    #[serde(rename = "distributed_locking.etcd")]
    DistributedLockingEtcd,
    #[serde(rename = "lockd.etcd")]
    V1DistributedLockingEtcd,
    #[serde(rename = "sql.postgres")]
    SqlPostgres,
}

impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Resource::BlobstoreAwsS3 => write!(f, "blobstore.aws_s3"),
            Resource::BlobstoreAzblob => write!(f, "blobstore.azblob"),
            Resource::KeyvalueAwsDynamoDb => write!(f, "keyvalue.awsdynamodb"),
            Resource::KeyvalueAzblob => write!(f, "keyvalue.azblob"),
            Resource::KeyvalueFilesystem => write!(f, "keyvalue.filesystem"),
            Resource::KeyvalueRedis => write!(f, "keyvalue.redis"),
            Resource::KeyvalueDapr => write!(f, "keyvalue.dapr"),
            Resource::MessagingAzsbus => write!(f, "messaging.azsbus"),
            Resource::MessagingConfluentApacheKafka => {
                write!(f, "messaging.confluent_apache_kafka")
            }
            Resource::MessagingFilesystem => write!(f, "messaging.filesystem"),
            Resource::MessagingMosquitto => write!(f, "messaging.mosquitto"),
            Resource::MessagingNats => write!(f, "messaging.nats"),
            Resource::HttpServer => write!(f, "http"),
            Resource::HttpClient => write!(f, "http-client"),
            Resource::ConfigsAzapp => write!(f, "configs.azapp"),
            Resource::ConfigsEnvvars => write!(f, "configs.envvars"),
            Resource::ConfigsUsersecrets => write!(f, "configs.usersecrets"),
            Resource::DistributedLockingEtcd => write!(f, "distributed_locking.etcd"),
            Resource::SqlPostgres => write!(f, "sql.postgres"),
            Resource::V1KeyvalueAwsDynamoDb => write!(f, "kv.awsdynamodb"),
            Resource::V1KeyvalueAzblob => write!(f, "kv.azblob"),
            Resource::V1KeyvalueFilesystem => write!(f, "kv.filesystem"),
            Resource::V1KeyvalueRedis => write!(f, "kv.redis"),
            Resource::V1MessagingAzsbus => write!(f, "mq.azsbus"),
            Resource::V1MessagingFilesystem => write!(f, "mq.filesystem"),
            Resource::V1DistributedLockingEtcd => write!(f, "lockd.etcd"),
        }
    }
}
