/// Default update interval for GPU metrics in milliseconds
pub const DEFAULT_UPDATE_INTERVAL_MS: u64 = 1000;

/// Temperature thresholds in Celsius
pub mod temperature {
    /// Default critical temperature threshold for Apple Silicon GPUs
    pub const APPLE_SILICON_CRITICAL_TEMP: f64 = 100.0;
    /// Default critical temperature threshold for Intel integrated GPUs
    pub const INTEL_CRITICAL_TEMP: f64 = 95.0;
    /// Default critical temperature threshold for discrete GPUs
    pub const DISCRETE_CRITICAL_TEMP: f64 = 90.0;
}

/// Memory thresholds
pub mod memory {
    /// Warning threshold for memory usage (percentage)
    pub const MEMORY_WARNING_THRESHOLD: f64 = 90.0;
    /// Critical threshold for memory usage (percentage)
    pub const MEMORY_CRITICAL_THRESHOLD: f64 = 95.0;
}

/// Utilization thresholds
pub mod utilization {
    /// High utilization threshold (percentage)
    pub const HIGH_UTILIZATION_THRESHOLD: f64 = 80.0;
    /// Sustained high utilization duration threshold (seconds)
    pub const SUSTAINED_HIGH_UTILIZATION_SECONDS: u64 = 300;
}

/// Metal API constants
pub mod metal {
    /// Default Metal device index
    pub const DEFAULT_DEVICE_INDEX: usize = 0;
    /// Maximum number of supported Metal devices
    pub const MAX_DEVICES: usize = 8;
}
