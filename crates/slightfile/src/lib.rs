use anyhow::Result;
use std::{collections::HashMap, fmt::Display, path::Path};

use serde::{Deserialize, Deserializer, Serialize};

pub mod resource;
pub mod secret_store;
pub mod slightfile;
pub use resource::Resource;
pub use secret_store::SecretStoreResource;
pub use slightfile::SlightFileInner;
/// slightfile version.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub enum SpecVersion {
    /// Version 0.1 format.
    #[serde(rename = "0.1")]
    V1,
    /// Version 0.2 format.
    #[serde(rename = "0.2")]
    #[default]
    V2,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SlightFile {
    pub specversion: SpecVersion,
    pub secret_store: Option<SecretStoreResource>,
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
        matches!(self, Capability::V1(_))
    }
    pub fn is_v2(&self) -> bool {
        matches!(self, Capability::V2(_))
    }
    pub fn resource(&self) -> Resource {
        match self {
            Capability::V1(c) => c.name,
            Capability::V2(c) => c.resource,
        }
    }
    pub fn name(&self) -> CapabilityName {
        match self {
            Capability::V1(c) => CapabilityName::Specific(c.name.to_string()),
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
    pub name: CapabilityName,
    pub configs: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub enum CapabilityName {
    #[serde(rename = "*")]
    Any,
    Specific(String),
}

impl Display for CapabilityName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilityName::Any => write!(f, "*"),
            CapabilityName::Specific(s) => write!(f, "{}", s),
        }
    }
}

impl<'de> Deserialize<'de> for CapabilityName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "*" {
            Ok(Self::Any)
        } else {
            Ok(Self::Specific(s))
        }
    }
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

#[derive(Debug, Clone, Default)]
pub struct SlightFileBuilder {
    file_content: String,
}

impl SlightFileBuilder {
    pub fn new() -> Self {
        Self {
            file_content: String::new(),
        }
    }
    pub fn path(mut self, path: impl AsRef<Path>) -> Result<Self> {
        let toml_file_contents = std::fs::read_to_string(path.as_ref())?;
        self.file_content = toml_file_contents;
        Ok(self)
    }
    pub fn build(self) -> Result<SlightFileInner> {
        let mut slight_file = SlightFileInner::from_toml_string(&self.file_content)?;
        slight_file.check_version()?;
        slight_file.validate_namespace()?;
        Ok(slight_file)
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
            .map(|path| {
                let toml_file = SlightFileBuilder::new().path(path.clone()).unwrap();
                let toml_file = toml_file.build();
                (toml_file, path)
            })
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
            .map(|path| {
                let toml_file = SlightFileBuilder::new().path(path.clone()).unwrap();
                let toml_file = toml_file.build();
                (toml_file, path)
            })
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
        let azblob = Resource::Blob(resource::BlobResource::Azblob);
        assert_eq!(azblob.to_string(), "blobstore.azblob");
    }

    #[test]
    fn deserialize_wildcard() -> Result<()> {
        let path = format!("{}/tests/good/msg.toml", env!("CARGO_MANIFEST_DIR"));

        // deserialize the toml file to struct
        let builder = SlightFileBuilder::new();
        let toml_file = builder.path(path)?.build()?;
        if let Some(capability) = &toml_file.as_ref().capability {
            assert!(capability.len() == 1);
            assert!(matches!(capability[0].name(), CapabilityName::Any));
        }

        // serialize the struct to toml
        let toml = toml::to_string(toml_file.as_ref())?;
        assert!(toml.contains("name = \"*\""));

        Ok(())
    }
}
