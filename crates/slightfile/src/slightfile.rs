use std::collections::{HashMap, HashSet};

use crate::{
    resource::HttpServerResource, Capability, Resource, ResourceName, SlightFile, SpecVersion,
};
use anyhow::{bail, Result};

#[derive(Debug, Clone, Default)]
pub struct SlightFileInner {
    inner: SlightFile,
    groups_by_resource: HashMap<String, HashSet<ResourceName>>,
}

impl AsRef<SlightFile> for SlightFileInner {
    fn as_ref(&self) -> &SlightFile {
        &self.inner
    }
}

impl AsMut<SlightFile> for SlightFileInner {
    fn as_mut(&mut self) -> &mut SlightFile {
        &mut self.inner
    }
}

impl SlightFileInner {
    pub fn from_toml_string(toml: &str) -> Result<Self> {
        let inner = toml::from_str::<SlightFile>(toml)?;
        Ok(Self {
            inner,
            ..Default::default()
        })
    }
    pub fn check_version(&self) -> Result<()> {
        // check specversion
        match self.inner.specversion {
            SpecVersion::V1 => {
                if self.inner.capability.as_ref().is_some()
                    && self
                        .inner
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
                if self.inner.capability.as_ref().is_some()
                    && self
                        .inner
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
    pub fn de_dup(self) -> Result<Self> {
        unimplemented!("de-duplicate logic is not implemented yet");
    }

    /// Validate the namespace for each resource is unique in the slightfile.
    ///
    /// Each reousrce can only acquire one unique namespace. Any two resources
    /// with overlapping namespaces are invalid.
    ///
    /// A special case is when you have a resource that uses the Any resource name.
    /// In this case, if there are any resources of the same capability are defined
    /// in the slightfile, this function returns an error.
    pub fn validate_namespace(&mut self) -> Result<()> {
        if let Some(capabilities) = &mut self.inner.capability {
            for cap in capabilities {
                let res = cap.resource().to_cap_name();
                self.groups_by_resource.entry(res.clone()).or_default();
                if self
                    .groups_by_resource
                    .get(&res)
                    .unwrap()
                    .contains(&cap.name())
                    || self
                        .groups_by_resource
                        .get(&res)
                        .unwrap()
                        .contains(&ResourceName::Any)
                {
                    bail!(
                        "Error: the namespace {} is already defined in the slightfile",
                        cap.name()
                    );
                } else {
                    // if current resource is Any, and if there are any other resources defined in the slightfile,
                    // this is an error.
                    if cap.name() == ResourceName::Any
                        && !self.groups_by_resource.get(&res).unwrap().is_empty()
                    {
                        bail!(
                            "Error: the namespace {} is already defined in the slightfile",
                            cap.name()
                        );
                    }

                    self.groups_by_resource
                        .get_mut(&res)
                        .unwrap()
                        .insert(cap.name());
                }
            }
        }
        Ok(())
    }

    pub fn has_http_cap(&self) -> bool {
        if let Some(capability) = &self.inner.capability {
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
}
