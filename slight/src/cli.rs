use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
    #[clap(short, long, value_parser)]
    pub config: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
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
    /// Download a SpiderLightning interface
    Add {
        #[clap(short, long, value_parser)]
        interface_at_release: String,
    },
    /// Start a new Slight project
    New {
        #[clap(subcommand)]
        command: Templates,
        #[clap(short, long, value_parser)]
        name_at_release: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum Templates {
    /// Start a new C Slight project
    C,
    /// Start a new Rust Slight Project
    Rust,
}

impl std::fmt::Display for Templates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Templates::C => "c",
                Templates::Rust => "rust",
            }
        )
    }
}
