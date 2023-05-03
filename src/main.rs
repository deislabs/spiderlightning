use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use slight_lib::{
    cli::{Args, Commands},
    commands::{
        add::handle_add,
        buildjs::handle_buildjs,
        new::handle_new,
        run::{handle_run, RunArgs},
        secret::handle_secret,
    },
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
        Commands::Run {
            module,
            link_all_capabilities,
        } => {
            let run_args = RunArgs {
                module: PathBuf::from(&module.path),
                slightfile: PathBuf::from(args.config.unwrap()),
                link_all_capabilities: *link_all_capabilities,
                ..Default::default()
            };
            handle_run(run_args).await
        }
        Commands::Secret { key, value } => handle_secret(key, value, args.config.unwrap()),
        Commands::Add {
            interface_at_release,
        } => handle_add(interface_at_release.to_owned(), None).await,
        Commands::New {
            command,
            name_at_release,
        } => handle_new(name_at_release, command).await,
        Commands::Buildjs {
            src,
            engine,
            output,
        } => handle_buildjs(&engine.path, src, &output.path),
    }
}
