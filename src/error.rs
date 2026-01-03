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

impl Error {
    /// Get the error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            Error::InvalidCoordinates(_) => "INVALID_COORDINATES",
            Error::InvalidRadius(_) => "INVALID_RADIUS",
            Error::Qrng(_) => "QRNG_ERROR",
            Error::Config(_) => "CONFIG_ERROR",
            Error::Io(_) => "IO_ERROR",
            Error::Http(_) => "HTTP_ERROR",
            Error::Json(_) => "JSON_ERROR",
            Error::Server(_) => "SERVER_ERROR",
            Error::Geocoding(_) => "GEOCODING_ERROR",
            Error::Geo(_) => "GEO_ERROR",
        }
    }
}

/// Result type alias for q-explore operations
pub type Result<T> = std::result::Result<T, Error>;
