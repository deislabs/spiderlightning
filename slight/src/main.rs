use anyhow::Result;
use clap::{Parser, Subcommand};
use slight_lib::commands::{run::handle_run, secret::handle_secret};

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

    match &args.command {
        Commands::Run { module } => handle_run(module, &args.config).await,
        Commands::Secret { key, value } => handle_secret(key, value, &args.config),
    }
}
