/// Maximum number of CPU cores supported
pub const MAX_CORES: usize = 64;

/// Maximum CPU frequency in MHz
pub const MAX_FREQUENCY_MHZ: u32 = 10_000;

/// Default CPU name when model info is unavailable
pub const DEFAULT_CPU_NAME: &str = "Unknown CPU";

/// Temperature threshold for critical state (in Celsius)
pub const CRITICAL_TEMPERATURE_CELSIUS: f64 = 95.0;

/// Default update interval for CPU metrics in milliseconds
pub const DEFAULT_UPDATE_INTERVAL_MS: u64 = 1000;

/// Minimum time between CPU metric updates in milliseconds
pub const MIN_UPDATE_INTERVAL_MS: u64 = 100;

/// Maximum time between CPU metric updates in milliseconds
pub const MAX_UPDATE_INTERVAL_MS: u64 = 10_000;
