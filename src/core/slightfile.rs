use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TomlFile {
    pub specversion: Option<String>,
    pub secret_store: Option<String>,
    pub secret_settings: Option<Vec<Config>>,
    pub capability: Option<Vec<Capability>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub value: String,
}

impl Config {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}
