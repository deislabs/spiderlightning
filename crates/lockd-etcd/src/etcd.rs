use anyhow::Result;
use etcd_client::{Client, LockOptions, LockResponse};

pub async fn lock(client: &mut Client, lock_name: &[u8]) -> Result<Vec<u8>> {
    let resp = client.lock(lock_name, None).await?;
    Ok(resp.key().to_vec())
}

pub async fn lease_grant(client: &mut Client, ttl: i64) -> Result<i64> {
    let resp = client.lease_grant(ttl, None).await?;
    Ok(resp.id())
}

pub async fn lock_with_lease(
    client: &mut Client,
    lock_name: &[u8],
    time_to_live_in_secs: i64,
) -> Result<Vec<u8>> {
    let mut resp = create_lease_and_lock_with_it(client, lock_name, time_to_live_in_secs).await;
    if resp.is_err() {
        // if we get an error here, it's because the lease expired before we could grab a lock
        resp = create_lease_and_lock_with_it(client, lock_name, time_to_live_in_secs).await;
    }
    Ok(resp?.key().to_vec())
}

pub async fn unlock(client: &mut Client, lock_key: &[u8]) -> Result<()> {
    client.unlock(lock_key).await?;
    Ok(())
}

pub async fn create_lease_and_lock_with_it(
    client: &mut Client,
    lock_name: &[u8],
    time_to_live_in_secs: i64,
) -> Result<LockResponse> {
    let lease_id = lease_grant(client, time_to_live_in_secs).await?;
    let lock_options = LockOptions::new().with_lease(lease_id);
    client.lock(lock_name, Some(lock_options))
        .await
        .map_err(|_| anyhow::anyhow!("error at create_lease_and_lock_with_it"))
}
