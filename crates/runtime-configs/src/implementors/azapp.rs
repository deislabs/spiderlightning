use anyhow::Result;
use azure_app_configuration::{client::AzureAppConfigClient, search_label::SearchLabel};

use super::envvars::EnvVars;

// TODO: maybe make this configurable
const MAX_NUM_RETRIES: i32 = 3;

pub struct AzApp;

fn make_client() -> Result<AzureAppConfigClient> {
    Ok(AzureAppConfigClient::new(
        String::from_utf8(EnvVars::get("AZAPPCONFIG_ENDPOINT")?)?,
        String::from_utf8(EnvVars::get("AZAPPCONFIG_KEYID")?)?,
        String::from_utf8(EnvVars::get("AZAPPCONFIG_KEYSECRET")?)?,
    ))
}

impl AzApp {
    pub async fn get(key: &str) -> Result<Vec<u8>> {
        let app_config_client = make_client()?;
        let mut ret = String::new();
        let mut count = 0;
        while count < MAX_NUM_RETRIES {
            let res = app_config_client.get_key_value(key, SearchLabel::All).await;
            if let Ok(r) = res {
                ret = r.value;
                break;
            } else {
                count += 1;
            }
        }

        if count == MAX_NUM_RETRIES {
            anyhow::bail!("failed to get message: maximum number of retries reached");
        }

        Ok(ret.as_bytes().to_vec())
    }

    pub async fn set(key: &str, value: &[u8]) -> Result<()> {
        let app_config_client = make_client()?;

        tracing::debug!("attempting to set key...");

        let mut count = 0;
        while count < MAX_NUM_RETRIES {
            let res = app_config_client.set_key(
                key,
                std::str::from_utf8(value)?,
                SearchLabel::For(""),
                None,
                None,
            ).await;

            if res.is_err() {
                count += 1;
            } else {
                break;
            }
        }

        if count == MAX_NUM_RETRIES {
            anyhow::bail!("failed to send message: maximum number of retries reached");
        }

        Ok(())
    }
}
