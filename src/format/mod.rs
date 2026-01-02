//! Output formatters
//!
//! Provides trait-based output formatting for generation results.

pub mod gpx;
pub mod json;
pub mod text;
pub mod url;

use crate::config::Config;
use crate::coord::flower::GenerationResponse;
use crate::coord::AnomalyType;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Information about an output format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    /// Format name
    pub name: String,
    /// Format description
    pub description: String,
}

/// Trait for output formatters
pub trait OutputFormatter: Send + Sync {
    /// Get the format name
    fn name(&self) -> &str;

    /// Get the format description
    fn description(&self) -> &str;

    /// Format the generation response
    ///
    /// # Arguments
    /// * `response` - The generation response to format
    /// * `display_type` - The anomaly type to highlight (for url format)
    /// * `config` - Application config (for url providers, etc.)
    fn format(
        &self,
        response: &GenerationResponse,
        display_type: AnomalyType,
        config: &Config,
    ) -> Result<String>;
}

/// Get a formatter by name
pub fn get_formatter(name: &str) -> Option<Box<dyn OutputFormatter>> {
    match name.to_lowercase().as_str() {
        "json" => Some(Box::new(json::JsonFormatter)),
        "text" => Some(Box::new(text::TextFormatter)),
        "gpx" => Some(Box::new(gpx::GpxFormatter)),
        "url" => Some(Box::new(url::UrlFormatter)),
        _ => None,
    }
}

/// List all available formatters
pub fn available_formats() -> Vec<FormatInfo> {
    vec![
        FormatInfo {
            name: "json".to_string(),
            description: "Full JSON response".to_string(),
        },
        FormatInfo {
            name: "text".to_string(),
            description: "Human-readable text".to_string(),
        },
        FormatInfo {
            name: "gpx".to_string(),
            description: "GPX waypoint file".to_string(),
        },
        FormatInfo {
            name: "url".to_string(),
            description: "Map URL for selected type".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_formatter() {
        assert!(get_formatter("json").is_some());
        assert!(get_formatter("text").is_some());
        assert!(get_formatter("gpx").is_some());
        assert!(get_formatter("url").is_some());
        assert!(get_formatter("unknown").is_none());
    }

    #[test]
    fn test_get_formatter_case_insensitive() {
        assert!(get_formatter("JSON").is_some());
        assert!(get_formatter("Text").is_some());
        assert!(get_formatter("GPX").is_some());
    }

    #[test]
    fn test_available_formats() {
        let formats = available_formats();
        assert_eq!(formats.len(), 4);
        assert!(formats.iter().any(|f| f.name == "json"));
        assert!(formats.iter().any(|f| f.name == "text"));
        assert!(formats.iter().any(|f| f.name == "gpx"));
        assert!(formats.iter().any(|f| f.name == "url"));
    }
}
