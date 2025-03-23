use crate::{
    error::Result,
    hardware::iokit::IOKit,
    traits::hardware::{FanMonitor, ThermalMonitor},
    temperature::constants::CPU_CRITICAL_TEMPERATURE,
};
use std::sync::Arc;

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

/// Type of temperature monitor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MonitorType {
    /// CPU temperature monitor
    Cpu,
    /// GPU temperature monitor
    Gpu,
    /// Ambient temperature monitor
    Ambient,
    /// Battery temperature monitor
    Battery,
    /// SSD temperature monitor
    Ssd,
    /// Fan monitor with index
    Fan(usize),
}

impl std::fmt::Display for MonitorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorType::Cpu => write!(f, "CPU"),
            MonitorType::Gpu => write!(f, "GPU"),
            MonitorType::Ambient => write!(f, "Ambient"),
            MonitorType::Battery => write!(f, "Battery"),
            MonitorType::Ssd => write!(f, "SSD"),
            MonitorType::Fan(index) => write!(f, "Fan {}", index),
        }
    }
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

impl Fan {
    pub fn new(
        current_speed: u32,
        target_speed: u32,
        min_speed: u32,
        max_speed: u32,
        index: usize,
        speed_rpm: u32,
        percentage: f64,
    ) -> Self {
        Self {
            current_speed,
            target_speed,
            min_speed,
            max_speed,
            index,
            speed_rpm,
            percentage,
        }
    }

    pub fn default() -> Self {
        Self {
            current_speed: 0,
            target_speed: 0,
            min_speed: 0,
            max_speed: 0,
            index: 0,
            speed_rpm: 0,
            percentage: 0.0,
        }
    }
}

#[async_trait::async_trait]
impl FanMonitor for Fan {
    async fn speed_rpm(&self) -> crate::error::Result<u32> {
        Ok(self.speed_rpm)
    }

    async fn min_speed(&self) -> crate::error::Result<u32> {
        Ok(self.min_speed)
    }

    async fn max_speed(&self) -> crate::error::Result<u32> {
        Ok(self.max_speed)
    }

    async fn percentage(&self) -> crate::error::Result<f64> {
        Ok(self.percentage)
    }

    async fn fan_name(&self) -> crate::error::Result<String> {
        Ok(format!("Fan {}", self.index))
    }
}

/// Thermal details for the system
#[derive(Debug, Clone)]
pub struct ThermalDetails {
    pub cpu_temp: Option<f64>,
    pub gpu_temp: Option<f64>,
    pub memory_temp: Option<f64>,
    pub battery_temp: Option<f64>,
    pub ambient_temp: Option<f64>,
    pub ssd_temp: Option<f64>,
    pub is_throttling: bool,
}

impl Default for ThermalDetails {
    fn default() -> Self {
        Self {
            cpu_temp: None,
            gpu_temp: None,
            memory_temp: None,
            battery_temp: None,
            ambient_temp: None,
            ssd_temp: None,
            is_throttling: false,
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

impl ThermalMetrics {
    pub fn new(io_kit: Arc<dyn IOKit>) -> Self {
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
