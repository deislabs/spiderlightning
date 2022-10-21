use std::time::Duration;

use anyhow::{bail, Result};
use azure_core::{HttpClient, StatusCode};
use azure_messaging_servicebus::prelude::*;

pub async fn send(client: &mut Client, msg: String) -> Result<()> {
    client.send_message(&msg).await?;
    Ok(())
}

pub async fn receive(client: &mut Client) -> Result<Vec<u8>> {
    let peek_lock = client
        .peek_lock_message2(Some(Duration::new(60, 0)))
        .await?;

    if !peek_lock.status().is_success() {
        bail!("{} when reading queue.", peek_lock.status());
    }

    if peek_lock.status() == &StatusCode::NoContent {
        bail!("no new messages found.");
    }

    let body = peek_lock.body();
    peek_lock.delete_message().await?;
    Ok(body.as_bytes().to_vec())
}

// pub async fn create_or_update_queue(client: &Arc<dyn HttpClient>, namespace: &str, queue_name: &str) -> Result<()> {
//     let url = format!(
//         "https://{}.servicebus.windows.net/{}",
//         namespace,
//         queue_name
//     );
//     let req = finalize_request(
//         &url,
//         Method::Post,
//         Some(msg.to_string()),
//         policy_name,
//         signing_key,
//     )?;
// }