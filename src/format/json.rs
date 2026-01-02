//! JSON output formatter

use crate::config::Config;
use crate::coord::flower::GenerationResponse;
use crate::coord::AnomalyType;
use crate::error::Result;
use crate::format::OutputFormatter;

/// JSON formatter - outputs full response as pretty-printed JSON
pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn name(&self) -> &str {
        "json"
    }

    fn description(&self) -> &str {
        "Full JSON response"
    }

    fn format(
        &self,
        response: &GenerationResponse,
        _display_type: AnomalyType,
        _config: &Config,
    ) -> Result<String> {
        Ok(serde_json::to_string_pretty(response)?)
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
    fn test_json_format() {
        let formatter = JsonFormatter;
        let response = create_test_response();
        let config = Config::default();

        let output = formatter
            .format(&response, AnomalyType::Attractor, &config)
            .unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.get("id").is_some());
        assert!(parsed.get("request").is_some());
        assert!(parsed.get("circles").is_some());
        assert!(parsed.get("winners").is_some());
    }

    #[test]
    fn test_json_formatter_info() {
        let formatter = JsonFormatter;
        assert_eq!(formatter.name(), "json");
        assert!(!formatter.description().is_empty());
    }
}
