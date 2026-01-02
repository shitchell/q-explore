//! Serve command handler
//!
//! Starts the HTTP server in foreground mode.

use crate::config::Config;
use crate::error::Result;
use crate::server;
use clap::Args;
use tracing::info;
use tracing_subscriber::EnvFilter;

/// Serve command arguments
#[derive(Args)]
pub struct ServeArgs {
    /// Host address to bind to
    #[arg(long)]
    pub host: Option<String>,

    /// Port to listen on
    #[arg(long, short = 'p')]
    pub port: Option<u16>,
}

/// Run the serve command
pub async fn run(args: ServeArgs) -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Load and optionally override config
    let mut config = Config::load()?;

    if let Some(host) = args.host {
        config.server.host = host;
    }
    if let Some(port) = args.port {
        config.server.port = port;
    }

    info!(
        "Starting q-explore server v{} on {}",
        env!("CARGO_PKG_VERSION"),
        config.server_addr()
    );

    // Run the server
    server::run(config).await
}
