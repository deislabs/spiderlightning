use std::{env, fs::File, io::Write};

use crate::slightfile::{TomlFile, Config};
use anyhow::Result;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use short_crypt::ShortCrypt;

const EKEY_ENV_VAR_NAME: &str = "WASI_CLOUD_KEY";

pub fn handle_secret(
    key: &str,
    value: &str,
    toml: &mut TomlFile,
    toml_file: &mut File,
) -> Result<()> {
    // check if encryption key env var is present
    let encryption_key = if let Ok(s) = env::var(EKEY_ENV_VAR_NAME) {
        s
    } else {
        // if it isn't, create it
        let s = generate_key();
        env::set_var(EKEY_ENV_VAR_NAME, &s);
        s
    };

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
    if pos.is_some() {
        // check if key doesn't already exist
        // if it does, we want to modify the existing field
        toml.secret_settings.as_mut().unwrap()[pos.unwrap()] = secret;
        // ^^^ note: both of these unwraps cannot fail
    } else {
        // if it isn't, we will just create new
        toml.secret_settings.as_mut().unwrap().push(secret);
    }

    // write to toml file
    toml_file.write_all(toml::to_string(&toml)?.as_bytes())?;

    Ok(())
}

fn generate_key() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}