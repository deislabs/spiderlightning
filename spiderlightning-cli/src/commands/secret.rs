use std::fs::File;
use anyhow::Result;
use spiderlightning::{slightfile::TomlFile, core::secret::create_secret};

pub fn handle_secret(key: &str, value: &str, toml: &mut TomlFile, toml_file: &mut File) -> Result<()> {
    create_secret(key, value, toml, toml_file)
}