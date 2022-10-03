use std::{
    fs::{create_dir_all, remove_dir_all, File},
    io::{self, ErrorKind},
};

use anyhow::Result;

const GITHUB_URL: &str = "https://github.com/deislabs/spiderlightning/releases/download";

const KV_DOWNLOADS: [&str; 3] = ["kv", "types", "resources"];
const CONFIGS_DOWNLOADS: [&str; 2] = ["configs", "types"];
const EVENTS_DOWNLOADS: [&str; 4] = ["events", "event-handler", "types", "resources"];
const HTTP_DOWNLOADS: [&str; 4] = ["http", "http-handler", "http-types", "types"];
const LOCKD_DOWNLOADS: [&str; 3] = ["lockd", "types", "resources"];
const MQ_DOWNLOADS: [&str; 3] = ["mq", "types", "resources"];
const PUBSUB_DOWNLOADS: [&str; 3] = ["pubsub", "types", "resources"];

pub async fn handle_add(what_to_add: &str, folder_prefix: Option<&str>) -> Result<()> {
    let (interface, release, folder_name) = if !what_to_add.contains('@') {
        panic!("invalid usage: to add an interface to your project, say `slight add -i <interface-name>@<release-tag>`");
        // TODO: In the future, let's support omitting the release tag to download the latest release
    } else {
        let find_at = what_to_add.find('@').unwrap();
        // ^^^ fine to unwrap, we are guaranteed to have a '@' at this point.
        (
            &what_to_add[..find_at],
            &what_to_add[find_at + 1..],
            what_to_add.replace('@', "_"),
        )
    };

    match what_to_add {
        _ if interface.eq("kv")
            | interface.eq("configs")
            | interface.eq("events")
            | interface.eq("http")
            | interface.eq("lockd")
            | interface.eq("mq")
            | interface.eq("pubsub") =>
        {
            maybe_recreate_dir(&format!("{}{}", folder_prefix.unwrap_or("./"), folder_name))?;
            for i in get_interface_downloads_by_name(interface) {
                let resp = reqwest::get(format!("{}/{}/{}.wit", GITHUB_URL, release, i))
                    .await?
                    .text()
                    .await?;
                let mut out = File::create(format!("{}{}/{}.wit", folder_prefix.unwrap_or("./"), folder_name, i))?;
                io::copy(&mut resp.as_bytes(), &mut out)?;
            }
        }
        _ => {
            panic!("invalid interface name (1): currently, slight only supports the download of 'configs', 'events', 'kv', 'mq', 'lockd', 'pubsub', and 'http'.")
        }
    }
    Ok(())
}

fn maybe_recreate_dir(dir_name: &str) -> Result<()> {
    match remove_dir_all(dir_name) {
        Err(e) if e.kind() != ErrorKind::NotFound => {
            panic!("{}", e);
        }
        _ => {
            create_dir_all(dir_name)?;
        }
    }

    Ok(())
}

fn get_interface_downloads_by_name(name: &str) -> Vec<&str> {
    match name {
        _ if name.eq("kv") => KV_DOWNLOADS.to_vec(),
        _ if name.eq("configs") => CONFIGS_DOWNLOADS.to_vec(),
        _ if name.eq("events") => EVENTS_DOWNLOADS.to_vec(),
        _ if name.eq("http") => HTTP_DOWNLOADS.to_vec(),
        _ if name.eq("lockd") => LOCKD_DOWNLOADS.to_vec(),
        _ if name.eq("mq") => MQ_DOWNLOADS.to_vec(),
        _ if name.eq("pubsub") => PUBSUB_DOWNLOADS.to_vec(),
        _ => {
            panic!("invalid interface name (2): currently, slight only supports the download of 'configs', 'events', 'kv', 'mq', 'lockd', 'pubsub', and 'http'.")
        }
    }
}
