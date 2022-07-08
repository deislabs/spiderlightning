use std::fs::OpenOptions;

use anyhow::Result;
use clap::{Parser, Subcommand};
use wasi_cloud_cli::commands::{run::handle_run, secret::handle_secret};

const DEFAULT_CONFIG_FILEPATH: &str = "./wc-config.toml";

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
    #[clap(short, long, value_parser)]
    config: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run wasi-cloud providing a config and a module
    Run {
        #[clap(short, long, value_parser)]
        module: String,
    },
    /// Add a secret to the application
    Secret {
        #[clap(short, long, value_parser)]
        key: String,
        #[clap(short, long, value_parser)]
        value: String,
    },
}

/// The entry point for wasi-cloud CLI
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let toml_file_path = if let Some(c) = &args.config {
        c
    } else {
        DEFAULT_CONFIG_FILEPATH
    };

    let mut toml_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(toml_file_path)?;
    let toml_file_contents = std::fs::read_to_string(toml_file_path)?;
    let mut toml = toml::from_str(&toml_file_contents)?;

    match &args.command {
        Commands::Run { module } => handle_run(&module, &toml),
        Commands::Secret { key, value } => handle_secret(&key, &value, &mut toml, &mut toml_file),
    }
}
