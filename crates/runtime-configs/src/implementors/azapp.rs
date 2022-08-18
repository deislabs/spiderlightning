use anyhow::Result;
use azure_app_configuration::{client::AzureAppConfigClient, search_label::SearchLabel};
use futures::executor::block_on;

use super::envvars::EnvVars;

pub struct AzApp;

fn make_client() -> Result<AzureAppConfigClient> {
    Ok(AzureAppConfigClient::new(
        String::from_utf8(EnvVars::get("AZAPPCONFIG_ENDPOINT")?)?,
        String::from_utf8(EnvVars::get("AZAPPCONFIG_KEYID")?)?,
        String::from_utf8(EnvVars::get("AZAPPCONFIG_KEYSECRET")?)?,
    ))
}

impl AzApp {
    pub fn get(key: &str) -> Result<Vec<u8>> {
        let app_config_client = make_client()?;
        let res: String = block_on(app_config_client.get_key_value(key, SearchLabel::All))
            .expect("failed to get key")
            .value;

        Ok(res.as_bytes().to_vec())
    }

    pub fn set(key: &str, value: &[u8]) -> Result<()> {
        let app_config_client = make_client()?;

        tracing::debug!("attempting to set key...");
        block_on(app_config_client.set_key(
            key,
            std::str::from_utf8(value)?,
            SearchLabel::For(""),
            None,
            None,
        ))
        .expect("failed to set key value");

        Ok(())
    }
}
