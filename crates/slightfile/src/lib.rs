use anyhow::{bail, Result};
use std::{collections::HashMap, fmt::Display, path::Path};

use serde::{Deserialize, Serialize};

/// slightfile version.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SpecVersion {
    /// Version 0.1 format.
    #[serde(rename = "0.1")]
    V1,
    /// Version 0.2 format.
    #[serde(rename = "0.2")]
    V2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TomlFile {
    pub specversion: SpecVersion,
    pub secret_store: Option<String>,
    pub secret_settings: Option<Vec<Config>>,
    pub capability: Option<Vec<Capability>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Capability {
    V1(CapabilityV1),
    V2(CapabilityV2),
}

impl Capability {
    pub fn is_v1(&self) -> bool {
        match self {
            Capability::V1(_) => true,
            _ => false,
        }
    }
    pub fn is_v2(&self) -> bool {
        match self {
            Capability::V2(_) => true,
            _ => false,
        }
    }
    pub fn resource(&self) -> Resource {
        match self {
            Capability::V1(c) => c.name.clone(),
            Capability::V2(c) => c.resource.clone(),
        }
    }
    pub fn name(&self) -> String {
        match self {
            Capability::V1(c) => c.name.to_string(),
            Capability::V2(c) => c.name.clone(),
        }
    }
    pub fn configs(&self) -> Option<HashMap<String, String>> {
        match self {
            Capability::V1(_) => None,
            Capability::V2(c) => c.configs.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityV1 {
    pub name: Resource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityV2 {
    pub resource: Resource,
    pub name: String,
    pub configs: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub value: String,
}

impl Config {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(rename = "distributed_locking.etcd")]
    DistributedLockingEtcd,
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
            Resource::DistributedLockingEtcd => write!(f, "distributed_locking.etcd"),
            Resource::SqlPostgres => write!(f, "sql.postgres"),
        }
    }
}

pub fn read_as_toml_file(path: impl AsRef<Path>) -> Result<TomlFile> {
    let toml_file_contents = std::fs::read_to_string(path.as_ref())?;
    let toml = toml::from_str::<TomlFile>(&toml_file_contents)?;
    // check specversion
    match &toml.specversion {
        SpecVersion::V1 => {
            if toml.capability.as_ref().is_some() {
                if toml
                    .capability
                    .as_ref()
                    .unwrap()
                    .iter()
                    .any(|cap| cap.is_v2())
                {
                    bail!("Error: you are using a 0.1 specversion, but you are using a 0.2 capability format");
                }
            }
        }
        SpecVersion::V2 => {
            if toml.capability.as_ref().is_some() {
                if toml
                    .capability
                    .as_ref()
                    .unwrap()
                    .iter()
                    .any(|cap| cap.is_v1())
                {
                    bail!("Error: you are using a 0.2 specversion, but you are using a 0.1 capability format");
                }
            }
        }
    };
    Ok(toml)
}

pub fn has_http_cap(toml: &TomlFile) -> bool {
    if let Some(capability) = &toml.capability {
        capability.iter().any(|cap| match cap {
            Capability::V1(cap) => match cap.name {
                Resource::HttpServer => true,
                _ => false,
            },
            Capability::V2(cap) => match cap.resource {
                Resource::HttpServer => true,
                _ => false,
            },
        })
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use anyhow::bail;

    use super::*;

    #[test]
    fn test_good_read_as_toml_file() -> Result<()> {
        let path = format!("{}/tests/good", env!("CARGO_MANIFEST_DIR"));

        // read all files with .toml on `path` directory
        let toml_files = std::fs::read_dir(path)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().unwrap() == "toml")
            .map(|entry| entry.path())
            .map(|path| (read_as_toml_file(path.clone()), path))
            .collect::<Vec<_>>();

        // assert all files are valid
        for (toml_file, p) in toml_files {
            if let Err(e) = toml_file {
                bail!("Error: {:?} for path: {:?}", e, p);
            }
        }

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_bad_read_as_toml_file() {
        let path = format!("{}/tests/bad", env!("CARGO_MANIFEST_DIR"));

        // read all files with .toml on `path` directory
        let toml_files = std::fs::read_dir(path)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().unwrap() == "toml")
            .map(|entry| entry.path())
            .map(|path| (read_as_toml_file(path.clone()), path))
            .collect::<Vec<_>>();

        // all files should panic
        let mut should_panic: bool = true;
        for (toml_file, p) in toml_files {
            // run in a closure
            let f = || -> Result<()> {
                if let Err(e) = toml_file {
                    bail!("Error: {:?} for path: {:?}", e, p);
                }
                Ok(())
            }();
            if f.is_ok() {
                should_panic = false;
            }
        }

        if should_panic {
            panic!("Error: all files should panic");
        }
    }

    #[test]
    fn resource_to_str() {
        let azblob = Resource::BlobstoreAzblob;
        assert_eq!(azblob.to_string(), "blobstore.azblob");
    }
}
