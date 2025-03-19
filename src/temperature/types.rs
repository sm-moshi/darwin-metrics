use std::time::Instant;

/// Represents the location of a temperature sensor in the system
#[derive(Debug, Clone, PartialEq)]
pub enum SensorLocation {
    /// CPU temperature sensor
    Cpu,
    /// GPU temperature sensor
    Gpu,
    /// System memory temperature sensor
    Memory,
    /// Storage/SSD temperature sensor
    Storage,
    /// Battery temperature sensor
    Battery,
    /// Heatsink temperature sensor
    Heatsink,
    /// Ambient (inside case) temperature sensor
    Ambient,
    /// Other temperature sensor with a custom name
    Other(String),
}

/// Fan information including speed, min/max values, and utilization percentage
#[derive(Debug, Clone)]
pub struct Fan {
    /// Fan identifier (e.g., "CPU Fan", "System Fan")
    pub name: String,
    /// Current fan speed in RPM
    pub speed: f64,
    /// Minimum fan speed in RPM
    pub min_speed: f64,
    /// Maximum fan speed in RPM
    pub max_speed: f64,
    /// Current fan utilization as a percentage (0-100%)
    pub utilization: f64,
}

/// Configuration for temperature monitoring
#[derive(Debug, Clone)]
pub struct TemperatureConfig {
    /// How often to poll temperature sensors (in milliseconds)
    pub poll_interval_ms: u64,
    /// Throttling detection threshold in degrees Celsius
    pub throttling_threshold: f64,
    /// Whether to automatically refresh sensor data on read
    pub auto_refresh: bool,
    /// Update interval for sensor data
    pub update_interval: u64,
    /// Whether to enable GPU temperature monitoring
    pub enable_gpu: bool,
    /// Whether to enable CPU temperature monitoring
    pub enable_cpu: bool,
    /// Whether to enable battery temperature monitoring
    pub enable_battery: bool,
}

impl Default for TemperatureConfig {
    fn default() -> Self {
        use super::constants::*;
        Self {
            poll_interval_ms: DEFAULT_POLL_INTERVAL_MS,
            throttling_threshold: DEFAULT_THROTTLING_THRESHOLD,
            auto_refresh: true,
            update_interval: DEFAULT_POLL_INTERVAL_MS,
            enable_gpu: true,
            enable_cpu: true,
            enable_battery: true,
        }
    }
}

/// Comprehensive thermal metrics for the system
#[derive(Debug, Clone)]
pub struct ThermalMetrics {
    /// CPU temperature in degrees Celsius
    pub cpu_temperature: Option<f64>,
    /// GPU temperature in degrees Celsius
    pub gpu_temperature: Option<f64>,
    /// Memory temperature in degrees Celsius
    pub memory_temperature: Option<f64>,
    /// Ambient (inside case) temperature in degrees Celsius
    pub ambient_temperature: Option<f64>,
    /// Battery temperature in degrees Celsius
    pub battery_temperature: Option<f64>,
    /// SSD temperature in degrees Celsius
    pub ssd_temperature: Option<f64>,
    /// Whether the system is currently thermal throttling
    pub is_throttling: bool,
    /// Information about all fans in the system
    pub fans: Vec<Fan>,
}

impl Default for ThermalMetrics {
    fn default() -> Self {
        Self {
            cpu_temperature: None,
            gpu_temperature: None,
            memory_temperature: None,
            ambient_temperature: None,
            battery_temperature: None,
            ssd_temperature: None,
            is_throttling: false,
            fans: Vec::new(),
        }
    }
}
