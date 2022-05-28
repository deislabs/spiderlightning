use anyhow::{bail, Result};
use runtime::resource::Addressable;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};
use url::Url;

/// A Filesystem implementation for kv interface.
#[derive(Default, Debug)]
pub struct KvFilesystem {
    /// The root directory of the filesystem.
    path: String,
}

impl KvFilesystem {
    /// Create a new KvFilesystem.
    pub fn new(path: String) -> Self {
        Self { path }
    }

    /// Output the value of a set key.
    /// If key has not been set, return empty.
    pub fn get(&self, key: &str) -> Result<Vec<u8>> {
        let mut file = File::open(path(key, &self.path))?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Create a key-value pair.
    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut file = File::create(path(key, &self.path))?;
        file.write_all(value)?;
        Ok(())
    }

    /// Delete a key-value pair.
    pub fn delete(&self, key: &str) -> Result<()> {
        fs::remove_file(path(key, &self.path))?;
        Ok(())
    }
}

impl Addressable for KvFilesystem {
    fn from_url(url: Url) -> Result<Self> {
        let path = url.to_file_path();
        match path {
            Ok(path) => {
                let path = path.to_str().unwrap_or(".").to_string();
                Ok(KvFilesystem::new(path))
            }
            Err(_) => bail!("invalid url: {}", url),
        }
    }
}

/// Return the absolute path for the file corresponding to the given key.
fn path(name: &str, base: &str) -> PathBuf {
    PathBuf::from(base).join(name)
}
