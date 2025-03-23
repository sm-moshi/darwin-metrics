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
    pub current_speed: u32,
    pub target_speed: u32,
    pub min_speed: u32,
    pub max_speed: u32,
    pub index: usize,
    pub speed_rpm: u32,
    pub percentage: f64,
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

/// Thermal metrics for the system
#[derive(Debug, Clone)]
pub struct ThermalMetrics {
    pub io_kit: Arc<dyn IOKit>,
    pub fans: Vec<Fan>,
    pub details: ThermalDetails,
}

impl ThermalMetrics {
    pub fn new(io_kit: Arc<dyn IOKit>) -> Self {
        Self {
            io_kit,
            fans: Vec::new(),
            details: ThermalDetails::default(),
        }
    }
}

#[async_trait::async_trait]
impl ThermalMonitor for ThermalMetrics {
    async fn get_fans(&self) -> Result<Vec<Fan>> {
        Ok(self.fans.clone())
    }

    async fn cpu_temperature(&self) -> Result<Option<f64>> {
        Ok(self.details.cpu_temp)
    }

    async fn gpu_temperature(&self) -> Result<Option<f64>> {
        Ok(self.details.gpu_temp)
    }

    async fn memory_temperature(&self) -> Result<Option<f64>> {
        Ok(self.details.memory_temp)
    }

    async fn battery_temperature(&self) -> Result<Option<f64>> {
        Ok(self.details.battery_temp)
    }

    async fn ambient_temperature(&self) -> Result<Option<f64>> {
        Ok(self.details.ambient_temp)
    }

    async fn ssd_temperature(&self) -> Result<Option<f64>> {
        Ok(self.details.ssd_temp)
    }

    async fn is_throttling(&self) -> Result<bool> {
        Ok(self.details.is_throttling)
    }

    async fn get_thermal_metrics(&self) -> Result<ThermalMetrics> {
        Ok(self.clone())
    }
}

/// Configuration for thermal monitoring
#[derive(Debug, Clone)]
pub struct ThermalConfig {
    /// Temperature threshold for thermal throttling in degrees Celsius
    pub throttling_threshold: f64,
}

impl Default for ThermalConfig {
    fn default() -> Self {
        Self {
            throttling_threshold: CPU_CRITICAL_TEMPERATURE,
        }
    }
}
