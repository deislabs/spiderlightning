use anyhow::{bail, Result};
use resource::HttpServerResource;
use std::{collections::HashMap, fmt::Display, path::Path};

use serde::{Deserialize, Deserializer, Serialize};

pub mod resource;
pub mod secret_store;
pub use resource::Resource;
pub use secret_store::SecretStoreResource;
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
pub struct SlightFile {
    pub specversion: SpecVersion,
    pub secret_store: Option<SecretStoreResource>,
    pub secret_settings: Option<Vec<Config>>,
    pub capability: Option<Vec<Capability>>,
}

impl SlightFile {
    pub fn from_toml_string(toml: &str) -> Result<Self> {
        let s = toml::from_str::<SlightFile>(toml)?;
        Ok(s)
    }
    pub fn check_version(&self) -> Result<()> {
        // check specversion
        match self.specversion {
            SpecVersion::V1 => {
                if self.capability.as_ref().is_some()
                    && self
                        .capability
                        .as_ref()
                        .unwrap()
                        .iter()
                        .any(|cap| cap.is_v2())
                {
                    bail!("Error: you are using a 0.1 specversion, but you are using a 0.2 capability format");
                }
            }
            SpecVersion::V2 => {
                if self.capability.as_ref().is_some()
                    && self
                        .capability
                        .as_ref()
                        .unwrap()
                        .iter()
                        .any(|cap| cap.is_v1())
                {
                    bail!("Error: you are using a 0.2 specversion, but you are using a 0.1 capability format");
                }
            }
        };
        Ok(())
    }

    /// For each capability, deduplicate the resource names.
    ///
    /// For example, if you have two resources with the same resource name,
    /// this will return only one resource.
    ///
    /// A special case is when you have a resource that uses the
    /// Any resource name. In this case, all reousrces of the same capability
    /// except this one will be removed from the list.
    pub fn de_dup(mut self) -> Result<Self> {
        // if let Some(capabilities) = &mut self.capability {
        //     let mut new_capabilities = vec!();
        //     for cap in capabilities {
        //         let resource = cap.resource();

        //         if let Some(cap) = new_capabilities.iter_mut().find(|c| c.resource() == resource) {
        //             if cap.name() == CapabilityName::Any {
        //                 continue;
        //             }
        //             if name == CapabilityName::Any {
        //                 *cap = Capability::V2(CapabilityV2 {
        //                     resource,
        //                     name,
        //                     configs,
        //                 });
        //             }
        //         } else {
        //             new_capabilities.push(Capability::V2(CapabilityV2 {
        //                 resource,
        //                 name,
        //                 configs,
        //             }));
        //         }
        //     }
        //     self.capability = Some(new_capabilities);
        // }
        Ok(self)
    }
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
            Capability::V1(c) => c.name.clone(),
            Capability::V2(c) => c.resource.clone(),
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

#[derive(Debug, Clone, Serialize)]
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
    pub fn build(self) -> Result<SlightFile> {
        let slight_file = SlightFile::from_toml_string(&self.file_content)?;
        slight_file.check_version()?;
        Ok(slight_file)
    }
}

pub fn has_http_cap(toml: &SlightFile) -> bool {
    if let Some(capability) = &toml.capability {
        capability.iter().any(|cap| match cap {
            Capability::V1(cap) => {
                matches!(cap.name, Resource::HttpServer(HttpServerResource::Server))
            }
            Capability::V2(cap) => matches!(
                cap.resource,
                Resource::HttpServer(HttpServerResource::Server)
            ),
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
        let toml_file = builder.path(path.clone())?.build()?;
        if let Some(capability) = &toml_file.capability {
            assert!(capability.len() == 1);
            assert!(matches!(capability[0].name(), CapabilityName::Any));
        }

        // serialize the struct to toml
        let toml = toml::to_string(&toml_file)?;
        assert!(toml.contains("name = \"*\""));

        Ok(())
    }
}
