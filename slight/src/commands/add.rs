use std::{
    fs::{create_dir_all, remove_dir_all, File},
    io::{self, ErrorKind},
    process::Command,
};

use anyhow::Result;

const GITHUB_URL: &str = "https://github.com/deislabs/spiderlightning/releases/download";

const KEYVALUE_DOWNLOADS: [&str; 1] = ["keyvalue"];
const CONFIGS_DOWNLOADS: [&str; 1] = ["configs"];
const HTTP_DOWNLOADS: [&str; 3] = ["http-server", "http-handler", "http-types"];
const HTTP_CLIENT_DOWNLOADS: [&str; 2] = ["http-types", "http-client"];
const DISTRIBUTED_LOCKING_DOWNLOADS: [&str; 1] = ["distributed-locking"];
const MESSAGING_DOWNLOADS: [&str; 1] = ["messaging"];

pub async fn handle_add(what_to_add: &str, folder_prefix: Option<&str>) -> Result<()> {
    let (interface, mut release, mut folder_name) = if !what_to_add.contains('@') {
        panic!("invalid usage: to add an interface to your project, say `slight add -i <interface-name>@<release-tag>`");
        // TODO: In the future, let's support omitting the release tag to download the latest release
    } else {
        let find_at = what_to_add.find('@').unwrap();
        // ^^^ fine to unwrap, we are guaranteed to have a '@' at this point.
        (
            &what_to_add[..find_at],
            what_to_add[find_at + 1..].to_string(),
            what_to_add.replace('@', "_"),
        )
    };

    let output = Command::new("slight")
        .arg("--version")
        .output()
        .expect("failed to execute process");

    let version = String::from_utf8_lossy(&output.stdout);
    let version = version.replace("slight", "").trim().to_string();

    // if version is diff. from release, panic
    release = if !version.eq(&release) {
        // println that we are using release equal to version instead
        println!("slight version {} is different from the release you are trying to add. slight will use version v{} instead.", release, version);
        folder_name = folder_name.replace(&release, &format!("v{}", version));
        format!("v{}", version)
    } else {
        release
    };

    // change folder name to have new release

    match what_to_add {
        _ if interface.eq("keyvalue")
            | interface.eq("configs")
            | interface.eq("http-server")
            | interface.eq("http-client")
            | interface.eq("distributed_locking")
            | interface.eq("messaging") =>
        {
            maybe_recreate_dir(&format!("{}{}", folder_prefix.unwrap_or("./"), folder_name))?;
            for i in get_interface_downloads_by_name(interface) {
                let resp = reqwest::get(format!("{GITHUB_URL}/{release}/{i}.wit"))
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
            panic!("invalid interface name (1): currently, slight only supports the download of 'configs', 'keyvalue', 'distributed_locking', 'messaging', and 'http'.")
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
        _ if name.eq("keyvalue") => KEYVALUE_DOWNLOADS.to_vec(),
        _ if name.eq("configs") => CONFIGS_DOWNLOADS.to_vec(),
        _ if name.eq("http_server") => HTTP_DOWNLOADS.to_vec(),
        _ if name.eq("http_client") => HTTP_CLIENT_DOWNLOADS.to_vec(),
        _ if name.eq("distributed_locking") => DISTRIBUTED_LOCKING_DOWNLOADS.to_vec(),
        _ if name.eq("messaging") => MESSAGING_DOWNLOADS.to_vec(),
        _ => {
            panic!("invalid interface name (2): currently, slight only supports the download of 'configs', 'keyvalue', 'distributed_locking', 'messaging', and 'http'.")
        }
    }
}
