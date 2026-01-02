//! Generate command handler
//!
//! Generates random coordinates based on user input.

use crate::config::Config;
use crate::coord::flower::generate;
use crate::coord::{AnomalyType, Coordinates, GenerationMode};
use crate::error::Result;
use crate::format::{get_formatter, available_formats};
use crate::geo::{get_geocoder, get_ip_locator, GeoBackend};
use crate::history::History;
use crate::qrng::get_backend;
use clap::Args;
use std::str::FromStr;

/// Generate command arguments
#[derive(Args)]
pub struct GenerateArgs {
    /// Latitude
    #[arg(long)]
    pub lat: Option<f64>,

    /// Longitude
    #[arg(long)]
    pub lng: Option<f64>,

    /// Named location (geocoded)
    #[arg(long, conflicts_with_all = ["lat", "lng", "here"])]
    pub location: Option<String>,

    /// Use current location (IP geolocation)
    #[arg(long, conflicts_with_all = ["lat", "lng", "location"])]
    pub here: bool,

    /// Search radius in meters
    #[arg(long, short = 'r')]
    pub radius: Option<f64>,

    /// Generation type to display
    #[arg(long, short = 't')]
    pub r#type: Option<String>,

    /// Output format
    #[arg(long, short = 'f')]
    pub format: Option<String>,

    /// QRNG backend
    #[arg(long, short = 'b')]
    pub backend: Option<String>,

    /// Number of points for analysis
    #[arg(long, short = 'p')]
    pub points: Option<usize>,

    /// Generation mode: standard or flower_power
    #[arg(long, short = 'm')]
    pub mode: Option<String>,

    /// Include all generated points in response
    #[arg(long)]
    pub include_points: bool,

    /// Don't save to history
    #[arg(long)]
    pub no_history: bool,

    /// Write output to file
    #[arg(long, short = 'o')]
    pub output: Option<String>,

    /// List available types
    #[arg(short = 'T', long = "list-types")]
    pub list_types: bool,

    /// List available formats
    #[arg(short = 'F', long = "list-formats")]
    pub list_formats: bool,
}

/// Run the generate command
pub async fn run(args: GenerateArgs) -> Result<()> {
    // Handle list flags first
    if args.list_types {
        list_types();
        return Ok(());
    }

    if args.list_formats {
        list_formats();
        return Ok(());
    }

    // Load config
    let config = Config::load()?;

    // Determine location
    let center = if args.here {
        let ip_locator = get_ip_locator();
        let location = ip_locator.locate().await?;
        eprintln!("Using IP location: {}", location.display_name);
        Coordinates::new(location.lat, location.lng)
    } else if let Some(location_query) = &args.location {
        let geocoder = get_geocoder();
        match geocoder.geocode(location_query).await? {
            Some(location) => {
                eprintln!("Geocoded to: {}", location.display_name);
                Coordinates::new(location.lat, location.lng)
            }
            None => {
                eprintln!("Error: Could not geocode '{}'", location_query);
                std::process::exit(1);
            }
        }
    } else if let (Some(lat), Some(lng)) = (args.lat, args.lng) {
        Coordinates::new(lat, lng)
    } else {
        // Use config default or prompt
        if config.location.default_here {
            let ip_locator = get_ip_locator();
            let location = ip_locator.locate().await?;
            eprintln!("Using IP location: {}", location.display_name);
            Coordinates::new(location.lat, location.lng)
        } else {
            eprintln!("Error: No location specified. Use --lat/--lng, --location, or --here");
            std::process::exit(1);
        }
    };

    // Validate coordinates
    center.validate()?;

    // Get parameters with config defaults
    let radius = args.radius.unwrap_or(config.defaults.radius);
    let points = args.points.unwrap_or(config.defaults.points);
    let backend_name = args.backend.unwrap_or(config.defaults.backend.clone());
    let mode_str = args.mode.unwrap_or(config.defaults.mode.clone());
    let format = args.format.unwrap_or(config.defaults.format.clone());
    let anomaly_type_str = args.r#type.unwrap_or(config.defaults.anomaly_type.clone());

    // Parse mode
    let mode = GenerationMode::from_str(&mode_str)
        .map_err(|e| crate::error::Error::Config(e))?;

    // Parse anomaly type for display
    let display_type = AnomalyType::from_str(&anomaly_type_str)
        .map_err(|e| crate::error::Error::Config(e))?;

    // Get backend
    let backend = get_backend(&backend_name);

    // Generate
    let response = generate(
        center,
        radius,
        points,
        50, // grid_resolution
        args.include_points,
        mode,
        backend.name(),
        backend.as_ref(),
    )?;

    // Save to history (unless disabled)
    if !args.no_history {
        if let Ok(mut history) = History::load() {
            history.add_response(response.clone());
            let _ = history.save();
        }
    }

    // Format output
    let formatter = get_formatter(&format).ok_or_else(|| {
        crate::error::Error::Config(format!("Unknown format: {}", format))
    })?;
    let output = formatter.format(&response, display_type, &config)?;

    // Write output
    if let Some(path) = args.output {
        std::fs::write(&path, &output)?;
        eprintln!("Output written to {}", path);
    } else {
        println!("{}", output);
    }

    Ok(())
}

/// Print available anomaly types
fn list_types() {
    println!("Available anomaly types:");
    println!("  blind_spot  - Random point with no analysis");
    println!("  attractor   - Densest cluster of points");
    println!("  void        - Emptiest region");
    println!("  power       - Most statistically anomalous");
}

/// Print available output formats
fn list_formats() {
    println!("Available output formats:");
    for format in available_formats() {
        println!("  {:6} - {}", format.name, format.description);
    }
}
