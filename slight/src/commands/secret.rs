use anyhow::Result;
use spiderlightning::core::{secret::create_secret, slightfile::TomlFile};
use std::fs::File;

pub fn handle_secret(
    key: &str,
    value: &str,
    toml: &mut TomlFile,
    toml_file: &mut File,
) -> Result<()> {
    toml.secret_store = Some("configs.usersecrets".to_string());
    create_secret(key, value, toml, toml_file)
}
