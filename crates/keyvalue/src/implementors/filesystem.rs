use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use slight_common::BasicState;

use super::KeyvalueImplementor;

/// This is the underlying struct behind the `Filesystem` variant of the `KeyvalueImplementor` enum.
///
/// It provides two properties that pertain solely to the filesystem implementation of
/// of this capability:
///     - `base`
///
/// As per its' usage in `KeyvalueImplementor`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
pub struct FilesystemImplementor {
    /// The base path for where the key-value store can be found in your file-system
    pub base: String,
}

impl FilesystemImplementor {
    pub async fn new(_slight_state: &BasicState, name: &str) -> Self {
        Self {
            base: env::temp_dir().join(name).to_str().unwrap().to_owned(),
        }
    }
}

#[async_trait]
impl KeyvalueImplementor for FilesystemImplementor {
    async fn get(&self, key: &str) -> Result<Vec<u8>> {
        fs::create_dir_all(&self.base)
            .with_context(|| "failed to create base directory for keyvalue instance")?;
        let mut file =
            File::open(PathBuf::from(&self.base).join(key)).with_context(|| "failed to get key")?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .with_context(|| "failed to read key's value")?;
        Ok(buf)
    }

    async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        fs::create_dir_all(&self.base)
            .with_context(|| "failed to create base directory for keyvalue instance")?;

        let mut file = File::create(PathBuf::from(&self.base).join(key))
            .with_context(|| "failed to create key")?;

        file.write_all(value)
            .with_context(|| "failed to set key's value")?;
        Ok(())
    }

    async fn keys(&self) -> Result<Vec<String>> {
        fs::create_dir_all(&self.base)
            .with_context(|| "failed to create base directory for keyvalue instance")?;

        let mut keys = Vec::new();
        for entry in fs::read_dir(&self.base).with_context(|| "failed to read base directory")? {
            let entry = entry.with_context(|| "failed to read base directory entry")?;
            keys.push(entry.file_name().to_str().unwrap().to_owned());
        }
        Ok(keys)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        fs::create_dir_all(&self.base)
            .with_context(|| "failed to create base directory for keyvalue instance")?;
        fs::remove_file(PathBuf::from(&self.base).join(key))
            .with_context(|| "failed to delete key's value")?;
        Ok(())
    }
}
