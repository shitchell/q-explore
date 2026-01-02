//! Config command handler
//!
//! View and modify configuration settings.

use crate::config::Config;
use crate::error::Result;
use clap::Args;

/// Config command arguments
#[derive(Args)]
pub struct ConfigArgs {
    /// Configuration key (e.g., "defaults.backend")
    pub key: Option<String>,

    /// Value to set (if not provided, shows current value)
    pub value: Option<String>,

    /// Show config file path
    #[arg(long)]
    pub path: bool,

    /// Reset config to defaults
    #[arg(long)]
    pub reset: bool,
}

/// Run the config command
pub fn run(args: ConfigArgs) -> Result<()> {
    // Show path
    if args.path {
        let path = Config::config_path()?;
        println!("{}", path.display());
        return Ok(());
    }

    // Reset config
    if args.reset {
        let config = Config::default();
        config.save()?;
        println!("Configuration reset to defaults");
        return Ok(());
    }

    let mut config = Config::load()?;

    match (&args.key, &args.value) {
        // No arguments: show all config
        (None, None) => {
            show_all_config(&config);
        }

        // Key only: show that value
        (Some(key), None) => {
            if let Some(value) = config.get(key) {
                println!("{}", value);
            } else {
                eprintln!("Unknown config key: {}", key);
                eprintln!("\nAvailable keys:");
                for k in Config::available_keys() {
                    eprintln!("  {}", k);
                }
                std::process::exit(1);
            }
        }

        // Key and value: set the value
        (Some(key), Some(value)) => {
            config.set(key, value)?;
            config.save()?;
            println!("{} = {}", key, value);
        }

        // Value without key: not valid
        (None, Some(_)) => {
            eprintln!("Error: Must specify a key to set a value");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Display all configuration values
fn show_all_config(config: &Config) {
    println!("[defaults]");
    println!("backend = \"{}\"", config.defaults.backend);
    println!("radius = {}", config.defaults.radius);
    println!("points = {}", config.defaults.points);
    println!("format = \"{}\"", config.defaults.format);
    println!("type = \"{}\"", config.defaults.anomaly_type);
    println!("mode = \"{}\"", config.defaults.mode);
    println!();

    println!("[server]");
    println!("host = \"{}\"", config.server.host);
    println!("port = {}", config.server.port);
    println!("shutdown_timeout_secs = {}", config.server.shutdown_timeout_secs);
    println!();

    println!("[location]");
    println!("default_here = {}", config.location.default_here);
    println!();

    println!("[url]");
    println!("default = \"{}\"", config.url.default);
    println!();

    println!("[url.providers]");
    for (name, template) in &config.url.providers {
        println!("{} = \"{}\"", name, template);
    }
    println!();

    println!("[api_keys]");
    if config.api_keys.anu.is_empty() {
        println!("anu = \"\" # not configured");
    } else {
        println!("anu = \"***\" # configured");
    }
}
