use anyhow::{bail, Result};
use azure_messaging_servicebus::prelude::*;
use chrono::Duration;

pub async fn send(client: &mut Client, msg: String) -> Result<()> {
    client.send_message(&msg).await?;
    Ok(())
}

pub async fn receive(client: &mut Client) -> Result<Vec<u8>> {
    let peek_lock = client
        .peek_lock_message2(Some(Duration::seconds(60)))
        .await?;

    if !peek_lock.status().is_success() {
        bail!("{} when reading queue.", peek_lock.status());
    }

    if peek_lock.status() == http::StatusCode::NO_CONTENT {
        bail!("no new messages found.");
    }

    let body = peek_lock.body();
    peek_lock.delete_message().await?;
    Ok(body.as_bytes().to_vec())
}
