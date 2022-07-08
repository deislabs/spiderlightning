use std::{env, fs::File, io::Write};

use crate::wc_config::{Config, Secret};
use anyhow::Result;
use argon2rs::argon2i_simple;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

const ENV_VAR_NAME: &str = "WASI_CLOUD_SALT";

pub fn handle_secret(
    key: &str,
    value: &str,
    toml: &mut Config,
    toml_file: &mut File,
) -> Result<()> {
    // check if salt env var is present
    let salt = if let Ok(s) = env::var(ENV_VAR_NAME) {
        s
    } else {
        // if it isn't, create
        let s = generate_salt();
        env::set_var(ENV_VAR_NAME, &s);
        s
    };

    toml.secret_settings = if let Some(s) = &toml.secret_settings {
        // check that the secrets field is present
        Some(s.to_vec())
    } else {
        // if not, instantiate empty
        Some(vec![])
    };

    let pos = toml
        .secret_settings
        .as_ref()
        .unwrap() // note: unwrapping here is fine because it is guaranteed to succeed
        .iter()
        .position(|s| s.name == key);

    let hash = argon2i_simple(&value, &salt)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    let secret = Secret::new(key.to_string(), hash);
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

fn generate_salt() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}