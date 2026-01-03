//! Default configuration values
//!
//! Named constants for all tunable parameters

/// Default QRNG backend
pub const DEFAULT_BACKEND: &str = "pseudo";

/// Default search radius in meters
pub const DEFAULT_RADIUS: f64 = 3000.0;

/// Default number of points for density analysis
pub const DEFAULT_POINTS: usize = 10_000;

/// Default grid resolution for density analysis
pub const DEFAULT_GRID_RESOLUTION: usize = 50;

/// Default output format
pub const DEFAULT_FORMAT: &str = "json";

/// Default anomaly type to display
pub const DEFAULT_TYPE: &str = "attractor";

/// Default generation mode
pub const DEFAULT_MODE: &str = "standard";

/// Default server host
pub const DEFAULT_HOST: &str = "127.0.0.1";

/// Default server port
pub const DEFAULT_PORT: u16 = 7878;

/// Default shutdown timeout in seconds (after last client disconnects)
pub const DEFAULT_SHUTDOWN_TIMEOUT_SECS: u64 = 30;

/// Default URL provider
pub const DEFAULT_URL_PROVIDER: &str = "google";

/// Config file name
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// Application directory name (for XDG paths)
pub const APP_DIR_NAME: &str = "q-explore";
