use anyhow::{bail, Result};
use async_trait::async_trait;
use redis::{Client, Commands};
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;

use super::KeyvalueImplementor;

/// This is the underlying struct behind the `AzBlob` variant of the `KeyvalueImplementor` enum.
///
/// It provides a property that pertains solely to the azblob implementation
/// of this capability:
///     - `container_client`
///
/// As per its' usage in `KeyvalueImplementor`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
pub struct RedisImplementor {
    client: Client,
    container_name: String,
}

impl RedisImplementor {
    pub async fn new(slight_state: &BasicState, name: &str) -> Self {
        let connection_string = get_from_state("REDIS_ADDRESS", slight_state).await.unwrap();
        let client = redis::Client::open(connection_string).unwrap();
        let container_name = name.to_string();
        Self {
            client,
            container_name,
        }
    }
}

#[async_trait]
impl KeyvalueImplementor for RedisImplementor {
    async fn get(&self, key: &str) -> Result<Vec<u8>> {
        let mut con = self.client.get_connection()?;
        let val: Vec<u8> = con.get(format!("{}:{}", self.container_name, key))?;
        // Redis GET returns [:ok; nil] for non-existent keys
        if val.is_empty() {
            bail!("key not found");
        }
        Ok(val)
    }

    async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut con = self.client.get_connection()?;
        con.set(format!("{}:{}", self.container_name, key), value)?;

        Ok(())
    }

    async fn keys(&self) -> Result<Vec<String>> {
        let mut con = self.client.get_connection()?;
        let keys: Vec<String> = con.keys(format!("{}:*", self.container_name))?;
        // remove prefix
        let keys: Vec<String> = keys
            .iter()
            .map(|k| k.replace(format!("{}:", self.container_name).as_str(), ""))
            .collect();
        Ok(keys)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut con = self.client.get_connection()?;
        con.del(format!("{}:{}", self.container_name, key))?;

        Ok(())
    }
}
