use anyhow::Result;
use runtime::resource::{get, Context, DataT, HostResource, Linker, Resource, ResourceMap};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;

use kv::*;

wit_bindgen_wasmtime::export!("../../wit/kv.wit");

const SCHEME_NAME: &str = "file";

/// A Filesystem implementation for kv interface.
#[derive(Default, Clone)]
pub struct KvFilesystem {
    /// The root directory of the filesystem.
    path: String,
    resource_map: Option<ResourceMap>,
}

impl kv::Kv for KvFilesystem {
    fn get_kv(&mut self, name: &str) -> Result<ResourceDescriptorResult, Error> {
        // TODO: we hard code to use the `/tmp` directory for now.
        let path = Path::new("/tmp").join(name);
        let path = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("invalid path: {}", name))?
            .to_string();
        self.path = path;

        let uuid = Uuid::new_v4();
        let rd = uuid.to_string();

        let cloned = self.clone();
        let mut map = self
            .resource_map
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        map.set(rd.clone(), Box::new(cloned))?;

        Ok(rd)
    }

    /// Output the value of a set key.
    /// If key has not been set, return empty.
    fn get(&mut self, rd: ResourceDescriptorParam, key: &str) -> Result<PayloadResult, Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }

        let map = self
            .resource_map
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        let base = map.get::<String>(rd)?;
        fs::create_dir_all(&base)?;
        let mut file = File::open(path(key, base))?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Create a key-value pair.
    fn set(
        &mut self,
        rd: ResourceDescriptorParam,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }

        let map = self
            .resource_map
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        let base = map.get::<String>(rd)?;

        fs::create_dir_all(&base)?;

        let mut file = match File::create(path(key, base)) {
            Ok(file) => file,
            Err(_) => {
                return Err(Error::IoError);
            }
        };

        file.write_all(value)?;
        Ok(())
    }

    /// Delete a key-value pair.
    fn delete(&mut self, rd: ResourceDescriptorParam, key: &str) -> Result<(), Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }

        let map = self
            .resource_map
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        let base = map.get::<String>(rd)?;

        fs::create_dir_all(&base)?;
        fs::remove_file(path(key, base))?;
        Ok(())
    }
}

impl Resource for KvFilesystem {
    // fn from_url(url: Url) -> Result<Self> {
    //     let path = url.to_file_path();
    //     match path {
    //         Ok(path) => {
    //             let path = path.to_str().unwrap_or(".").to_string();
    //             Ok(KvFilesystem::new(path))
    //         }
    //         Err(_) => bail!("invalid url: {}", url),
    //     }
    // }

    fn add_resource_map(&mut self, resource_map: ResourceMap) -> Result<()> {
        self.resource_map = Some(resource_map);
        Ok(())
    }

    fn get_inner(&self) -> &dyn std::any::Any {
        &self.path
    }
}

impl HostResource for KvFilesystem {
    fn add_to_linker(linker: &mut Linker<Context<DataT>>) -> Result<()> {
        crate::add_to_linker(linker, |cx| get::<Self>(cx, SCHEME_NAME.to_string()))
    }

    fn build_data() -> Result<DataT> {
        let kv_filesystem = Self::default();
        Ok(Box::new(kv_filesystem))
    }
}

/// Return the absolute path for the file corresponding to the given key.
fn path(name: &str, base: &str) -> PathBuf {
    PathBuf::from(base).join(name)
}

impl From<anyhow::Error> for Error {
    fn from(_: anyhow::Error) -> Self {
        Self::OtherError
    }
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::IoError
    }
}
