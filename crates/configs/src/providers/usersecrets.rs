// use std::env;

use anyhow::Result;

pub fn get_config_usersecrets(_key: &str) -> Result<Vec<u8>> {
    // let toml_file_path = env::var(CFP_ENV_VAR_NAME);
    // serialize toml file to get key
    // get env var encryption key
    // decrypt key and return value
    Ok(vec![])
}

pub fn set_config_usersecrets(_key: &str, _value: &[u8]) -> Result<()> {
    // call in to wasi-cloud-cli to handle config creation

    Ok(())
} 