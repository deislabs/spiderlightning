use anyhow::{bail, Result};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};
use url::Url;
use runtime::resource::{Resource, Context, Linker};

use kv::*;

wit_bindgen_wasmtime::export!("../../wit/kv.wit");

/// A Filesystem implementation for kv interface.
#[derive(Default)]
pub struct KvFilesystem {
    /// The root directory of the filesystem.
    path: String,
}

impl KvFilesystem {
    /// Create a new KvFilesystem.
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl kv::Kv for KvFilesystem {
    type ResourceDescriptor = u64;

    fn get_kv(&mut self) -> Result<Self::ResourceDescriptor, Error> {
        Ok(0)
    }

    /// Output the value of a set key.
    /// If key has not been set, return empty.
    fn get(&mut self, rd: &Self::ResourceDescriptor, key: &str) -> Result<PayloadResult, Error> {
        if *rd != 0 {
            return Err(Error::DescriptorError);
        }

        let mut file = File::open(path(key, &self.path))?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Create a key-value pair.
    fn set(
        &mut self,
        rd: &Self::ResourceDescriptor,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        if *rd != 0 {
            return Err(Error::DescriptorError);
        }
        let mut file = File::create(path(key, &self.path))?;
        file.write_all(value)?;
        Ok(())
    }

    /// Delete a key-value pair.
    fn delete(&mut self, rd: &Self::ResourceDescriptor, key: &str) -> Result<(), Error> {
        if *rd != 0 {
            return Err(Error::DescriptorError);
        }
        fs::remove_file(path(key, &self.path))?;
        Ok(())
    }
}

impl Resource for KvFilesystem {
    type State = (Self, KvTables<Self>);

    fn from_url(url: Url) -> Result<Self>
    where
            Self: Sized {
        let path = url.to_file_path();
        match path {
            Ok(path) => {
                let path = path.to_str().unwrap_or(".").to_string();
                Ok(KvFilesystem::new(path))
            }
            Err(_) => bail!("invalid url: {}", url),
        }
    }

    fn build_state(url: Url) -> Result<Self::State> {
        Ok((Self::from_url(url)?, Default::default()))
    }

    fn add_to_linker(
        linker: &mut Linker<Context<Self::State>>,
    ) -> Result<()> {
        kv::add_to_linker(linker, |ctx| {
            let (resource, resource_type) = Self::get_state(ctx);
            (resource, resource_type)
        })
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
