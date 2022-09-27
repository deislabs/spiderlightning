use std::sync::{Arc};
use std::borrow::BorrowMut;
use tokio::sync::Mutex;
use anyhow::{Context, Result};
use etcd_client::Client;
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;

use crate::providers::etcd;

/// This is the underlying struct behind the `Etcd` variant of the `EtcdImplementor` enum.
///
/// It provides a property that pertains solely to the etcd implementation
/// of this capability:
///     - `client`
///
/// As per its' usage in `EtcdImplementor`, it must `derive` `Debug`, and `Clone`.
#[derive(Clone)]
pub struct EtcdImplementor {
    client: Arc<Mutex<Client>>,
}

impl std::fmt::Debug for EtcdImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EtcdImplementor")
    }
}

impl EtcdImplementor {
    pub async fn new(slight_state: &BasicState) -> Self {
        let endpoint = get_from_state("ETCD_ENDPOINT", slight_state).await.unwrap();

        let client = Client::connect([endpoint], None).await
            .with_context(|| "failed to connect to etcd server")
            .unwrap();
        // ^^^ from my tests with localhost client, this never fails
        Self {
            client: Arc::new(Mutex::new(client)),
        }
    }

    pub async fn lock(&self, lock_name: &[u8]) -> Result<Vec<u8>> {
        let mut inner = self.client.lock().await;
        let pr = etcd::lock(inner.borrow_mut(), lock_name).await
            .with_context(|| "failed to acquire lock")?;
        Ok(pr)
    }

    pub async fn lock_with_time_to_live(
        &self,
        lock_name: &[u8],
        time_to_live_in_secs: i64,
    ) -> Result<Vec<u8>> {
        let pr = etcd::lock_with_lease(
            self.client.lock().await.borrow_mut(),
            lock_name,
            time_to_live_in_secs,
        ).await
        .with_context(|| "failed to acquire lock with time to live")?;
        Ok(pr)
    }

    pub async fn unlock(&self, lock_key: &[u8]) -> Result<()> {
        etcd::unlock(self.client.lock().await.borrow_mut(), lock_key).await
            .with_context(|| "failed to unlock")?;
        Ok(())
    }
}
