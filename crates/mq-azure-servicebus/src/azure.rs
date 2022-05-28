use azure_messaging_servicebus::prelude::*;
use anyhow::{Result, bail};
use chrono::Duration;

pub async fn send(client: &mut Client, msg: String) -> Result<()> {
    client.send_message(&msg)
        .await
        .unwrap();
    Ok(())
}

pub async fn receive(client: &mut Client) -> Result<Vec<u8>> {
    let peek_lock = match client
            .peek_lock_message2(Some(Duration::seconds(60)))
            .await
            .map_err(|e| {
                bail!("{:?}", e);
            }) {
        Ok(it) => {
            it
        },
        Err(err) => {
            return err;
        }
    };

    if !peek_lock.status().is_success() {
        bail!("{} when reading queue.", peek_lock.status());
    }

    if peek_lock.status() == http::StatusCode::NO_CONTENT {
        bail!("No new messages found.");
    }

    let body = peek_lock.body();
    peek_lock.delete_message().await?;
    Ok(body.as_bytes().to_vec())
}