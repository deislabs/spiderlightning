use anyhow::{Context, Result};
use proc_macro_utils::{Resource, RuntimeResource};
use runtime::resource::{
    get, DataT, Linker, Map, Resource, ResourceMap, RuntimeContext, RuntimeResource,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;

use kv::*;

wit_bindgen_wasmtime::export!("../../wit/kv.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);

const SCHEME_NAME: &str = "filekv";

/// A Filesystem implementation for kv interface.
#[derive(Default, Clone, Resource, RuntimeResource)]
pub struct KvFilesystem {
    /// The root directory of the filesystem.
    inner: Option<String>,
    resource_map: Option<ResourceMap>,
}

impl kv::Kv for KvFilesystem {
    fn get_kv(&mut self, name: &str) -> Result<ResourceDescriptorResult, Error> {
        let path = Path::new("/tmp").join(name);
        let path = path
            .to_str()
            .with_context(|| format!("invalid path: {}", name))?
            .to_string();
        self.inner = Some(path);

        let rd = Uuid::new_v4().to_string();
        let cloned = self.clone(); // have to clone here because of the mutable borrow below
        let mut map = Map::unwrap(&mut self.resource_map)?;
        map.set(rd.clone(), Box::new(cloned));
        Ok(rd)
    }

    /// Output the value of a set key.
    fn get(&mut self, rd: ResourceDescriptorParam, key: &str) -> Result<PayloadResult, Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::unwrap(&mut self.resource_map)?;
        let base = map.get::<String>(rd)?;
        fs::create_dir_all(&base)
            .with_context(|| "failed to create base directory for kv store instance")?;
        let mut file =
            File::open(PathBuf::from(base).join(key)).with_context(|| "failed to get key")?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .with_context(|| "failed to read key's value")?;
        Ok(buf)
    }

    /// Create a key-value pair.
    fn set(
        &mut self,
        rd: ResourceDescriptorParam,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::unwrap(&mut self.resource_map)?;
        let base = map.get::<String>(rd)?;

        fs::create_dir_all(&base)
            .with_context(|| "failed to create base directory for kv store instance")?;

        let mut file =
            File::create(PathBuf::from(base).join(key)).with_context(|| "failed to create key")?;

        file.write_all(value)
            .with_context(|| "failed to set key's value")?;
        Ok(())
    }

    /// Delete a key-value pair.
    fn delete(&mut self, rd: ResourceDescriptorParam, key: &str) -> Result<(), Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::unwrap(&mut self.resource_map)?;
        let base = map.get::<String>(rd)?;

        fs::create_dir_all(&base)
            .with_context(|| "failed to create base directory for kv store instance")?;
        fs::remove_file(PathBuf::from(base).join(key))
            .with_context(|| "failed to delete key's value")?;
        Ok(())
    }
}
