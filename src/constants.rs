//! Centralized constants for the q-explore crate
//!
//! This module consolidates constants that are used across multiple modules
//! to avoid duplication and ensure consistency.

/// Geographic constants
pub mod geo {
    /// Mean Earth radius in meters (WGS84 approximation)
    pub const EARTH_RADIUS_METERS: f64 = 6_371_000.0;

    /// Meters per degree of latitude (approximate, varies slightly with latitude)
    pub const METERS_PER_DEGREE_LAT: f64 = 111_320.0;
}

/// External API endpoints
pub mod api {
    /// OpenStreetMap Nominatim geocoding API
    pub const NOMINATIM_URL: &str = "https://nominatim.openstreetmap.org";

    /// IP geolocation API (free, no key required)
    pub const IP_API_URL: &str = "http://ip-api.com/json";

    /// ANU QRNG free tier (has expired SSL cert)
    pub const ANU_FREE_URL: &str = "https://qrng.anu.edu.au/API/jsonI.php";

    /// ANU QRNG paid tier (requires API key)
    pub const ANU_PAID_URL: &str = "https://api.quantumnumbers.anu.edu.au";
}

/// Cache settings
pub mod cache {
    /// IP location cache duration in seconds (1 hour)
    pub const IP_LOCATION_TTL_SECS: u64 = 3600;

    /// IP location cache file name
    pub const IP_LOCATION_CACHE_FILE: &str = "ip_location_cache.json";
}
