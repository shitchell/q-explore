//! GPX output formatter

use crate::config::Config;
use crate::coord::flower::GenerationResponse;
use crate::coord::AnomalyType;
use crate::error::Result;
use crate::format::OutputFormatter;

/// GPX formatter - outputs GPX waypoint file
pub struct GpxFormatter;

impl OutputFormatter for GpxFormatter {
    fn name(&self) -> &str {
        "gpx"
    }

    fn description(&self) -> &str {
        "GPX waypoint file"
    }

    fn format(
        &self,
        response: &GenerationResponse,
        _display_type: AnomalyType,
        _config: &Config,
    ) -> Result<String> {
        let mut gpx = String::new();

        // XML header
        gpx.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        gpx.push('\n');
        gpx.push_str(r#"<gpx version="1.1" creator="q-explore">"#);
        gpx.push('\n');

        // Metadata
        gpx.push_str("  <metadata>\n");
        gpx.push_str(&format!("    <name>q-explore generation {}</name>\n", response.id));
        gpx.push_str(&format!("    <time>{}</time>\n", response.metadata.timestamp));
        gpx.push_str("  </metadata>\n");

        // Center waypoint
        gpx.push_str(&format!(
            r#"  <wpt lat="{}" lon="{}">"#,
            response.request.lat, response.request.lng
        ));
        gpx.push('\n');
        gpx.push_str("    <name>Center</name>\n");
        gpx.push_str(&format!(
            "    <desc>Origin point, radius: {}m</desc>\n",
            response.request.radius
        ));
        gpx.push_str("  </wpt>\n");

        // Result waypoints
        for (anomaly_type, winner) in &response.winners {
            let point = &winner.result;
            gpx.push_str(&format!(
                r#"  <wpt lat="{}" lon="{}">"#,
                point.coords.lat, point.coords.lng
            ));
            gpx.push('\n');

            // Capitalize first letter of anomaly type
            let name = format!("{}", anomaly_type);
            let name = name
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if i == 0 {
                        c.to_uppercase().next().unwrap_or(c)
                    } else {
                        c
                    }
                })
                .collect::<String>();
            gpx.push_str(&format!("    <name>{}</name>\n", name));

            if let Some(z) = point.z_score {
                gpx.push_str(&format!("    <desc>z-score: {:.2}</desc>\n", z));
            }

            // Add symbol based on type
            let symbol = match anomaly_type {
                AnomalyType::Attractor => "attraction",
                AnomalyType::Void => "void",
                AnomalyType::Power => "star",
                AnomalyType::BlindSpot => "random",
            };
            gpx.push_str(&format!("    <sym>{}</sym>\n", symbol));

            gpx.push_str("  </wpt>\n");
        }

        gpx.push_str("</gpx>\n");
        Ok(gpx)
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
    fn test_gpx_format() {
        let formatter = GpxFormatter;
        let response = create_test_response();
        let config = Config::default();

        let output = formatter
            .format(&response, AnomalyType::Attractor, &config)
            .unwrap();

        // Verify GPX structure
        assert!(output.contains(r#"<?xml version="1.0""#));
        assert!(output.contains(r#"<gpx version="1.1""#));
        assert!(output.contains("<wpt"));
        assert!(output.contains("<name>"));
        assert!(output.contains("</gpx>"));
        assert!(output.contains("Center"));
    }

    #[test]
    fn test_gpx_formatter_info() {
        let formatter = GpxFormatter;
        assert_eq!(formatter.name(), "gpx");
        assert!(!formatter.description().is_empty());
    }
}
