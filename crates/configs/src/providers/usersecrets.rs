use std::{env, fs::OpenOptions};

use anyhow::{bail, Result};
use short_crypt::ShortCrypt;
use spiderlightning::{
    constants::{SLIGHTFILE_PATH, SLIGHTKEY},
    core::secret::handle_secret,
    slightfile::TomlFile,
};

pub fn get(key: &str) -> Result<Vec<u8>> {
    // check if encryption key env var is present
    let encryption_key = if let Ok(s) = env::var(SLIGHTKEY) {
        s
    } else {
        bail!("failed because user secrets has never been initialized")
    };

    // serialize toml file to get key
    let toml_file_path = env::var(SLIGHTFILE_PATH)?;
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

    let value = if pos.is_some() {
        &toml.secret_settings.as_ref().unwrap()[pos.unwrap()].value
        // ^^^ note: both of these unwraps cannot fail
    } else {
        // if it isn't, we will just create new
        bail!("failed because this secret isn't encrypted in the toml file")
    };

    // decrypt key and return value
    let sc = ShortCrypt::new(encryption_key);
    sc.decrypt_url_component(&value)
        .map_err(|err| anyhow::anyhow!(err))
}

pub fn set(key: &str, value: &[u8]) -> Result<()> {
    // call in to slight to handle config creation
    let toml_file_path = env::var(SLIGHTFILE_PATH)?;
    let mut toml_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&toml_file_path)?;
    let toml_file_contents = std::fs::read_to_string(&toml_file_path)?;
    let mut toml = toml::from_str::<TomlFile>(&toml_file_contents)?;
    handle_secret(key, std::str::from_utf8(value)?, &mut toml, &mut toml_file)
}
