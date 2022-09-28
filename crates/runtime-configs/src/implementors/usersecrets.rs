use std::{fs::OpenOptions, path::Path};

use anyhow::{bail, Result};
use short_crypt::ShortCrypt;
use spiderlightning::core::{
    secret::{create_secret, get_key},
    slightfile::TomlFile,
};

pub struct UserSecrets;

impl UserSecrets {
    pub fn get(key: &str, toml_file_path: impl AsRef<Path>) -> Result<Vec<u8>> {
        // check if encryption key env var is present
        let encryption_key = if let Ok(s) = get_key() {
            s
        } else {
            bail!("failed because user secrets has never been initialized")
        };

        // serialize toml file to get key
        let toml_file_path = toml_file_path;
        let toml_file_contents = std::fs::read_to_string(toml_file_path)?;
        let toml = toml::from_str::<TomlFile>(&toml_file_contents)?;
        if toml.secret_settings.is_none() {
            bail!("failed because toml file has no secrets");
        }

        // get env var encryption key
        let pos = toml
            .secret_settings
            .as_ref()
            .unwrap() // note: this unwrap will never fail, so it's ok
            .iter()
            .position(|s| s.name == key);

        let value = if let Some(value) = pos {
            &toml.secret_settings.as_ref().unwrap()[value].value
            // ^^^ note: the unwrap cannot fail
        } else {
            // if it isn't, we will just create new
            bail!("failed because this secret isn't encrypted in the toml file")
        };

        // decrypt key and return value
        let sc = ShortCrypt::new(encryption_key);
        sc.decrypt_url_component(value)
            .map_err(|err| anyhow::anyhow!(err))
    }

    pub fn set(key: &str, value: &[u8], toml_file_path: impl AsRef<Path>) -> Result<()> {
        // call in to slight to handle config creation
        let toml_file_path = toml_file_path;
        let mut toml_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&toml_file_path)?;
        let toml_file_contents = std::fs::read_to_string(&toml_file_path)?;
        let mut toml = toml::from_str::<TomlFile>(&toml_file_contents)?;
        create_secret(key, std::str::from_utf8(value)?, &mut toml, &mut toml_file)
    }
}

#[cfg(test)]
mod unittests {
    use anyhow::Result;
    use tempdir::TempDir;

    use super::UserSecrets;

    #[test]
    fn set_then_get_test() -> Result<()> {
        let dir = TempDir::new("tmp")?;
        let file_path = dir.path().join("slightfile.toml");
        let toml_file_path = file_path.to_str().unwrap();
        UserSecrets::set("key", "value".as_bytes(), toml_file_path)?;
        assert!(UserSecrets::get("key", toml_file_path).is_ok());
        Ok(())
    }
}
