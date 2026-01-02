//! Status command handler
//!
//! Shows server status and entropy quality.

use crate::config::Config;
use crate::entropy::run_all_tests;
use crate::error::Result;
use crate::qrng::get_backend;
use clap::Args;

/// Status command arguments
#[derive(Args)]
pub struct StatusArgs {
    /// Check a specific backend
    #[arg(long, short = 'b')]
    pub backend: Option<String>,

    /// Run entropy tests with N bytes
    #[arg(long, default_value = "10000")]
    pub entropy_bytes: usize,

    /// Check if server is running (tries to connect)
    #[arg(long)]
    pub server: bool,
}

/// Run the status command
pub async fn run(args: StatusArgs) -> Result<()> {
    let config = Config::load()?;

    // Check server status if requested
    if args.server {
        check_server_status(&config).await;
    }

    // Get backend
    let backend_name = args.backend.unwrap_or(config.defaults.backend.clone());
    let backend = get_backend(&backend_name);

    println!("q-explore v{}", env!("CARGO_PKG_VERSION"));
    println!();

    println!("Backend: {} ({})", backend.name(), backend.description());
    println!();

    // Run entropy tests
    println!("Entropy Quality Test ({} bytes):", args.entropy_bytes);
    match backend.bytes(args.entropy_bytes) {
        Ok(bytes) => {
            let results = run_all_tests(&bytes);

            let status = |score: f64| {
                if score >= 0.1 {
                    "PASS"
                } else if score >= 0.01 {
                    "MARGINAL"
                } else {
                    "FAIL"
                }
            };

            println!(
                "  Balanced (monobit):   {:.4} [{}]",
                results.balanced,
                status(results.balanced)
            );
            println!(
                "  Uniform (chi-square): {:.4} [{}]",
                results.uniform,
                status(results.uniform)
            );
            println!(
                "  Scattered (runs):     {:.4} [{}]",
                results.scattered,
                status(results.scattered)
            );
            println!();
            println!(
                "  Overall: {:.4} [{}]",
                results.overall,
                if results.all_passed() { "PASS" } else { "FAIL" }
            );
        }
        Err(e) => {
            println!("  Error: Failed to generate random bytes: {}", e);
        }
    }

    Ok(())
}

/// Check if the server is running
async fn check_server_status(config: &Config) {
    let url = format!("http://{}/api/status", config.server_addr());

    match reqwest::get(&url).await {
        Ok(response) => {
            if response.status().is_success() {
                println!("Server: RUNNING on {}", config.server_addr());
                if let Ok(body) = response.text().await {
                    if let Ok(status) = serde_json::from_str::<serde_json::Value>(&body) {
                        if let Some(version) = status.get("version").and_then(|v| v.as_str()) {
                            println!("  Version: {}", version);
                        }
                        if let Some(backend) = status.get("backend").and_then(|v| v.as_str()) {
                            println!("  Backend: {}", backend);
                        }
                    }
                }
            } else {
                println!("Server: ERROR (status {})", response.status());
            }
        }
        Err(_) => {
            println!("Server: NOT RUNNING on {}", config.server_addr());
        }
    }
    println!();
}
