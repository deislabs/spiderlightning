use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretStoreResource {
    #[serde(rename = "configs.azapp")]
    Azapp,
    #[serde(rename = "configs.envvars")]
    Envvars,
    #[serde(rename = "configs.usersecrets")]
    Usersecrets,
    #[serde(rename = "configs.local")]
    Local,
}

impl TryFrom<String> for SecretStoreResource {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "configs.azapp" => Ok(SecretStoreResource::Azapp),
            "configs.envvars" => Ok(SecretStoreResource::Envvars),
            "configs.usersecrets" => Ok(SecretStoreResource::Usersecrets),
            "configs.local" => Ok(SecretStoreResource::Local),
            _ => bail!("Unknown secret store resource: {}", value),
        }
    }
}

impl From<SecretStoreResource> for String {
    fn from(value: SecretStoreResource) -> Self {
        match value {
            SecretStoreResource::Azapp => String::from("configs.azapp"),
            SecretStoreResource::Envvars => String::from("configs.envvars"),
            SecretStoreResource::Usersecrets => String::from("configs.usersecrets"),
            SecretStoreResource::Local => String::from("configs.local"),
        }
    }
}
