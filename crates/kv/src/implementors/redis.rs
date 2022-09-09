use anyhow::{Context, Result};
use redis::{Client, Commands};
use slight_common::BasicState;

/// This is the underlying struct behind the `AzBlob` variant of the `KvImplementor` enum.
///
/// It provides a property that pertains solely to the azblob implementation
/// of this capability:
///     - `container_client`
///
/// As per its' usage in `KvImplementor`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
pub struct RedisImplementor {
    client: Client,
}

impl RedisImplementor {
    pub fn new(slight_state: &BasicState, _name: &str) -> Self {
        let connection_string = String::from_utf8(
            slight_runtime_configs::get(
                &slight_state.secret_store,
                "REDIS_ADDRESS",
                &slight_state.slightfile_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'REDIS_ADDRESS' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();

        let client = redis::Client::open(connection_string).unwrap();
        Self { client }
    }

    pub fn get(&self, key: &str) -> Result<Vec<u8>> {
        let mut con = self.client.get_connection()?;
        let val: Vec<u8> = con.get(key)?;
        // Redis GET returns [:ok; nil] for non-existent keys
        if val.is_empty() {
            return Err(anyhow::anyhow!("key not found"));
        }
        Ok(val)
    }

    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut con = self.client.get_connection()?;
        con.set(key, value)?;

        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        let mut con = self.client.get_connection()?;
        con.del(key)?;

        Ok(())
    }
}
