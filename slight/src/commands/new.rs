use anyhow::Result;
use flate2::bufread::GzDecoder;
use std::io::Write;
use std::{
    fs::{read_to_string, File},
    io::BufReader,
};
use tar::Archive;

use crate::cli::Templates;

use super::add::handle_add;

pub async fn handle_new(name_at_release: &str, template: &Templates) -> Result<()> {
    let (project_name, release) = if !name_at_release.contains('@') {
        panic!(
            "invalid usage: to start a new project, say `slight new -n <project-name>@<release-tag> <some-template>`"
        );
        // TODO: In the future, let's support omitting the release tag to download the latest release
    } else {
        let find_at = name_at_release.find('@').unwrap();
        // ^^^ fine to unwrap, we are guaranteed to have a '@' at this point.
        (&name_at_release[..find_at], &name_at_release[find_at + 1..])
    };

    let resp = reqwest::get(format!(
        "https://github.com/deislabs/spiderlightning/releases/download/{}/{}-template.tar.gz",
        release, template
    ))
    .await?
    .bytes()
    .await?;
    Archive::new(GzDecoder::new(BufReader::new(resp.as_ref()))).unpack("./")?;

    match template {
        Templates::C => setup_c_template(project_name, release)?,
        Templates::Rust => setup_rust_template(project_name, release)?,
    };

    handle_add(
        &format!("kv@{}", release),
        Some(&format!("./{}/wit/", project_name)),
    )
    .await?;

    Ok(())
}

fn setup_c_template(project_name: &str, release: &str) -> Result<()> {
    let mut makefile_s = read_to_string("./c/Makefile")?;
    makefile_s = makefile_s.replace("{{project-name}}", project_name);
    makefile_s = makefile_s.replace("{{release}}", release);

    let mut makefile_f = File::create("./c/Makefile")?;
    write!(makefile_f, "{}", makefile_s)?;
    drop(makefile_f);

    std::fs::rename("c", project_name)?;

    Ok(())
}

fn setup_rust_template(project_name: &str, release: &str) -> Result<()> {
    let mut cargo_s = read_to_string("./rust/Cargo.toml")?;
    cargo_s = cargo_s.replace("{{project-name}}", project_name);
    let mut cargo_f = File::create("./rust/Cargo.toml")?;
    write!(cargo_f, "{}", cargo_s)?;

    let mut main_s = read_to_string("./rust/src/main.rs")?;
    main_s = main_s.replace("{{release}}", release);
    let mut main_f = File::create("./rust/src/main.rs")?;
    write!(main_f, "{}", main_s)?;
    drop(cargo_f);
    drop(main_f);

    std::fs::rename("rust", project_name)?;

    Ok(())
}
