//! Coordinate generation and analysis
//!
//! This module handles:
//! - Generating random points within a circle
//! - Density grid analysis
//! - Anomaly detection (attractor, void, power)
//! - Flower power multi-circle generation

pub mod anomaly;
pub mod density;
pub mod flower;
pub mod point;

use serde::{Deserialize, Serialize};

/// A geographic coordinate (latitude, longitude)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lng: f64,
}

impl Coordinates {
    /// Create new coordinates
    pub fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }

    /// Validate that coordinates are within valid ranges
    ///
    /// Latitude: -90 to 90
    /// Longitude: -180 to 180
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.lat < -90.0 || self.lat > 90.0 {
            return Err(crate::error::Error::InvalidCoordinates(format!(
                "Latitude {} is out of range [-90, 90]",
                self.lat
            )));
        }
        if self.lng < -180.0 || self.lng > 180.0 {
            return Err(crate::error::Error::InvalidCoordinates(format!(
                "Longitude {} is out of range [-180, 180]",
                self.lng
            )));
        }
        Ok(())
    }
}

/// A point with optional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub coords: Coordinates,

    /// Z-score for anomaly detection (how many std devs from expected)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z_score: Option<f64>,

    /// For power anomalies: is this an attractor (true) or void (false)?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_attractor: Option<bool>,
}

impl Point {
    /// Create a simple point from coordinates
    pub fn new(coords: Coordinates) -> Self {
        Self {
            coords,
            z_score: None,
            is_attractor: None,
        }
    }

    /// Create a point with z-score (for attractor/void)
    pub fn with_z_score(coords: Coordinates, z_score: f64) -> Self {
        Self {
            coords,
            z_score: Some(z_score),
            is_attractor: None,
        }
    }

    /// Create a power point (with z-score and attractor/void flag)
    pub fn power(coords: Coordinates, z_score: f64, is_attractor: bool) -> Self {
        Self {
            coords,
            z_score: Some(z_score),
            is_attractor: Some(is_attractor),
        }
    }
}

/// Generation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationMode {
    /// Single circle around the center point
    Standard,
    /// Seven overlapping circles (flower pattern)
    FlowerPower,
}

impl Default for GenerationMode {
    fn default() -> Self {
        Self::Standard
    }
}

impl std::str::FromStr for GenerationMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standard" => Ok(Self::Standard),
            "flower_power" | "flower-power" | "flowerpower" => Ok(Self::FlowerPower),
            _ => Err(format!("Unknown generation mode: {}", s)),
        }
    }
}

/// Anomaly types that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyType {
    /// Single random point (no analysis)
    BlindSpot,
    /// Densest cluster (most points in area)
    Attractor,
    /// Emptiest region (fewest points in area)
    Void,
    /// Most statistically anomalous (highest absolute z-score)
    Power,
}

impl std::fmt::Display for AnomalyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BlindSpot => write!(f, "blind_spot"),
            Self::Attractor => write!(f, "attractor"),
            Self::Void => write!(f, "void"),
            Self::Power => write!(f, "power"),
        }
    }
}

impl std::str::FromStr for AnomalyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "blind_spot" | "blind-spot" | "blindspot" => Ok(Self::BlindSpot),
            "attractor" => Ok(Self::Attractor),
            "void" => Ok(Self::Void),
            "power" => Ok(Self::Power),
            _ => Err(format!("Unknown anomaly type: {}", s)),
        }
    }
}

/// List all available anomaly types
pub fn available_types() -> Vec<AnomalyType> {
    vec![
        AnomalyType::BlindSpot,
        AnomalyType::Attractor,
        AnomalyType::Void,
        AnomalyType::Power,
    ]
}
