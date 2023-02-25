use anyhow::{bail, Context, Result};
use flate2::bufread::GzDecoder;
use slight_core::interface_parser::InterfaceAtRelease;
use std::io::Write;
use std::{
    fs::{read_to_string, File},
    io::BufReader,
};
use tar::Archive;

use crate::cli::Templates;

use super::add::handle_add;

pub async fn handle_new(name_at_release: &InterfaceAtRelease, template: &Templates) -> Result<()> {
    let project_name = name_at_release.name.to_owned();

    // check project_name is not C or Rust
    if project_name == "c" || project_name == "rust" {
        bail!("project name cannot be c or rust");
    }

    let resp = reqwest::get(format!(
        "https://github.com/deislabs/spiderlightning/releases/download/v{release}/{template}-template.tar.gz"
    ))
    .await?;
    if resp.status() == 404 {
        bail!("could not find template {} for release {}, pleases see all releases in https://github.com/deislabs/spiderlightning/releases", template, release);
    }

    let resp = resp.bytes().await?;

    Archive::new(GzDecoder::new(BufReader::new(resp.as_ref()))).unpack("./")?;

    match template {
        Templates::C => setup_c_template(&project_name, &release)?,
        Templates::Rust => setup_rust_template(&project_name, &release)?,
    };

    handle_add(
        InterfaceAtRelease::new("keyvalue", &release),
        Some(&format!("./{project_name}/wit/")),
    )
    .await?;

    Ok(())
}

fn setup_c_template(project_name: &str, release: &str) -> Result<()> {
    let mut makefile_s = read_to_string("./c/Makefile")?;
    makefile_s = makefile_s.replace("{{project-name}}", project_name);
    makefile_s = makefile_s.replace("{{release}}", release);

    let mut makefile_f = File::create("./c/Makefile")?;
    write!(makefile_f, "{makefile_s}")?;
    drop(makefile_f);

    better_rename("c", project_name)?;

    Ok(())
}

fn setup_rust_template(project_name: &str, release: &str) -> Result<()> {
    let mut cargo_s = read_to_string("./rust/Cargo.toml")?;
    cargo_s = cargo_s.replace("{{project-name}}", project_name);
    let mut cargo_f = File::create("./rust/Cargo.toml")?;
    write!(cargo_f, "{cargo_s}")?;

    let mut main_s = read_to_string("./rust/src/main.rs")?;
    main_s = main_s.replace("{{release}}", release);
    let mut main_f = File::create("./rust/src/main.rs")?;
    write!(main_f, "{main_s}")?;
    drop(cargo_f);
    drop(main_f);

    better_rename("rust", project_name)?;

    Ok(())
}

fn better_rename(from: &str, to: &str) -> Result<()> {
    std::fs::rename(from, to).with_context(|| format!("could not create project: {to}"))?;
    Ok(())
}
