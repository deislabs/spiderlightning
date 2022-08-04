use std::fs::OpenOptions;

use crate::commands::{run::handle_run, secret::handle_secret};
use anyhow::Result;
use clap::{Parser, Subcommand};
use spiderlightning::core::slightfile::TomlFile;

mod commands;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
    #[clap(short, long, value_parser)]
    config: String,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run slight providing a config and a module
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

/// The entry point for slight CLI
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let args = Args::parse();
    let toml_file_path = args.config;
    let mut toml_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&toml_file_path)?;
    let toml_file_contents = std::fs::read_to_string(&toml_file_path)?;
    let mut toml = toml::from_str::<TomlFile>(&toml_file_contents)?;

    match &args.command {
        Commands::Run { module } => handle_run(module, &toml, &toml_file_path).await,
        Commands::Secret { key, value } => handle_secret(key, value, &mut toml, &mut toml_file),
    }
}
