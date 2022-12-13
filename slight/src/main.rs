use anyhow::Result;
use clap::Parser;
use slight_lib::{
    cli::{Args, Commands},
    commands::{add::handle_add, new::handle_new, run::handle_run, secret::handle_secret},
};

/// The entry point for slight CLI
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let args = Args::parse();

    match &args.command {
        Commands::Run { module } => handle_run(module, args.config.unwrap()).await,
        Commands::Secret { key, value } => handle_secret(key, value, args.config.unwrap()),
        Commands::Add {
            interface_at_release,
        } => handle_add(interface_at_release, None).await,
        Commands::New {
            command,
            name_at_release,
        } => handle_new(name_at_release, command).await,
    }
}
