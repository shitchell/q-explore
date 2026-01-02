//! CLI command handlers
//!
//! Each subcommand has its own module with handler functions.

pub mod config;
pub mod generate;
pub mod history;
pub mod serve;
pub mod status;

use clap::{Parser, Subcommand};

/// Quantum random coordinate generator
#[derive(Parser)]
#[command(name = "q-explore")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate random coordinates
    Generate(generate::GenerateArgs),

    /// Start web server (foreground)
    Serve(serve::ServeArgs),

    /// Manage configuration
    Config(config::ConfigArgs),

    /// Show server/entropy status
    Status(status::StatusArgs),

    /// View and manage history
    History(history::HistoryArgs),
}

/// Run the CLI
pub async fn run() -> crate::error::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate(args) => generate::run(args).await,
        Commands::Serve(args) => serve::run(args).await,
        Commands::Config(args) => config::run(args),
        Commands::Status(args) => status::run(args).await,
        Commands::History(args) => history::run(args).await,
    }
}
