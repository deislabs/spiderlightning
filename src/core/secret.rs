use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use crate::core::slightfile::{Config, TomlFile};
use anyhow::{bail, Result};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use short_crypt::ShortCrypt;

pub const SLIGHTKEY: &str = "/tmp/.slightkey";

pub fn create_secret(
    key: &str,
    value: &str,
    toml: &mut TomlFile,
    toml_file: &mut File,
) -> Result<()> {
    maybe_set_key()?;
    let encryption_key = get_key()?;

    toml.secret_settings = if let Some(s) = &toml.secret_settings {
        // check that the secrets field is present
        Some(s.to_vec())
    } else {
        // if not, instantiate empty
        Some(vec![])
    };

    // find position of secret in toml's secret array
    let pos = toml
        .secret_settings
        .as_ref()
        .unwrap() // note: unwrapping here is fine because it is guaranteed to succeed
        .iter()
        .position(|s| s.name == key);

    // create encrypter instance w/ our encryption key
    let sc = ShortCrypt::new(encryption_key);
    let encrypted_value = sc.encrypt_to_url_component(&value); // encrypt our value to a random-like string
    let secret = Config::new(key.to_string(), encrypted_value);
    if let Some(p) = pos {
        // check if key doesn't already exist
        // if it does, we want to modify the existing field
        toml.secret_settings.as_mut().unwrap()[p] = secret;
        // ^^^ note: both of these unwraps cannot fail
    } else {
        // if it isn't, we will just create new
        toml.secret_settings.as_mut().unwrap().push(secret);
    }

    // write to toml file
    toml_file.write_all(toml::to_string(&toml)?.as_bytes())?;

    Ok(())
}

pub fn generate_key() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

pub fn get_key() -> Result<String> {
    if Path::new(SLIGHTKEY).exists() {
        Ok(std::fs::read_to_string(SLIGHTKEY)?)
    } else {
        bail!("usersecrets haven't been initialized yet, you can set your user secrets with `slight -c <config_file> -k <some_key> -v <some_value`.")
    }
}

pub fn maybe_set_key() -> Result<()> {
    let mut keyfile = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(SLIGHTKEY)?;

    if keyfile.metadata().unwrap().len() == 0 {
        // check file is empty
        keyfile.write_all(generate_key().as_bytes())?;
    }

    // if not empty, we keep the original key

    Ok(())
}
