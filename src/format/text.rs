//! Human-readable text output formatter

use crate::config::Config;
use crate::coord::flower::GenerationResponse;
use crate::coord::AnomalyType;
use crate::error::Result;
use crate::format::OutputFormatter;

/// Text formatter - outputs human-readable summary
pub struct TextFormatter;

impl OutputFormatter for TextFormatter {
    fn name(&self) -> &str {
        "text"
    }

    fn description(&self) -> &str {
        "Human-readable text"
    }

    fn format(
        &self,
        response: &GenerationResponse,
        _display_type: AnomalyType,
        _config: &Config,
    ) -> Result<String> {
        let mut output = String::new();

        // Header
        output.push_str(&format!("q-explore generation ({})\n", response.id));
        output.push_str(&format!(
            "Center: ({:.6}, {:.6})\n",
            response.request.lat, response.request.lng
        ));
        output.push_str(&format!("Radius: {}m\n", response.request.radius));
        output.push_str(&format!("Mode: {:?}\n", response.request.mode));
        output.push_str(&format!("Backend: {}\n\n", response.request.backend));

        // Results
        output.push_str("Results:\n");
        for (anomaly_type, winner) in &response.winners {
            let point = &winner.result;
            output.push_str(&format!(
                "  {}: ({:.6}, {:.6}){}\n",
                anomaly_type, point.coords.lat, point.coords.lng, point.format_z_score()
            ));
        }

        // Entropy quality if available
        if let Some(quality) = &response.metadata.entropy_quality {
            output.push_str("\nEntropy Quality:\n");
            output.push_str(&format!("  Balanced: {:.2}\n", quality.balanced));
            output.push_str(&format!("  Uniform: {:.2}\n", quality.uniform));
            output.push_str(&format!("  Scattered: {:.2}\n", quality.scattered));
        }

        Ok(output)
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
    fn test_text_format() {
        let formatter = TextFormatter;
        let response = create_test_response();
        let config = Config::default();

        let output = formatter
            .format(&response, AnomalyType::Attractor, &config)
            .unwrap();

        assert!(output.contains("q-explore generation"));
        assert!(output.contains("Center:"));
        assert!(output.contains("Radius:"));
        assert!(output.contains("Results:"));
        assert!(output.contains("attractor"));
        assert!(output.contains("void"));
    }

    #[test]
    fn test_text_formatter_info() {
        let formatter = TextFormatter;
        assert_eq!(formatter.name(), "text");
        assert!(!formatter.description().is_empty());
    }
}
