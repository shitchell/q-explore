//! HTTP server for q-explore
//!
//! Provides REST API endpoints for coordinate generation.

pub mod routes;
pub mod state;

use crate::config::Config;
use crate::error::Result;
use routes::create_router;
use state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

/// Start the HTTP server
///
/// # Arguments
/// * `config` - Server configuration
///
/// # Returns
/// Never returns unless the server shuts down
pub async fn run(config: Config) -> Result<()> {
    let addr: SocketAddr = config.server_addr().parse().map_err(|e| {
        crate::error::Error::Server(format!("Invalid server address: {}", e))
    })?;

    let state = Arc::new(AppState::new(config));
    let app = create_router(state);

    info!("Starting server on {}", addr);

    let listener = TcpListener::bind(addr).await.map_err(|e| {
        crate::error::Error::Server(format!("Failed to bind to {}: {}", addr, e))
    })?;

    axum::serve(listener, app).await.map_err(|e| {
        crate::error::Error::Server(format!("Server error: {}", e))
    })?;

    Ok(())
}

/// Start the HTTP server with a specific address
///
/// Useful for tests or when you want to override config
pub async fn run_on(addr: &str, config: Config) -> Result<()> {
    let addr: SocketAddr = addr.parse().map_err(|e| {
        crate::error::Error::Server(format!("Invalid server address: {}", e))
    })?;

    let state = Arc::new(AppState::new(config));
    let app = create_router(state);

    info!("Starting server on {}", addr);

    let listener = TcpListener::bind(addr).await.map_err(|e| {
        crate::error::Error::Server(format!("Failed to bind to {}: {}", addr, e))
    })?;

    axum::serve(listener, app).await.map_err(|e| {
        crate::error::Error::Server(format!("Server error: {}", e))
    })?;

    Ok(())
}
