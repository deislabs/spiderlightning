use azure_sdk_service_bus::prelude::*;
use anyhow::{Result, bail};
use chrono::Duration;
use hyper::StatusCode;

pub async fn send(client: &mut Client, msg: String) -> Result<()> {
    client.send_event(&msg, Duration::hours(1))
        .await
        .unwrap();
    Ok(())
}

pub async fn receive(client: &mut Client) -> Result<Vec<u8>> {
    Ok(client.receive_and_delete(Duration::hours(1)).await?.as_bytes().to_vec())

    // let peek_lock = match client
    //         .peek_lock_full(Duration::hours(1), None)
    //         .await
    //         .map_err(|e| {
    //             bail!("{:?}", e);
    //         }) {
    //     Ok(it) => it,
    //     Err(err) => {
    //         println!("{:#?}", err);
    //         return err;
    //     }
    // };

    // if !peek_lock.status().is_success() {
    //     bail!("{} when reading queue.", peek_lock.status());
    // }

    // if peek_lock.status() == StatusCode::NO_CONTENT {
    //     bail!("No new messages found.");
    // }

    // let body = peek_lock.body();

    // println!(
    //     "Message found perfectly parseable ({:?}).",
    //     body,
    // );
    // Ok(body.as_bytes().to_vec())
}