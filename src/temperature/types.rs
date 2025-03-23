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
    pub speed_rpm: u32,
    /// Minimum fan speed in RPM
    pub min_speed: u32,
    /// Maximum fan speed in RPM
    pub max_speed: u32,
    /// Target fan speed in RPM
    pub target_speed: u32,
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

/// Thermal metrics for a device
#[derive(Debug, Clone)]
pub struct ThermalMetrics {
    /// Fan speeds in RPM
    pub fan_speeds: Vec<u32>,
    /// Current thermal level
    pub thermal_level: ThermalLevel,
    /// Memory temperature in Celsius
    pub memory_temperature: Option<f64>,
    /// Whether the device is throttling
    pub is_throttling: bool,
    /// Fan information
    pub fans: Vec<Fan>,
    /// CPU temperature in Celsius
    pub cpu_temperature: Option<f64>,
    /// GPU temperature in Celsius
    pub gpu_temperature: Option<f64>,
    /// Battery temperature in Celsius
    pub battery_temperature: Option<f64>,
    /// SSD temperature in Celsius
    pub ssd_temperature: Option<f64>,
    /// Ambient temperature in Celsius
    pub ambient_temperature: Option<f64>,
}

impl Default for ThermalMetrics {
    fn default() -> Self {
        Self {
            fan_speeds: Vec::new(),
            thermal_level: ThermalLevel::Normal,
            memory_temperature: None,
            is_throttling: false,
            fans: Vec::new(),
            cpu_temperature: None,
            gpu_temperature: None,
            battery_temperature: None,
            ssd_temperature: None,
            ambient_temperature: None,
        }
    }
}

/// Thermal level of the device
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalLevel {
    /// Normal operating temperature
    Normal,
    /// Warning temperature level
    Warning,
    /// Critical temperature level
    Critical,
}
