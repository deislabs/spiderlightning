use anyhow::Result;
use etcd_client::{Client, LockOptions};

pub async fn lock(client: &mut Client, lock_name: &[u8]) -> Result<Vec<u8>> {
    let resp = client.lock(lock_name, None).await?;
    Ok(resp.key().to_vec())
}

pub async fn lease_grant(client: &mut Client, ttl: i64) -> Result<i64> {
    let resp = client.lease_grant(ttl, None).await?;
    Ok(resp.id())
}

pub async fn lock_with_lease(client: &mut Client, lock_name: &[u8], lease_id: i64) -> Result<Vec<u8>> {
    let lock_options = LockOptions::new().with_lease(lease_id);
    let resp = client.lock(lock_name, Some(lock_options)).await?;
    Ok(resp.key().to_vec())
}

pub async fn unlock(client: &mut Client, lock_key: &[u8]) -> Result<()> {
    client.unlock(lock_key).await?;
    Ok(())
}