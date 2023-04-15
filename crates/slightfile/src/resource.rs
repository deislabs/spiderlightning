use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum BlobResource {
    #[serde(rename = "blobstore.aws_s3")]
    AwsS3,
    #[serde(rename = "blobstore.azblob")]
    Azblob,
}

impl Display for BlobResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlobResource::AwsS3 => write!(f, "blobstore.aws_s3"),
            BlobResource::Azblob => write!(f, "blobstore.azblob"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum KeyvalueResource {
    #[serde(rename = "keyvalue.awsdynamodb")]
    AwsDynamoDb,
    #[serde(rename = "keyvalue.azblob")]
    Azblob,
    #[serde(rename = "keyvalue.filesystem")]
    Filesystem,
    #[serde(rename = "keyvalue.redis")]
    Redis,
    #[serde(rename = "kv.awsdynamodb")]
    V1AwsDynamoDb,
    #[serde(rename = "kv.azblob")]
    V1Azblob,
    #[serde(rename = "kv.filesystem")]
    V1Filesystem,
    #[serde(rename = "kv.redis")]
    V1Redis,
    #[serde(rename = "keyvalue.dapr")]
    Dapr,
}

impl Display for KeyvalueResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyvalueResource::AwsDynamoDb => write!(f, "keyvalue.awsdynamodb"),
            KeyvalueResource::Azblob => write!(f, "keyvalue.azblob"),
            KeyvalueResource::Filesystem => write!(f, "keyvalue.filesystem"),
            KeyvalueResource::Redis => write!(f, "keyvalue.redis"),
            KeyvalueResource::V1AwsDynamoDb => write!(f, "kv.awsdynamodb"),
            KeyvalueResource::V1Azblob => write!(f, "kv.azblob"),
            KeyvalueResource::V1Filesystem => write!(f, "kv.filesystem"),
            KeyvalueResource::V1Redis => write!(f, "kv.redis"),
            KeyvalueResource::Dapr => write!(f, "keyvalue.dapr"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum MessagingResource {
    #[serde(rename = "messaging.azsbus")]
    Azsbus,
    #[serde(rename = "messaging.confluent_apache_kafka")]
    ConfluentApacheKafka,
    #[serde(rename = "messaging.filesystem")]
    Filesystem,
    #[serde(rename = "messaging.mosquitto")]
    Mosquitto,
    #[serde(rename = "messaging.nats")]
    Nats,
    #[serde(rename = "mq.azsbus")]
    V1Azsbus,
    #[serde(rename = "mq.filesystem")]
    V1Filesystem,
}

impl Display for MessagingResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessagingResource::Azsbus => write!(f, "messaging.azsbus"),
            MessagingResource::ConfluentApacheKafka => {
                write!(f, "messaging.confluent_apache_kafka")
            }
            MessagingResource::Filesystem => write!(f, "messaging.filesystem"),
            MessagingResource::Mosquitto => write!(f, "messaging.mosquitto"),
            MessagingResource::Nats => write!(f, "messaging.nats"),
            MessagingResource::V1Azsbus => write!(f, "mq.azsbus"),
            MessagingResource::V1Filesystem => write!(f, "mq.filesystem"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash, Default)]
pub enum HttpServerResource {
    #[serde(rename = "http")]
    #[default]
    Server,
}

impl Display for HttpServerResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpServerResource::Server => write!(f, "http"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum HttpClientResource {
    #[serde(rename = "http-client")]
    Client,
}

impl Display for HttpClientResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpClientResource::Client => write!(f, "http-client"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum ConfigsResource {
    #[serde(rename = "configs.azapp")]
    Azapp,
    #[serde(rename = "configs.envvars")]
    Envvars,
    #[serde(rename = "configs.usersecrets")]
    Usersecrets,
}

impl Display for ConfigsResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigsResource::Azapp => write!(f, "configs.azapp"),
            ConfigsResource::Envvars => write!(f, "configs.envvars"),
            ConfigsResource::Usersecrets => write!(f, "configs.usersecrets"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum DistributedLockingResource {
    #[serde(rename = "distributed_locking.etcd")]
    Etcd,
    #[serde(rename = "lockd.etcd")]
    V1Etcd,
}

impl Display for DistributedLockingResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DistributedLockingResource::Etcd => {
                write!(f, "distributed_locking.etcd")
            }
            DistributedLockingResource::V1Etcd => write!(f, "lockd.etcd"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum SqlResource {
    #[serde(rename = "sql.postgres")]
    Postgres,
}

impl Display for SqlResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SqlResource::Postgres => write!(f, "sql.postgres"),
        }
    }
}

/// All the resources that slightfile supports. This is used in the
/// `Capability` section in slightfile to specify what resource a
/// capability is for.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
#[serde(untagged)]
pub enum Resource {
    Blob(BlobResource),
    Keyvalue(KeyvalueResource),
    Messaging(MessagingResource),
    HttpServer(HttpServerResource),
    HttpClient(HttpClientResource),
    Configs(ConfigsResource),
    DistributedLocking(DistributedLockingResource),
    Sql(SqlResource),
}

impl Default for Resource {
    fn default() -> Self {
        Resource::HttpServer(HttpServerResource::Server)
    }
}

impl Resource {
    pub fn to_cap_name(&self) -> String {
        match self {
            Resource::Blob(_) => "blob".into(),
            Resource::Keyvalue(_) => "keyvalue".into(),
            Resource::Messaging(_) => "messaging".into(),
            Resource::HttpServer(_) => "http".into(),
            Resource::HttpClient(_) => "http-client".into(),
            Resource::Configs(_) => "configs".into(),
            Resource::DistributedLocking(_) => "distributed_locking".into(),
            Resource::Sql(_) => "sql".into(),
        }
    }
}

impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Resource::Blob(blob) => write!(f, "{blob}"),
            Resource::Keyvalue(keyvalue) => write!(f, "{keyvalue}"),
            Resource::Messaging(messaging) => write!(f, "{messaging}"),
            Resource::HttpServer(http_server) => write!(f, "{http_server}"),
            Resource::HttpClient(http_client) => write!(f, "{http_client}"),
            Resource::Configs(configs) => write!(f, "{configs}"),
            Resource::DistributedLocking(distributed_locking) => {
                write!(f, "{distributed_locking}")
            }
            Resource::Sql(sql_postgres) => write!(f, "{sql_postgres}"),
        }
    }
}
