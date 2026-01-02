//! Point-in-circle generation
//!
//! Generates random points uniformly distributed within a circle.
//! Uses the sqrt() correction on radius to ensure uniform distribution.

use crate::coord::Coordinates;
use crate::error::Result;
use crate::qrng::QrngBackend;
use std::f64::consts::PI;

/// Meters per degree of latitude (approximately constant)
const METERS_PER_DEG_LAT: f64 = 111_320.0;

/// Generate a single random point uniformly distributed within a circle
///
/// # Arguments
/// * `center` - Center of the circle
/// * `radius_meters` - Radius in meters
/// * `rng` - Random number generator backend
///
/// # Returns
/// A random point within the circle
///
/// # Algorithm
/// Uses the standard uniform disk point picking algorithm:
/// - r = radius * sqrt(random())  -- sqrt corrects for area distribution
/// - theta = 2 * PI * random()
/// - Convert polar to lat/lng offset
///
/// Without sqrt(), points would cluster toward the center because the
/// probability density would be uniform in radius, but area increases
/// with r^2.
pub fn generate_point_in_circle(
    center: Coordinates,
    radius_meters: f64,
    rng: &dyn QrngBackend,
) -> Result<Coordinates> {
    let floats = rng.floats(2)?;
    let u1 = floats[0]; // For radius
    let u2 = floats[1]; // For angle

    // Uniform distribution in circle requires sqrt on radius
    let r = radius_meters * u1.sqrt();
    let theta = 2.0 * PI * u2;

    // Convert to lat/lng offset
    // Longitude degrees per meter varies with latitude
    let meters_per_deg_lng = METERS_PER_DEG_LAT * (center.lat * PI / 180.0).cos();

    let delta_lat = (r * theta.cos()) / METERS_PER_DEG_LAT;
    let delta_lng = (r * theta.sin()) / meters_per_deg_lng;

    Ok(Coordinates::new(center.lat + delta_lat, center.lng + delta_lng))
}

/// Generate many random points uniformly distributed within a circle
///
/// # Arguments
/// * `center` - Center of the circle
/// * `radius_meters` - Radius in meters
/// * `count` - Number of points to generate
/// * `rng` - Random number generator backend
///
/// # Returns
/// Vector of random points within the circle
pub fn generate_points_in_circle(
    center: Coordinates,
    radius_meters: f64,
    count: usize,
    rng: &dyn QrngBackend,
) -> Result<Vec<Coordinates>> {
    // Get all random floats at once for efficiency
    let floats = rng.floats(count * 2)?;
    let mut points = Vec::with_capacity(count);

    let meters_per_deg_lng = METERS_PER_DEG_LAT * (center.lat * PI / 180.0).cos();

    for i in 0..count {
        let u1 = floats[i * 2];     // For radius
        let u2 = floats[i * 2 + 1]; // For angle

        let r = radius_meters * u1.sqrt();
        let theta = 2.0 * PI * u2;

        let delta_lat = (r * theta.cos()) / METERS_PER_DEG_LAT;
        let delta_lng = (r * theta.sin()) / meters_per_deg_lng;

        points.push(Coordinates::new(center.lat + delta_lat, center.lng + delta_lng));
    }

    Ok(points)
}

/// Calculate the distance between two points in meters (Haversine formula)
///
/// # Arguments
/// * `p1` - First point
/// * `p2` - Second point
///
/// # Returns
/// Distance in meters
pub fn haversine_distance(p1: Coordinates, p2: Coordinates) -> f64 {
    const EARTH_RADIUS_METERS: f64 = 6_371_000.0;

    let lat1 = p1.lat * PI / 180.0;
    let lat2 = p2.lat * PI / 180.0;
    let delta_lat = (p2.lat - p1.lat) * PI / 180.0;
    let delta_lng = (p2.lng - p1.lng) * PI / 180.0;

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * (delta_lng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_METERS * c
}

/// Check if a point is within a circle
///
/// # Arguments
/// * `point` - Point to check
/// * `center` - Center of the circle
/// * `radius_meters` - Radius in meters
///
/// # Returns
/// true if the point is within the circle
pub fn is_in_circle(point: Coordinates, center: Coordinates, radius_meters: f64) -> bool {
    haversine_distance(point, center) <= radius_meters
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qrng::pseudo::SeededPseudoBackend;

    #[test]
    fn test_generate_point_in_circle() {
        let backend = SeededPseudoBackend::new(42);
        let center = Coordinates::new(40.7128, -74.0060); // NYC
        let radius = 1000.0; // 1 km

        let point = generate_point_in_circle(center, radius, &backend).unwrap();

        // Point should be within radius
        let distance = haversine_distance(center, point);
        assert!(
            distance <= radius * 1.01, // Allow 1% tolerance for floating point
            "Point at distance {} exceeds radius {}",
            distance,
            radius
        );
    }

    #[test]
    fn test_generate_many_points_in_circle() {
        let backend = SeededPseudoBackend::new(42);
        let center = Coordinates::new(40.7128, -74.0060);
        let radius = 1000.0;
        let count = 1000;

        let points = generate_points_in_circle(center, radius, count, &backend).unwrap();

        assert_eq!(points.len(), count);

        // All points should be within radius
        for point in &points {
            let distance = haversine_distance(center, *point);
            assert!(
                distance <= radius * 1.01,
                "Point at distance {} exceeds radius {}",
                distance,
                radius
            );
        }
    }

    #[test]
    fn test_uniform_distribution() {
        // Generate many points and check they're roughly uniformly distributed
        // by comparing the average distance to center vs expected for uniform disk
        let backend = SeededPseudoBackend::new(12345);
        let center = Coordinates::new(0.0, 0.0); // Equator to simplify
        let radius = 10000.0; // 10 km
        let count = 10000;

        let points = generate_points_in_circle(center, radius, count, &backend).unwrap();

        // For uniform distribution in a disk, expected average distance = 2R/3
        let expected_avg_distance = 2.0 * radius / 3.0;
        let actual_avg_distance: f64 = points
            .iter()
            .map(|p| haversine_distance(center, *p))
            .sum::<f64>()
            / count as f64;

        // Allow 5% tolerance
        let tolerance = expected_avg_distance * 0.05;
        assert!(
            (actual_avg_distance - expected_avg_distance).abs() < tolerance,
            "Average distance {} differs from expected {} by more than {}",
            actual_avg_distance,
            expected_avg_distance,
            tolerance
        );
    }

    #[test]
    fn test_haversine_distance() {
        // NYC to nearby point (about 1 degree = ~111km)
        let nyc = Coordinates::new(40.7128, -74.0060);
        let nearby = Coordinates::new(41.7128, -74.0060);

        let distance = haversine_distance(nyc, nearby);

        // Should be approximately 111 km
        assert!(
            (distance - 111_000.0).abs() < 1000.0,
            "Distance {} should be approximately 111000",
            distance
        );
    }

    #[test]
    fn test_is_in_circle() {
        let center = Coordinates::new(40.7128, -74.0060);
        let radius = 1000.0;

        // Center is definitely in circle
        assert!(is_in_circle(center, center, radius));

        // Point 500m away should be in circle
        let inside = Coordinates::new(40.7128 + 0.004, -74.0060); // ~440m north
        assert!(is_in_circle(inside, center, radius));

        // Point 2km away should be outside
        let outside = Coordinates::new(40.7128 + 0.02, -74.0060); // ~2.2km north
        assert!(!is_in_circle(outside, center, radius));
    }
}
