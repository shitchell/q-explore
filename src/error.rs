//! Error types for q-explore

use thiserror::Error;

/// Main error type for q-explore operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("QRNG error: {0}")]
    Qrng(String),

    #[error("Invalid coordinates: {0}")]
    InvalidCoordinates(String),

    #[error("Invalid radius: {0}")]
    InvalidRadius(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Geocoding error: {0}")]
    Geocoding(String),

    #[error("Geo error: {0}")]
    Geo(String),
}

/// Result type alias for q-explore operations
pub type Result<T> = std::result::Result<T, Error>;
