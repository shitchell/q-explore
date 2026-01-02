//! q-explore CLI entry point
//!
//! Quantum random coordinate generator - CLI + web app

use q_explore::cli;

#[tokio::main]
async fn main() {
    if let Err(e) = cli::run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
