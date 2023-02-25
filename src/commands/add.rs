use std::{
    fs::{create_dir_all, remove_dir_all, File},
    io::{self, ErrorKind},
};

use anyhow::Result;
use slight_core::interface_parser::InterfaceAtRelease;

const GITHUB_URL: &str = "https://github.com/deislabs/spiderlightning/releases/download";

const KEYVALUE_DOWNLOADS: [&str; 1] = ["keyvalue"];
const CONFIGS_DOWNLOADS: [&str; 1] = ["configs"];
const HTTP_DOWNLOADS: [&str; 3] = ["http-server", "http-handler", "http-types"];
const HTTP_CLIENT_DOWNLOADS: [&str; 2] = ["http-types", "http-client"];
const DISTRIBUTED_LOCKING_DOWNLOADS: [&str; 1] = ["distributed-locking"];
const MESSAGING_DOWNLOADS: [&str; 1] = ["messaging"];
const SQL_DOWNLOADS: [&str; 1] = ["sql"];

const ERROR_MSG: &str = "invalid interface name (2): currently, slight only supports the download of 'configs', 'keyvalue', 'distributed_locking', 'messaging', 'sql', and 'http'.";

pub async fn handle_add(
    what_to_add: InterfaceAtRelease,
    folder_prefix: Option<&str>,
) -> Result<()> {
    let (interface, release) = (what_to_add.name, what_to_add.version.to_string());
    let folder_name = format!("{interface}_{release}");

    match interface.as_str() {
        "keyvalue"
        | "configs"
        | "http-server"
        | "http-client"
        | "distributed_locking"
        | "messaging"
        | "sql" => {
            maybe_recreate_dir(&format!("{}{}", folder_prefix.unwrap_or("./"), folder_name))?;
            for i in get_interface_downloads_by_name(&interface) {
                let resp = reqwest::get(format!("{GITHUB_URL}/v{release}/{i}.wit"))
                    .await?
                    .text()
                    .await?;
                let mut out = File::create(format!(
                    "{}{}/{}.wit",
                    folder_prefix.unwrap_or("./"),
                    folder_name,
                    i
                ))?;
                io::copy(&mut resp.as_bytes(), &mut out)?;
            }
        }
        _ => {
            panic!("{}", ERROR_MSG)
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

pub fn get_interface_downloads_by_name(name: &str) -> Vec<&str> {
    match name {
        _ if name.eq("keyvalue") => KEYVALUE_DOWNLOADS.to_vec(),
        _ if name.eq("configs") => CONFIGS_DOWNLOADS.to_vec(),
        _ if name.eq("http_server") => HTTP_DOWNLOADS.to_vec(),
        _ if name.eq("http_client") => HTTP_CLIENT_DOWNLOADS.to_vec(),
        _ if name.eq("distributed_locking") => DISTRIBUTED_LOCKING_DOWNLOADS.to_vec(),
        _ if name.eq("messaging") => MESSAGING_DOWNLOADS.to_vec(),
        _ if name.eq("sql") => SQL_DOWNLOADS.to_vec(),
        _ => {
            panic!("{}", ERROR_MSG)
        }
    }
}
