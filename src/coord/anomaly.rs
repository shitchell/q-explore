//! Anomaly detection for coordinate generation
//!
//! Detects attractors (dense areas), voids (sparse areas), and power anomalies
//! (most statistically extreme in either direction).

use crate::coord::density::{
    find_densest_cell, find_emptiest_cell, find_most_anomalous_cell, DensityGrid,
};
pub use crate::coord::density::DEFAULT_GRID_RESOLUTION;
use crate::coord::point::generate_points_in_circle;
use crate::coord::{AnomalyType, Coordinates, Point};
use crate::error::Result;
use crate::qrng::QrngBackend;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Default number of points to generate for analysis
pub const DEFAULT_POINT_COUNT: usize = 10_000;

/// Results of anomaly detection for a single circle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleResults {
    /// Circle identifier (e.g., "center", "petal_0")
    pub id: String,

    /// Center of this circle
    pub center: Coordinates,

    /// Radius in meters
    pub radius: f64,

    /// Anomaly results by type
    pub anomalies: HashMap<AnomalyType, Point>,

    /// All generated points (only included if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<Vec<Coordinates>>,
}

/// Analyze a set of points and find all anomaly types
///
/// # Arguments
/// * `center` - Center of the search circle
/// * `radius` - Radius in meters
/// * `points` - Pre-generated points to analyze
/// * `grid_resolution` - Resolution of the density grid
///
/// # Returns
/// HashMap mapping each anomaly type to its result point
pub fn find_all_anomalies(
    center: Coordinates,
    radius: f64,
    points: &[Coordinates],
    grid_resolution: usize,
) -> HashMap<AnomalyType, Point> {
    let mut results = HashMap::new();

    // Blind spot is just the first point (or we can pick randomly)
    // In practice, for blind spot we'd only generate 1 point, but since
    // we're generating many for analysis, we just use the first one
    if !points.is_empty() {
        results.insert(AnomalyType::BlindSpot, Point::new(points[0]));
    }

    // Build density grid for attractor/void/power analysis
    let mut grid = DensityGrid::new(center, radius, grid_resolution);
    grid.add_points(points);

    // Find attractor (densest)
    if let Some(cell) = find_densest_cell(&grid) {
        results.insert(
            AnomalyType::Attractor,
            Point::with_z_score(cell.coords, cell.z_score),
        );
    }

    // Find void (emptiest)
    if let Some(cell) = find_emptiest_cell(&grid) {
        results.insert(
            AnomalyType::Void,
            Point::with_z_score(cell.coords, cell.z_score),
        );
    }

    // Find power (most anomalous)
    if let Some((cell, is_attractor)) = find_most_anomalous_cell(&grid) {
        results.insert(
            AnomalyType::Power,
            Point::power(cell.coords, cell.z_score, is_attractor),
        );
    }

    results
}

/// Generate points and analyze a single circle
///
/// # Arguments
/// * `id` - Identifier for this circle (e.g., "center")
/// * `center` - Center of the circle
/// * `radius` - Radius in meters
/// * `point_count` - Number of points to generate
/// * `grid_resolution` - Resolution of the density grid
/// * `include_points` - Whether to include all points in the result
/// * `rng` - Random number generator backend
///
/// # Returns
/// CircleResults with all anomaly types
pub fn analyze_circle(
    id: &str,
    center: Coordinates,
    radius: f64,
    point_count: usize,
    grid_resolution: usize,
    include_points: bool,
    rng: &dyn QrngBackend,
) -> Result<CircleResults> {
    // Generate random points
    let points = generate_points_in_circle(center, radius, point_count, rng)?;

    // Find all anomalies
    let anomalies = find_all_anomalies(center, radius, &points, grid_resolution);

    Ok(CircleResults {
        id: id.to_string(),
        center,
        radius,
        anomalies,
        points: if include_points { Some(points) } else { None },
    })
}

/// Find the winner for a specific anomaly type across multiple circles
///
/// # Arguments
/// * `circles` - Results from multiple circles
/// * `anomaly_type` - Which anomaly type to find the winner for
///
/// # Returns
/// Tuple of (winning circle id, winning point)
pub fn find_winner(
    circles: &[CircleResults],
    anomaly_type: AnomalyType,
) -> Option<(String, Point)> {
    let mut best: Option<(String, Point)> = None;

    for circle in circles {
        if let Some(point) = circle.anomalies.get(&anomaly_type) {
            let dominated = match anomaly_type {
                AnomalyType::BlindSpot => {
                    // For blind spot, just take the first one (or could be random)
                    best.is_some()
                }
                AnomalyType::Attractor => {
                    // Higher z-score is better (more dense)
                    best.as_ref().is_some_and(|(_, p)| {
                        p.z_score.unwrap_or(f64::NEG_INFINITY)
                            >= point.z_score.unwrap_or(f64::NEG_INFINITY)
                    })
                }
                AnomalyType::Void => {
                    // Lower z-score is better (more sparse)
                    best.as_ref().is_some_and(|(_, p)| {
                        p.z_score.unwrap_or(f64::INFINITY)
                            <= point.z_score.unwrap_or(f64::INFINITY)
                    })
                }
                AnomalyType::Power => {
                    // Higher absolute z-score is better
                    best.as_ref().is_some_and(|(_, p)| {
                        p.z_score.unwrap_or(0.0).abs() >= point.z_score.unwrap_or(0.0).abs()
                    })
                }
            };

            if !dominated {
                best = Some((circle.id.clone(), point.clone()));
            }
        }
    }

    best
}

/// Find winners for all anomaly types across multiple circles
pub fn find_all_winners(circles: &[CircleResults]) -> HashMap<AnomalyType, (String, Point)> {
    let mut winners = HashMap::new();

    for anomaly_type in [
        AnomalyType::BlindSpot,
        AnomalyType::Attractor,
        AnomalyType::Void,
        AnomalyType::Power,
    ] {
        if let Some(winner) = find_winner(circles, anomaly_type) {
            winners.insert(anomaly_type, winner);
        }
    }

    winners
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qrng::pseudo::SeededPseudoBackend;

    #[test]
    fn test_find_all_anomalies() {
        let backend = SeededPseudoBackend::new(42);
        let center = Coordinates::new(40.7128, -74.0060);
        let radius = 1000.0;
        let points = generate_points_in_circle(center, radius, 10000, &backend).unwrap();

        let anomalies = find_all_anomalies(center, radius, &points, 50);

        // Should have all four anomaly types
        assert!(anomalies.contains_key(&AnomalyType::BlindSpot));
        assert!(anomalies.contains_key(&AnomalyType::Attractor));
        assert!(anomalies.contains_key(&AnomalyType::Void));
        assert!(anomalies.contains_key(&AnomalyType::Power));

        // Attractor should have positive z-score
        let attractor = anomalies.get(&AnomalyType::Attractor).unwrap();
        assert!(attractor.z_score.unwrap() > 0.0);

        // Void should have negative z-score
        let void = anomalies.get(&AnomalyType::Void).unwrap();
        assert!(void.z_score.unwrap() < 0.0);

        // Power should have is_attractor set
        let power = anomalies.get(&AnomalyType::Power).unwrap();
        assert!(power.is_attractor.is_some());
    }

    #[test]
    fn test_analyze_circle() {
        let backend = SeededPseudoBackend::new(42);
        let center = Coordinates::new(40.7128, -74.0060);
        let radius = 1000.0;

        let result = analyze_circle("center", center, radius, 10000, 50, false, &backend).unwrap();

        assert_eq!(result.id, "center");
        assert_eq!(result.center.lat, center.lat);
        assert_eq!(result.center.lng, center.lng);
        assert!(result.points.is_none()); // Didn't request points

        // Should have all anomalies
        assert_eq!(result.anomalies.len(), 4);
    }

    #[test]
    fn test_analyze_circle_with_points() {
        let backend = SeededPseudoBackend::new(42);
        let center = Coordinates::new(40.7128, -74.0060);
        let radius = 1000.0;

        let result = analyze_circle("center", center, radius, 1000, 50, true, &backend).unwrap();

        // Should include points
        assert!(result.points.is_some());
        assert_eq!(result.points.as_ref().unwrap().len(), 1000);
    }

    #[test]
    fn test_find_winners() {
        let backend = SeededPseudoBackend::new(42);
        let center = Coordinates::new(40.7128, -74.0060);
        let radius = 1000.0;

        // Create two circles
        let circle1 = analyze_circle("center", center, radius, 5000, 50, false, &backend).unwrap();

        let backend2 = SeededPseudoBackend::new(123);
        let circle2 = analyze_circle("petal_0", center, radius, 5000, 50, false, &backend2).unwrap();

        let circles = vec![circle1, circle2];
        let winners = find_all_winners(&circles);

        // Should have winners for all types
        assert!(winners.contains_key(&AnomalyType::BlindSpot));
        assert!(winners.contains_key(&AnomalyType::Attractor));
        assert!(winners.contains_key(&AnomalyType::Void));
        assert!(winners.contains_key(&AnomalyType::Power));
    }
}
