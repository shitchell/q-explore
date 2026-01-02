//! URL output formatter

use crate::config::Config;
use crate::coord::flower::GenerationResponse;
use crate::coord::AnomalyType;
use crate::error::{Error, Result};
use crate::format::OutputFormatter;

/// URL formatter - outputs map URL for the selected anomaly type
pub struct UrlFormatter;

impl UrlFormatter {
    /// Format URL with optional provider override
    pub fn format_with_provider(
        &self,
        response: &GenerationResponse,
        display_type: AnomalyType,
        config: &Config,
        provider: Option<&str>,
    ) -> Result<String> {
        // Get the selected anomaly type's coordinates
        if let Some(winner) = response.winners.get(&display_type) {
            config.format_url(
                provider,
                winner.result.coords.lat,
                winner.result.coords.lng,
            )
        } else {
            Err(Error::Config(format!(
                "No result for anomaly type: {}",
                display_type
            )))
        }
    }
}

impl OutputFormatter for UrlFormatter {
    fn name(&self) -> &str {
        "url"
    }

    fn description(&self) -> &str {
        "Map URL for selected type"
    }

    fn format(
        &self,
        response: &GenerationResponse,
        display_type: AnomalyType,
        config: &Config,
    ) -> Result<String> {
        self.format_with_provider(response, display_type, config, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::flower::generate;
    use crate::coord::{Coordinates, GenerationMode};
    use crate::qrng::pseudo::SeededPseudoBackend;

    fn create_test_response() -> GenerationResponse {
        let backend = SeededPseudoBackend::new(12345);
        let center = Coordinates::new(40.7128, -74.0060);
        generate(center, 1000.0, 100, 10, false, GenerationMode::Standard, "test", &backend)
            .unwrap()
    }

    #[test]
    fn test_url_format_default_provider() {
        let formatter = UrlFormatter;
        let response = create_test_response();
        let config = Config::default();

        let output = formatter
            .format(&response, AnomalyType::Attractor, &config)
            .unwrap();

        // Default provider is Google
        assert!(output.contains("google.com/maps"));
    }

    #[test]
    fn test_url_format_with_provider() {
        let formatter = UrlFormatter;
        let response = create_test_response();
        let config = Config::default();

        let output = formatter
            .format_with_provider(&response, AnomalyType::Attractor, &config, Some("openstreetmap"))
            .unwrap();

        assert!(output.contains("openstreetmap.org"));
    }

    #[test]
    fn test_url_format_invalid_type() {
        let formatter = UrlFormatter;
        let response = create_test_response();
        let config = Config::default();

        // Blind spot should work
        let result = formatter.format(&response, AnomalyType::BlindSpot, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_url_formatter_info() {
        let formatter = UrlFormatter;
        assert_eq!(formatter.name(), "url");
        assert!(!formatter.description().is_empty());
    }
}
