//! q-explore: Quantum Random Coordinate Generator
//!
//! A library and CLI tool for generating random geographic coordinates using
//! quantum random number generators (QRNG).
//!
//! ## Features
//!
//! - Multiple QRNG backends (pseudo, ANU, etc.)
//! - Uniform point-in-circle generation with sqrt correction
//! - Anomaly detection (attractor, void, power)
//! - Flower power multi-circle generation
//! - HTTP API + CLI interface
//!
//! ## Quick Start
//!
//! ```rust
//! use q_explore::qrng::pseudo::PseudoBackend;
//! use q_explore::coord::{Coordinates, point};
//!
//! let backend = PseudoBackend::new();
//! let center = Coordinates::new(40.7128, -74.0060); // NYC
//! let radius = 1000.0; // 1 km
//!
//! // Generate a single random point
//! let point = point::generate_point_in_circle(center, radius, &backend).unwrap();
//! println!("Random point: {:?}", point);
//!
//! // Generate many points for analysis
//! let points = point::generate_points_in_circle(center, radius, 10000, &backend).unwrap();
//! println!("Generated {} points", points.len());
//! ```

pub mod cli;
pub mod config;
pub mod constants;
pub mod coord;
pub mod entropy;
pub mod error;
pub mod format;
pub mod geo;
pub mod history;
pub mod qrng;
pub mod server;

// Re-export commonly used types
pub use config::Config;
pub use coord::{AnomalyType, Coordinates, GenerationMode, Point};
pub use entropy::EntropyTestResults;
pub use error::{Error, Result};
