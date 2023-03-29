use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc};
use tokio::sync::Mutex;
use anyhow::{bail, Result};
use async_trait::async_trait;
use dapr::{Client, client::TonicClient};
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;

use super::KeyvalueImplementor;

/// This is the underlying struct behind the `Dapr` variant of the `KeyvalueImplementor` enum.
///
/// As per its' usage in `KeyvalueImplementor`, it must `derive` `Debug`, and `Clone`.
#[derive(Clone)]
pub struct DaprImplementor {
    client: Arc<Mutex<RefCell<Client<TonicClient>>>>,
    container_name: String,
}

impl Debug for DaprImplementor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("[DaprImplementor] container_name: {}", self.container_name).as_str())
    }
}

impl DaprImplementor {
    pub async fn new(slight_state: &BasicState, name: &str) -> Self {
        let connection_string = get_from_state("DAPR_ADDRESS", slight_state).await.unwrap();
        let client = Client::connect(connection_string).await.unwrap();
        let container_name = name.to_string();
        let internal_mut_client = Arc::new(Mutex::new(RefCell::new(client)));
        Self {
            client: internal_mut_client,
            container_name,
        }
    }
}

#[async_trait]
impl KeyvalueImplementor for DaprImplementor {
    async fn get(&self, key: &str) -> Result<Vec<u8>> {
        let container = self.container_name.clone();
        let mut client = self.client.lock().await;
        let res = client.get_mut().get_state(container, key.to_string(), None).await?;

        if res.data.is_empty() {
            bail!("key not found");
        }
        Ok(res.data)
    }

    async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let container = self.container_name.clone();
        let mut client = self.client.as_ref().lock().await;
        client.get_mut().save_state(container, vec![(key.to_string(), value.to_vec())]).await?;

        Ok(())
    }

    async fn keys(&self) -> Result<Vec<String>> {
        bail!("not implemented");
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let container = self.container_name.clone();
        let mut client = self.client.lock().await;
        client.get_mut().delete_state(container, key.to_string(), None).await?;

        Ok(())
    }
}
