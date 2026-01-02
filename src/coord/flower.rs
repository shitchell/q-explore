//! Flower Power multi-circle generation
//!
//! Generates 7 overlapping circles in a flower pattern (1 center + 6 petals)
//! and finds the strongest anomalies across all circles.

use crate::constants::geo::METERS_PER_DEGREE_LAT;
use crate::coord::anomaly::{analyze_circle, find_all_winners, CircleResults, DEFAULT_POINT_COUNT};
use crate::coord::density::DEFAULT_GRID_RESOLUTION;
use crate::coord::{AnomalyType, Coordinates, GenerationMode, Point};
use crate::error::Result;
use crate::qrng::QrngBackend;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::PI;

/// Minimum radius in meters for flower power mode
pub const FLOWER_POWER_MIN_RADIUS: f64 = 3000.0;

/// Number of petal circles (excluding center)
pub const PETAL_COUNT: usize = 6;

/// Full generation response with all circles and winners
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    /// Unique ID for this generation
    pub id: String,

    /// Original request parameters
    pub request: GenerationRequest,

    /// Results for each circle (1 for standard, 7 for flower power)
    pub circles: Vec<CircleResults>,

    /// Winners for each anomaly type (across all circles)
    pub winners: HashMap<AnomalyType, WinnerResult>,

    /// Metadata about the generation
    pub metadata: GenerationMetadata,
}

/// Request parameters for generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRequest {
    pub lat: f64,
    pub lng: f64,
    pub radius: f64,
    pub points: usize,
    pub backend: String,
    pub mode: GenerationMode,
    pub include_points: bool,
}

/// Winner result pointing to a specific circle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinnerResult {
    /// ID of the winning circle
    pub circle_id: String,
    /// The winning point/result
    pub result: Point,
}

/// Metadata about the generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    /// When this was generated
    pub timestamp: String,
    /// Entropy quality scores (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entropy_quality: Option<crate::entropy::EntropyTestResults>,
}

/// Generate coordinates using the specified mode
///
/// # Arguments
/// * `center` - Center coordinates
/// * `radius` - Radius in meters
/// * `points` - Number of points per circle
/// * `grid_resolution` - Resolution of density grid
/// * `include_points` - Whether to include all generated points
/// * `mode` - Standard or FlowerPower
/// * `backend_name` - Name of the QRNG backend
/// * `rng` - QRNG backend instance
///
/// # Returns
/// GenerationResponse with all circles and winners
pub fn generate(
    center: Coordinates,
    radius: f64,
    points: usize,
    grid_resolution: usize,
    include_points: bool,
    mode: GenerationMode,
    backend_name: &str,
    rng: &dyn QrngBackend,
) -> Result<GenerationResponse> {
    let circles = match mode {
        GenerationMode::Standard => generate_standard(
            center,
            radius,
            points,
            grid_resolution,
            include_points,
            rng,
        )?,
        GenerationMode::FlowerPower => generate_flower_power(
            center,
            radius,
            points,
            grid_resolution,
            include_points,
            rng,
        )?,
    };

    // Find winners across all circles
    let winner_map = find_all_winners(&circles);
    let winners: HashMap<AnomalyType, WinnerResult> = winner_map
        .into_iter()
        .map(|(anomaly_type, (circle_id, point))| {
            (
                anomaly_type,
                WinnerResult {
                    circle_id,
                    result: point,
                },
            )
        })
        .collect();

    Ok(GenerationResponse {
        id: uuid::Uuid::new_v4().to_string(),
        request: GenerationRequest {
            lat: center.lat,
            lng: center.lng,
            radius,
            points,
            backend: backend_name.to_string(),
            mode,
            include_points,
        },
        circles,
        winners,
        metadata: GenerationMetadata {
            timestamp: chrono::Utc::now().to_rfc3339(),
            entropy_quality: None, // Can be added if we run entropy tests
        },
    })
}

/// Generate using standard mode (single circle)
fn generate_standard(
    center: Coordinates,
    radius: f64,
    points: usize,
    grid_resolution: usize,
    include_points: bool,
    rng: &dyn QrngBackend,
) -> Result<Vec<CircleResults>> {
    let circle = analyze_circle(
        "center",
        center,
        radius,
        points,
        grid_resolution,
        include_points,
        rng,
    )?;
    Ok(vec![circle])
}

/// Generate using flower power mode (7 circles)
///
/// Layout: Center circle surrounded by 6 petals at 60-degree intervals.
/// Each petal is offset by half the radius, creating nice overlap.
fn generate_flower_power(
    center: Coordinates,
    radius: f64,
    points: usize,
    grid_resolution: usize,
    include_points: bool,
    rng: &dyn QrngBackend,
) -> Result<Vec<CircleResults>> {
    // Sub-radius for each circle (half of main radius gives good overlap)
    let sub_radius = radius / 2.0;

    // Calculate petal centers
    let petal_centers = calculate_petal_centers(center, sub_radius);

    let mut circles = Vec::with_capacity(7);

    // Center circle
    circles.push(analyze_circle(
        "center",
        center,
        sub_radius,
        points,
        grid_resolution,
        include_points,
        rng,
    )?);

    // Petal circles
    for (i, &petal_center) in petal_centers.iter().enumerate() {
        circles.push(analyze_circle(
            &format!("petal_{}", i),
            petal_center,
            sub_radius,
            points,
            grid_resolution,
            include_points,
            rng,
        )?);
    }

    Ok(circles)
}

/// Calculate the centers of the 6 petal circles
///
/// Petals are arranged in a hexagonal pattern around the center,
/// each offset by `offset_distance` at 60-degree intervals.
fn calculate_petal_centers(center: Coordinates, offset_distance: f64) -> [Coordinates; 6] {
    let meters_per_deg_lng = METERS_PER_DEGREE_LAT * (center.lat * PI / 180.0).cos();

    let mut petals = [center; 6];

    for i in 0..6 {
        let angle = (i as f64) * PI / 3.0; // 60 degrees each

        let delta_lat = (offset_distance * angle.cos()) / METERS_PER_DEGREE_LAT;
        let delta_lng = (offset_distance * angle.sin()) / meters_per_deg_lng;

        petals[i] = Coordinates::new(center.lat + delta_lat, center.lng + delta_lng);
    }

    petals
}

/// Convenience function to generate with defaults
pub fn generate_with_defaults(
    center: Coordinates,
    radius: f64,
    mode: GenerationMode,
    rng: &dyn QrngBackend,
) -> Result<GenerationResponse> {
    generate(
        center,
        radius,
        DEFAULT_POINT_COUNT,
        DEFAULT_GRID_RESOLUTION,
        false,
        mode,
        rng.name(),
        rng,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::point::haversine_distance;
    use crate::qrng::pseudo::SeededPseudoBackend;

    #[test]
    fn test_calculate_petal_centers() {
        let center = Coordinates::new(40.7128, -74.0060);
        let offset = 1000.0; // 1 km

        let petals = calculate_petal_centers(center, offset);

        // Should have 6 petals
        assert_eq!(petals.len(), 6);

        // Each petal should be approximately offset_distance from center
        for petal in &petals {
            let distance = haversine_distance(center, *petal);
            assert!(
                (distance - offset).abs() < 10.0,
                "Petal at distance {} should be ~{} from center",
                distance,
                offset
            );
        }

        // Petals should be roughly equidistant from each other
        for i in 0..6 {
            let next = (i + 1) % 6;
            let dist = haversine_distance(petals[i], petals[next]);
            // At 60 degrees apart with same radius, distance should be ~offset
            assert!(
                (dist - offset).abs() < 50.0,
                "Distance between adjacent petals {} should be ~{}",
                dist,
                offset
            );
        }
    }

    #[test]
    fn test_generate_standard() {
        let backend = SeededPseudoBackend::new(42);
        let center = Coordinates::new(40.7128, -74.0060);

        let response = generate(
            center,
            1000.0,
            1000,
            50,
            false,
            GenerationMode::Standard,
            "pseudo",
            &backend,
        )
        .unwrap();

        // Should have 1 circle
        assert_eq!(response.circles.len(), 1);
        assert_eq!(response.circles[0].id, "center");

        // Should have all anomaly types in winners
        assert!(response.winners.contains_key(&AnomalyType::BlindSpot));
        assert!(response.winners.contains_key(&AnomalyType::Attractor));
        assert!(response.winners.contains_key(&AnomalyType::Void));
        assert!(response.winners.contains_key(&AnomalyType::Power));
    }

    #[test]
    fn test_generate_flower_power() {
        let backend = SeededPseudoBackend::new(42);
        let center = Coordinates::new(40.7128, -74.0060);

        let response = generate(
            center,
            3000.0, // Must be >= FLOWER_POWER_MIN_RADIUS
            1000,
            50,
            false,
            GenerationMode::FlowerPower,
            "pseudo",
            &backend,
        )
        .unwrap();

        // Should have 7 circles
        assert_eq!(response.circles.len(), 7);
        assert_eq!(response.circles[0].id, "center");
        assert_eq!(response.circles[1].id, "petal_0");
        assert_eq!(response.circles[6].id, "petal_5");

        // Should have all anomaly types in winners
        assert!(response.winners.contains_key(&AnomalyType::BlindSpot));
        assert!(response.winners.contains_key(&AnomalyType::Attractor));
        assert!(response.winners.contains_key(&AnomalyType::Void));
        assert!(response.winners.contains_key(&AnomalyType::Power));
    }

    #[test]
    fn test_generate_with_points() {
        let backend = SeededPseudoBackend::new(42);
        let center = Coordinates::new(40.7128, -74.0060);

        let response = generate(
            center,
            1000.0,
            500,
            50,
            true, // Include points
            GenerationMode::Standard,
            "pseudo",
            &backend,
        )
        .unwrap();

        // Points should be included
        assert!(response.circles[0].points.is_some());
        assert_eq!(response.circles[0].points.as_ref().unwrap().len(), 500);
    }

    #[test]
    fn test_response_serialization() {
        let backend = SeededPseudoBackend::new(42);
        let center = Coordinates::new(40.7128, -74.0060);

        let response = generate(
            center,
            1000.0,
            100,
            50,
            false,
            GenerationMode::Standard,
            "pseudo",
            &backend,
        )
        .unwrap();

        // Should serialize to JSON without error
        let json = serde_json::to_string_pretty(&response).unwrap();
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"circles\""));
        assert!(json.contains("\"winners\""));

        // Should deserialize back
        let _: GenerationResponse = serde_json::from_str(&json).unwrap();
    }
}
