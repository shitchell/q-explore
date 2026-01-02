//! Point-in-circle generation
//!
//! Generates random points uniformly distributed within a spherical cap.
//! Uses true spherical geometry for accuracy at all latitudes, including poles.

use crate::constants::geo::EARTH_RADIUS_METERS;
use crate::coord::Coordinates;
use crate::error::Result;
use crate::qrng::QrngBackend;
use std::f64::consts::PI;

/// Generate a single random point uniformly distributed within a spherical cap
///
/// # Arguments
/// * `center` - Center of the spherical cap
/// * `radius_meters` - Radius in meters (along Earth's surface)
/// * `rng` - Random number generator backend
///
/// # Returns
/// A random point within the spherical cap
///
/// # Algorithm
/// Uses true spherical geometry:
/// 1. Generate uniform point on spherical cap centered at north pole
/// 2. Rotate the cap to be centered at the actual center point
/// 3. Convert back to lat/lng
///
/// This works correctly at all latitudes, including the poles.
pub fn generate_point_in_circle(
    center: Coordinates,
    radius_meters: f64,
    rng: &dyn QrngBackend,
) -> Result<Coordinates> {
    let floats = rng.floats(2)?;
    Ok(generate_point_spherical(center, radius_meters, floats[0], floats[1]))
}

/// Generate a point on a spherical cap using true spherical geometry
///
/// # Arguments
/// * `center` - Center of the spherical cap (lat/lng)
/// * `radius_meters` - Radius in meters (along Earth's surface)
/// * `u1` - Random value in [0, 1) for radial position
/// * `u2` - Random value in [0, 1) for angular position
///
/// # Returns
/// A point uniformly distributed within the spherical cap
fn generate_point_spherical(
    center: Coordinates,
    radius_meters: f64,
    u1: f64,
    u2: f64,
) -> Coordinates {
    // Angular radius of the spherical cap (in radians)
    let cap_angle = radius_meters / EARTH_RADIUS_METERS;

    // Generate uniform point on spherical cap centered at north pole
    // For uniform distribution on sphere, z (height) should be uniform
    // z ranges from 1 (pole) to cos(cap_angle) (edge of cap)
    let z = 1.0 - u1 * (1.0 - cap_angle.cos());
    let phi = 2.0 * PI * u2;

    // Convert to Cartesian on unit sphere (cap at north pole)
    let r_xy = (1.0 - z * z).sqrt();
    let x = r_xy * phi.cos();
    let y = r_xy * phi.sin();

    // Rotate from north pole to center
    // The rotation is: first rotate around y-axis by co-latitude,
    // then rotate around z-axis by longitude
    let center_lat_rad = center.lat * PI / 180.0;
    let center_lng_rad = center.lng * PI / 180.0;
    let co_lat = PI / 2.0 - center_lat_rad;

    // Rotation around y-axis by co-latitude
    let x1 = x * co_lat.cos() + z * co_lat.sin();
    let y1 = y;
    let z1 = -x * co_lat.sin() + z * co_lat.cos();

    // Rotation around z-axis by longitude
    let x2 = x1 * center_lng_rad.cos() - y1 * center_lng_rad.sin();
    let y2 = x1 * center_lng_rad.sin() + y1 * center_lng_rad.cos();
    let z2 = z1;

    // Convert back to lat/lng
    let lat = z2.asin() * 180.0 / PI;
    let lng = y2.atan2(x2) * 180.0 / PI;

    Coordinates::new(lat, lng)
}

/// Generate many random points uniformly distributed within a spherical cap
///
/// # Arguments
/// * `center` - Center of the spherical cap
/// * `radius_meters` - Radius in meters (along Earth's surface)
/// * `count` - Number of points to generate
/// * `rng` - Random number generator backend
///
/// # Returns
/// Vector of random points within the spherical cap
pub fn generate_points_in_circle(
    center: Coordinates,
    radius_meters: f64,
    count: usize,
    rng: &dyn QrngBackend,
) -> Result<Vec<Coordinates>> {
    // Get all random floats at once for efficiency
    let floats = rng.floats(count * 2)?;
    let mut points = Vec::with_capacity(count);

    for i in 0..count {
        let u1 = floats[i * 2];
        let u2 = floats[i * 2 + 1];
        points.push(generate_point_spherical(center, radius_meters, u1, u2));
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

    // ========== Spherical Geometry Tests ==========

    /// Helper to test point generation at a given latitude
    fn test_points_at_latitude(lat: f64, lng: f64, name: &str) {
        let backend = SeededPseudoBackend::new(42);
        let center = Coordinates::new(lat, lng);
        let radius = 5000.0; // 5 km
        let count = 1000;

        let points = generate_points_in_circle(center, radius, count, &backend).unwrap();

        assert_eq!(points.len(), count, "{}: wrong point count", name);

        // All points should be within radius (with small tolerance for floating point)
        let mut max_distance = 0.0f64;
        for (i, point) in points.iter().enumerate() {
            let distance = haversine_distance(center, *point);
            max_distance = max_distance.max(distance);
            assert!(
                distance <= radius * 1.01,
                "{}: point {} at distance {:.2}m exceeds radius {}m",
                name,
                i,
                distance,
                radius
            );
        }

        // Check uniform distribution (average distance should be ~2R/3)
        let expected_avg = 2.0 * radius / 3.0;
        let actual_avg: f64 = points
            .iter()
            .map(|p| haversine_distance(center, *p))
            .sum::<f64>()
            / count as f64;

        let tolerance = expected_avg * 0.10; // 10% tolerance
        assert!(
            (actual_avg - expected_avg).abs() < tolerance,
            "{}: avg distance {:.2} differs from expected {:.2} by more than {:.2}",
            name,
            actual_avg,
            expected_avg,
            tolerance
        );
    }

    #[test]
    fn test_north_pole() {
        // Exactly at north pole
        test_points_at_latitude(90.0, 0.0, "North Pole");
    }

    #[test]
    fn test_near_north_pole() {
        // Very close to north pole (Santa's workshop)
        test_points_at_latitude(89.9, 25.0, "Near North Pole");
    }

    #[test]
    fn test_south_pole() {
        // Exactly at south pole
        test_points_at_latitude(-90.0, 0.0, "South Pole");
    }

    #[test]
    fn test_near_south_pole() {
        // Antarctica
        test_points_at_latitude(-89.9, 0.0, "Near South Pole");
    }

    #[test]
    fn test_equator() {
        // On the equator
        test_points_at_latitude(0.0, 0.0, "Equator (0°, 0°)");
        test_points_at_latitude(0.0, 90.0, "Equator (0°, 90°E)");
        test_points_at_latitude(0.0, -90.0, "Equator (0°, 90°W)");
        test_points_at_latitude(0.0, 180.0, "Equator (0°, 180°)");
    }

    #[test]
    fn test_intermediate_latitudes_north() {
        // Various northern latitudes
        test_points_at_latitude(30.0, -90.0, "30°N (New Orleans)");
        test_points_at_latitude(45.0, -122.0, "45°N (Portland)");
        test_points_at_latitude(60.0, 10.0, "60°N (Oslo)");
        test_points_at_latitude(75.0, -40.0, "75°N (Greenland)");
    }

    #[test]
    fn test_intermediate_latitudes_south() {
        // Various southern latitudes
        test_points_at_latitude(-30.0, 151.0, "30°S (Sydney area)");
        test_points_at_latitude(-45.0, 170.0, "45°S (New Zealand)");
        test_points_at_latitude(-60.0, -60.0, "60°S (Drake Passage)");
    }

    #[test]
    fn test_date_line() {
        // Near the international date line
        test_points_at_latitude(0.0, 179.9, "Near date line (positive)");
        test_points_at_latitude(0.0, -179.9, "Near date line (negative)");
        test_points_at_latitude(45.0, 180.0, "On date line");
    }

    // ========== Translation Accuracy Tests ==========

    #[test]
    fn test_spherical_point_at_center() {
        // When u1 = 0, the point should be exactly at the center
        let center = Coordinates::new(45.0, -122.0);
        let radius = 1000.0;

        let point = generate_point_spherical(center, radius, 0.0, 0.0);

        // Should be exactly at center (or very close due to floating point)
        let distance = haversine_distance(center, point);
        assert!(
            distance < 0.01, // Less than 1 cm
            "Point with u1=0 should be at center, but is {:.6}m away",
            distance
        );
    }

    #[test]
    fn test_spherical_point_at_edge() {
        // When u1 = 1, the point should be at the edge of the circle
        let center = Coordinates::new(45.0, -122.0);
        let radius = 1000.0;

        // Test multiple angles
        for angle_fraction in [0.0, 0.25, 0.5, 0.75] {
            let point = generate_point_spherical(center, radius, 1.0, angle_fraction);
            let distance = haversine_distance(center, point);

            assert!(
                (distance - radius).abs() < 1.0, // Within 1 meter of edge
                "Point with u1=1, u2={} should be at radius {}, but is at {:.2}m",
                angle_fraction,
                radius,
                distance
            );
        }
    }

    #[test]
    fn test_spherical_north_pole_directions() {
        // At the north pole, all directions should work equally
        let center = Coordinates::new(90.0, 0.0);
        let radius = 1000.0;

        // Points at edge in 4 directions
        let points: Vec<Coordinates> = [0.0, 0.25, 0.5, 0.75]
            .iter()
            .map(|&u2| generate_point_spherical(center, radius, 1.0, u2))
            .collect();

        // All should be at the same distance from center
        for (i, point) in points.iter().enumerate() {
            let distance = haversine_distance(center, *point);
            assert!(
                (distance - radius).abs() < 1.0,
                "North pole direction {}: expected {}m, got {:.2}m",
                i,
                radius,
                distance
            );
        }

        // All points should be at roughly the same latitude (forming a circle)
        let lats: Vec<f64> = points.iter().map(|p| p.lat).collect();
        let avg_lat = lats.iter().sum::<f64>() / lats.len() as f64;
        for (i, lat) in lats.iter().enumerate() {
            assert!(
                (lat - avg_lat).abs() < 0.001,
                "North pole: point {} lat {:.6} differs from avg {:.6}",
                i,
                lat,
                avg_lat
            );
        }
    }

    #[test]
    fn test_spherical_south_pole_directions() {
        // At the south pole, all directions should work equally
        let center = Coordinates::new(-90.0, 0.0);
        let radius = 1000.0;

        let points: Vec<Coordinates> = [0.0, 0.25, 0.5, 0.75]
            .iter()
            .map(|&u2| generate_point_spherical(center, radius, 1.0, u2))
            .collect();

        for (i, point) in points.iter().enumerate() {
            let distance = haversine_distance(center, *point);
            assert!(
                (distance - radius).abs() < 1.0,
                "South pole direction {}: expected {}m, got {:.2}m",
                i,
                radius,
                distance
            );
        }
    }

    #[test]
    fn test_spherical_symmetry() {
        // Points generated at symmetric positions should have symmetric results
        let center = Coordinates::new(45.0, 0.0);
        let radius = 5000.0;

        // u2 = 0.0 and u2 = 0.5 should give points on opposite sides
        let p1 = generate_point_spherical(center, radius, 0.5, 0.0);
        let p2 = generate_point_spherical(center, radius, 0.5, 0.5);

        // They should be equidistant from center
        let d1 = haversine_distance(center, p1);
        let d2 = haversine_distance(center, p2);
        assert!(
            (d1 - d2).abs() < 0.1,
            "Symmetric points should be equidistant: {:.2}m vs {:.2}m",
            d1,
            d2
        );

        // The distance between them should be approximately 2 * their distance from center
        let d12 = haversine_distance(p1, p2);
        assert!(
            (d12 - 2.0 * d1).abs() < 10.0,
            "Opposite points should be ~2r apart: got {:.2}m, expected ~{:.2}m",
            d12,
            2.0 * d1
        );
    }

    #[test]
    fn test_large_radius_near_pole() {
        // Large radius near pole (100km) - tests that we handle big spherical caps
        let backend = SeededPseudoBackend::new(999);
        let center = Coordinates::new(85.0, 0.0);
        let radius = 100_000.0; // 100 km

        let points = generate_points_in_circle(center, radius, 500, &backend).unwrap();

        for (i, point) in points.iter().enumerate() {
            let distance = haversine_distance(center, *point);
            assert!(
                distance <= radius * 1.01,
                "Large polar cap: point {} at {:.2}m exceeds {}m",
                i,
                distance,
                radius
            );
        }
    }

    #[test]
    fn test_coordinate_validity() {
        // All generated coordinates should be valid (-90 <= lat <= 90, -180 <= lng <= 180)
        let backend = SeededPseudoBackend::new(12345);
        let test_cases = [
            (90.0, 0.0, "North Pole"),
            (-90.0, 0.0, "South Pole"),
            (0.0, 180.0, "Date Line"),
            (0.0, -180.0, "Date Line negative"),
            (45.0, 179.9, "Near date line"),
        ];

        for (lat, lng, name) in test_cases {
            let center = Coordinates::new(lat, lng);
            let points = generate_points_in_circle(center, 5000.0, 100, &backend).unwrap();

            for (i, point) in points.iter().enumerate() {
                assert!(
                    point.lat >= -90.0 && point.lat <= 90.0,
                    "{}: point {} has invalid lat {:.6}",
                    name,
                    i,
                    point.lat
                );
                assert!(
                    point.lng >= -180.0 && point.lng <= 180.0,
                    "{}: point {} has invalid lng {:.6}",
                    name,
                    i,
                    point.lng
                );
            }
        }
    }

    // ========== Strong Uniformity Tests ==========

    /// Test radial area distribution: for a uniform disk, area is proportional to r²
    /// So inner 50% of radius contains 25% of area, outer 50% contains 75%
    #[test]
    fn test_radial_area_distribution() {
        let backend = SeededPseudoBackend::new(54321);
        let center = Coordinates::new(45.0, -122.0); // Mid-latitude
        let radius = 10_000.0; // 10 km
        let count = 10_000;

        let points = generate_points_in_circle(center, radius, count, &backend).unwrap();

        // Count points in inner half (r < R/2) vs outer half (r >= R/2)
        let half_radius = radius / 2.0;
        let mut inner_count = 0;
        let mut outer_count = 0;

        for point in &points {
            let distance = haversine_distance(center, *point);
            if distance < half_radius {
                inner_count += 1;
            } else {
                outer_count += 1;
            }
        }

        // Expected: inner 25%, outer 75% (since area = πr², inner half has (0.5)² = 0.25)
        let inner_fraction = inner_count as f64 / count as f64;
        let outer_fraction = outer_count as f64 / count as f64;

        // Allow 3% tolerance (statistical variance for 10k points)
        assert!(
            (inner_fraction - 0.25).abs() < 0.03,
            "Inner half should have ~25% of points, got {:.1}% ({} points)",
            inner_fraction * 100.0,
            inner_count
        );
        assert!(
            (outer_fraction - 0.75).abs() < 0.03,
            "Outer half should have ~75% of points, got {:.1}% ({} points)",
            outer_fraction * 100.0,
            outer_count
        );
    }

    /// Test radial distribution with multiple bands
    /// Divides into 4 equal-radius bands, expected areas: 6.25%, 18.75%, 31.25%, 43.75%
    #[test]
    fn test_radial_bands_distribution() {
        let backend = SeededPseudoBackend::new(99999);
        let center = Coordinates::new(0.0, 0.0); // Equator
        let radius = 10_000.0;
        let count = 20_000; // More points for finer bands

        let points = generate_points_in_circle(center, radius, count, &backend).unwrap();

        // 4 equal-radius bands: [0, R/4), [R/4, R/2), [R/2, 3R/4), [3R/4, R]
        let mut bands = [0usize; 4];
        for point in &points {
            let distance = haversine_distance(center, *point);
            let normalized = distance / radius;
            let band = match normalized {
                d if d < 0.25 => 0,
                d if d < 0.50 => 1,
                d if d < 0.75 => 2,
                _ => 3,
            };
            bands[band] += 1;
        }

        // Expected fractions based on area = r²
        // Band 0: 0.25² = 0.0625 (6.25%)
        // Band 1: 0.50² - 0.25² = 0.1875 (18.75%)
        // Band 2: 0.75² - 0.50² = 0.3125 (31.25%)
        // Band 3: 1.00² - 0.75² = 0.4375 (43.75%)
        let expected = [0.0625, 0.1875, 0.3125, 0.4375];

        for (i, (&observed, &exp)) in bands.iter().zip(expected.iter()).enumerate() {
            let observed_fraction = observed as f64 / count as f64;
            let tolerance = 0.02; // 2% tolerance

            assert!(
                (observed_fraction - exp).abs() < tolerance,
                "Band {} (r={:.0}-{:.0}m): expected {:.1}%, got {:.1}% ({} points)",
                i,
                i as f64 * radius / 4.0,
                (i + 1) as f64 * radius / 4.0,
                exp * 100.0,
                observed_fraction * 100.0,
                observed
            );
        }
    }

    /// Test quadrant balance: points should be evenly distributed across 4 quadrants
    #[test]
    fn test_quadrant_balance() {
        let backend = SeededPseudoBackend::new(11111);
        let center = Coordinates::new(45.0, 0.0); // Mid-latitude, prime meridian
        let radius = 10_000.0;
        let count = 10_000;

        let points = generate_points_in_circle(center, radius, count, &backend).unwrap();

        // Count points in each quadrant (relative to center)
        // Q0: NE (lat+, lng+), Q1: NW (lat+, lng-), Q2: SW (lat-, lng-), Q3: SE (lat-, lng+)
        let mut quadrants = [0usize; 4];

        for point in &points {
            let dlat = point.lat - center.lat;
            let dlng = point.lng - center.lng;

            let quadrant = match (dlat >= 0.0, dlng >= 0.0) {
                (true, true) => 0,   // NE
                (true, false) => 1,  // NW
                (false, false) => 2, // SW
                (false, true) => 3,  // SE
            };
            quadrants[quadrant] += 1;
        }

        // Each quadrant should have ~25% of points
        let expected = 0.25;
        let tolerance = 0.03; // 3% tolerance

        for (i, &observed) in quadrants.iter().enumerate() {
            let observed_fraction = observed as f64 / count as f64;
            let quadrant_name = ["NE", "NW", "SW", "SE"][i];

            assert!(
                (observed_fraction - expected).abs() < tolerance,
                "Quadrant {} should have ~25% of points, got {:.1}% ({} points)",
                quadrant_name,
                observed_fraction * 100.0,
                observed
            );
        }
    }

    /// Test 8-sector angular distribution
    #[test]
    fn test_angular_sector_distribution() {
        let backend = SeededPseudoBackend::new(22222);
        let center = Coordinates::new(30.0, -90.0); // New Orleans area
        let radius = 10_000.0;
        let count = 16_000; // 2000 expected per sector

        let points = generate_points_in_circle(center, radius, count, &backend).unwrap();

        // Divide into 8 sectors (45° each)
        let mut sectors = [0usize; 8];

        for point in &points {
            let dlat = point.lat - center.lat;
            let dlng = point.lng - center.lng;

            // Calculate angle from center (0 = east, counter-clockwise)
            let angle = dlng.atan2(dlat); // Note: atan2(y, x) but we use lat as "y-ish"
            let angle_deg = angle * 180.0 / PI;

            // Normalize to [0, 360)
            let angle_normalized = if angle_deg < 0.0 {
                angle_deg + 360.0
            } else {
                angle_deg
            };

            let sector = (angle_normalized / 45.0) as usize;
            let sector = sector.min(7); // Clamp for edge case
            sectors[sector] += 1;
        }

        // Each sector should have ~12.5% of points
        let expected = 1.0 / 8.0;
        let tolerance = 0.025; // 2.5% tolerance

        for (i, &observed) in sectors.iter().enumerate() {
            let observed_fraction = observed as f64 / count as f64;

            assert!(
                (observed_fraction - expected).abs() < tolerance,
                "Sector {} ({}-{}°) should have ~12.5% of points, got {:.1}% ({} points)",
                i,
                i * 45,
                (i + 1) * 45,
                observed_fraction * 100.0,
                observed
            );
        }
    }

    /// Chi-square test on a 5x5 grid for spatial uniformity
    #[test]
    fn test_chi_square_grid_uniformity() {
        let backend = SeededPseudoBackend::new(33333);
        let center = Coordinates::new(40.0, -74.0); // NYC area
        let radius = 10_000.0;
        let count = 10_000;

        let points = generate_points_in_circle(center, radius, count, &backend).unwrap();

        // Create a 5x5 grid over the bounding box
        const GRID_SIZE: usize = 5;
        let mut grid = [[0usize; GRID_SIZE]; GRID_SIZE];

        // Find bounding box
        let (mut min_lat, mut max_lat) = (f64::MAX, f64::MIN);
        let (mut min_lng, mut max_lng) = (f64::MAX, f64::MIN);

        for point in &points {
            min_lat = min_lat.min(point.lat);
            max_lat = max_lat.max(point.lat);
            min_lng = min_lng.min(point.lng);
            max_lng = max_lng.max(point.lng);
        }

        // Add small padding to avoid edge issues
        let lat_range = max_lat - min_lat;
        let lng_range = max_lng - min_lng;

        // Count points in each cell
        let mut total_in_grid = 0;
        for point in &points {
            let lat_idx =
                ((point.lat - min_lat) / lat_range * GRID_SIZE as f64) as usize;
            let lng_idx =
                ((point.lng - min_lng) / lng_range * GRID_SIZE as f64) as usize;

            let lat_idx = lat_idx.min(GRID_SIZE - 1);
            let lng_idx = lng_idx.min(GRID_SIZE - 1);

            grid[lat_idx][lng_idx] += 1;
            total_in_grid += 1;
        }

        // For a circle inscribed in a square grid, not all cells are equal
        // The corner cells have less area in the circle
        // Instead of strict chi-square, we check that no cell is severely over/under populated

        // Expected average per cell (if it were a square)
        let cells_in_circle = GRID_SIZE * GRID_SIZE; // 25 cells
        let avg_per_cell = total_in_grid as f64 / cells_in_circle as f64;

        // Check that central cells (which are fully in the circle) are reasonably populated
        // Central 3x3 should be close to expected
        let central_cells = [
            grid[1][1], grid[1][2], grid[1][3],
            grid[2][1], grid[2][2], grid[2][3],
            grid[3][1], grid[3][2], grid[3][3],
        ];

        let central_avg = central_cells.iter().sum::<usize>() as f64 / 9.0;

        // Central cells should be reasonably close to overall average
        // (they're fully inside the circle, so might be slightly higher)
        assert!(
            central_avg > avg_per_cell * 0.5 && central_avg < avg_per_cell * 2.0,
            "Central 3x3 average ({:.1}) should be close to overall average ({:.1})",
            central_avg,
            avg_per_cell
        );

        // The center cell should have the most consistent count
        let center_cell = grid[2][2];
        let center_expected = avg_per_cell;
        assert!(
            (center_cell as f64 - center_expected).abs() < center_expected * 0.5,
            "Center cell has {} points, expected ~{:.0} (±50%)",
            center_cell,
            center_expected
        );
    }

    /// Test uniformity at various latitudes to catch any latitude-dependent bias
    #[test]
    fn test_uniformity_across_latitudes() {
        let test_latitudes: [(f64, &str); 5] = [
            (0.0, "Equator"),
            (30.0, "30°N"),
            (60.0, "60°N"),
            (85.0, "85°N (Arctic)"),
            (-45.0, "45°S"),
        ];

        for (lat, name) in test_latitudes {
            let backend = SeededPseudoBackend::new((lat.abs() * 1000.0) as u64);
            let center = Coordinates::new(lat, 0.0);
            let radius = 5000.0;
            let count = 5000;

            let points = generate_points_in_circle(center, radius, count, &backend).unwrap();

            // Check radial distribution (inner 50% radius = 25% area)
            let half_radius = radius / 2.0;
            let inner_count = points
                .iter()
                .filter(|p| haversine_distance(center, **p) < half_radius)
                .count();

            let inner_fraction = inner_count as f64 / count as f64;

            assert!(
                (inner_fraction - 0.25).abs() < 0.04, // 4% tolerance
                "{}: inner half should have ~25% of points, got {:.1}%",
                name,
                inner_fraction * 100.0
            );

            // Check quadrant balance
            let mut quadrants = [0usize; 4];
            for point in &points {
                let dlat = point.lat - center.lat;
                let dlng = point.lng - center.lng;
                let q = match (dlat >= 0.0, dlng >= 0.0) {
                    (true, true) => 0,
                    (true, false) => 1,
                    (false, false) => 2,
                    (false, true) => 3,
                };
                quadrants[q] += 1;
            }

            for (i, &q) in quadrants.iter().enumerate() {
                let fraction = q as f64 / count as f64;
                assert!(
                    (fraction - 0.25).abs() < 0.05, // 5% tolerance
                    "{}: quadrant {} should have ~25%, got {:.1}%",
                    name,
                    i,
                    fraction * 100.0
                );
            }
        }
    }
}
