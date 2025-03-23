/// Default polling interval in milliseconds
pub const DEFAULT_POLL_INTERVAL_MS: u64 = 1000;

/// Default throttling threshold in degrees Celsius
pub const DEFAULT_THROTTLING_THRESHOLD: f64 = 80.0;

/// Default update interval in milliseconds
pub const DEFAULT_UPDATE_INTERVAL_MS: u64 = 1000;

/// Critical temperature threshold in degrees Celsius
pub const CRITICAL_TEMPERATURE_THRESHOLD: f64 = 95.0;

/// Warning temperature threshold in degrees Celsius
pub const WARNING_TEMPERATURE_THRESHOLD: f64 = 85.0;

/// Default critical temperature for the ambient sensor (°C)
pub const AMBIENT_DEFAULT_CRITICAL_TEMP: f64 = AMBIENT_CRITICAL_TEMPERATURE;
/// Default critical temperature for the CPU (°C)
pub const CPU_DEFAULT_CRITICAL_TEMP: f64 = CPU_CRITICAL_TEMPERATURE;
/// Default critical temperature for the GPU (°C)
pub const GPU_DEFAULT_CRITICAL_TEMP: f64 = GPU_CRITICAL_TEMPERATURE;
/// Default critical temperature for SSDs and storage (°C)
pub const SSD_DEFAULT_CRITICAL_TEMP: f64 = SSD_CRITICAL_TEMPERATURE;
/// Default critical temperature for the battery (°C)
pub const BATTERY_DEFAULT_CRITICAL_TEMP: f64 = BATTERY_CRITICAL_TEMPERATURE;

/// Minimum fan speed percentage (0%)
pub const MIN_FAN_SPEED_PERCENTAGE: f64 = 0.0;
/// Maximum fan speed percentage (100%)
pub const MAX_FAN_SPEED_PERCENTAGE: f64 = 100.0;

/// Minimum valid temperature in degrees Celsius
pub const MIN_VALID_TEMPERATURE: f64 = -20.0;

/// Maximum valid temperature in degrees Celsius
pub const MAX_VALID_TEMPERATURE: f64 = 120.0;

/// Critical temperature threshold for SSD in degrees Celsius
pub const SSD_CRITICAL_TEMPERATURE: f64 = 70.0;

/// Critical temperature threshold for CPU in degrees Celsius
pub const CPU_CRITICAL_TEMPERATURE: f64 = 95.0;

/// Critical temperature threshold for GPU in degrees Celsius
pub const GPU_CRITICAL_TEMPERATURE: f64 = 90.0;

/// Critical temperature threshold for battery in degrees Celsius
pub const BATTERY_CRITICAL_TEMPERATURE: f64 = 65.0;

/// Critical temperature threshold for ambient sensor in degrees Celsius
pub const AMBIENT_CRITICAL_TEMPERATURE: f64 = 45.0;
