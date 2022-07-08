use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub specversion: Option<String>,
    pub secret_settings: Option<Vec<Secret>>,
    pub capability: Option<Vec<CapabilityConfig>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilityConfig {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub name: String,
    pub value: String,
}

impl Secret {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}
