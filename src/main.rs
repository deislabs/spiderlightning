use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use opentelemetry::{
    global::{self, shutdown_tracer_provider}, runtime,
    sdk::trace as sdktrace,
    trace::{TraceError, Tracer},
};
use slight_lib::{
    cli::{Args, Commands},
    commands::{
        add::handle_add,
        new::handle_new,
        run::{handle_run, RunArgs},
        secret::handle_secret,
    },
};

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("spiderlightning")
        .install_batch(runtime::Tokio)
}

/// The entry point for slight CLI
#[tokio::main]
async fn main() -> Result<()> {
    let _tracer = init_tracer()?;
    let tracer = global::tracer("spiderlightning");

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let args = Args::parse();

    tracer
        .in_span("slight command", |_cx| async {
            match &args.command {
                Commands::Run { module } => {
                    let run_args = RunArgs {
                        module: PathBuf::from(&module.path),
                        slightfile: PathBuf::from(args.config.unwrap()),
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
            }
        })
        .await?;

    shutdown_tracer_provider();

    Ok(())
}
