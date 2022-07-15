pub mod envvars;
pub mod usersecrets;

use crate::ConfigType;
use anyhow::Result;

pub fn get(config_type: &str, key: &str, toml_file_path: &str) -> Result<Vec<u8>> {
    match config_type.into() {
        ConfigType::EnvVars => Ok(envvars::get(key)?),
        ConfigType::UserSecrets => Ok(usersecrets::get(key, toml_file_path)?),
    }
}

pub fn set(config_type: &str, key: &str, value: &[u8], toml_file_path: &str) -> Result<()> {
    match config_type.into() {
        ConfigType::EnvVars => Ok(envvars::set(key, value)?),
        ConfigType::UserSecrets => Ok(usersecrets::set(key, value, toml_file_path)?),
    }
}