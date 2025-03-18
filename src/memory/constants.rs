/// Default warning threshold for memory pressure (percentage)
pub const DEFAULT_WARNING_THRESHOLD: f64 = 75.0;

/// Default critical threshold for memory pressure (percentage)
pub const DEFAULT_CRITICAL_THRESHOLD: f64 = 85.0;

/// Maximum history size for memory usage tracking
pub const MAX_HISTORY_SIZE: usize = 60;

/// Default interval for memory metric updates (milliseconds)
pub const DEFAULT_UPDATE_INTERVAL_MS: u64 = 1000;

/// Minimum memory metric update interval (milliseconds)
pub const MIN_UPDATE_INTERVAL_MS: u64 = 100;

/// Maximum memory metric update interval (milliseconds)
pub const MAX_UPDATE_INTERVAL_MS: u64 = 10_000;

/// Default capacity for memory pressure callback queue
pub const DEFAULT_CALLBACK_CAPACITY: usize = 16;
