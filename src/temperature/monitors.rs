use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;

/// Temperature monitoring implementations
///
/// This module contains implementations for monitoring various hardware component
/// temperatures including CPU, GPU, battery, SSD, and ambient sensors, as well as
/// fan monitoring functionality.
use crate::core::metrics::Metric;
use crate::core::types::Temperature as TemperatureType;
use crate::error::Result;
use crate::hardware::iokit::IOKit;
use crate::temperature::constants::*;
use crate::traits::{FanMonitor as TraitsFanMonitor, HardwareMonitor, TemperatureMonitor};

// Temperature constants
const AMBIENT_DEFAULT_CRITICAL_TEMP: f64 = AMBIENT_CRITICAL_TEMPERATURE;
const CPU_DEFAULT_CRITICAL_TEMP: f64 = CPU_CRITICAL_TEMPERATURE;
const GPU_DEFAULT_CRITICAL_TEMP: f64 = GPU_CRITICAL_TEMPERATURE;
const SSD_DEFAULT_CRITICAL_TEMP: f64 = SSD_CRITICAL_TEMPERATURE;
const BATTERY_DEFAULT_CRITICAL_TEMP: f64 = BATTERY_CRITICAL_TEMPERATURE;

// Fan speed percentage range constants
const MIN_FAN_SPEED_PERCENTAGE: f64 = 0.0;
const MAX_FAN_SPEED_PERCENTAGE: f64 = 100.0;

// Fan Monitor
//
/// Monitor for fan speed and control
pub struct FanMonitor {
    /// IOKit service for accessing hardware information
    io_kit: Arc<Box<dyn IOKit>>,
    /// Fan index
    index: usize,
}

impl FanMonitor {
    /// Create a new fan monitor
    pub fn new(io_kit: Arc<Box<dyn IOKit>>, index: usize) -> Self {
        Self { io_kit, index }
    }
}

#[async_trait]
impl TraitsFanMonitor for FanMonitor {
    async fn speed_rpm(&self) -> Result<u32> {
        let fans = self.io_kit.get_all_fans()?;
        fans.get(self.index)
            .map(|fan| fan.speed_rpm)
            .ok_or_else(|| crate::error::Error::NotAvailable {
                resource: format!("Fan {}", self.index),
                reason: "Not found".to_string(),
            })
    }

    async fn min_speed(&self) -> Result<u32> {
        let fans = self.io_kit.get_all_fans()?;
        fans.get(self.index)
            .map(|fan| fan.min_speed)
            .ok_or_else(|| crate::error::Error::NotAvailable {
                resource: format!("Fan {}", self.index),
                reason: "Not found".to_string(),
            })
    }

    async fn max_speed(&self) -> Result<u32> {
        let fans = self.io_kit.get_all_fans()?;
        fans.get(self.index)
            .map(|fan| fan.max_speed)
            .ok_or_else(|| crate::error::Error::NotAvailable {
                resource: format!("Fan {}", self.index),
                reason: "Not found".to_string(),
            })
    }

    async fn percentage(&self) -> Result<f64> {
        let speed = self.speed_rpm().await? as f64;
        let min = self.min_speed().await? as f64;
        let max = self.max_speed().await? as f64;

        if max == min {
            return Ok(0.0);
        }

        let percentage = ((speed - min) / (max - min)) * 100.0;
        Ok(percentage.clamp(MIN_FAN_SPEED_PERCENTAGE, MAX_FAN_SPEED_PERCENTAGE))
    }

    async fn fan_name(&self) -> Result<String> {
        Ok(format!("Fan {}", self.index))
    }
}

// CPU Temperature Monitor
//
/// Monitor for CPU temperature
pub struct CpuTemperatureMonitor {
    /// IOKit service for accessing hardware information
    io_kit: Arc<Box<dyn IOKit>>,
}

impl CpuTemperatureMonitor {
    /// Create a new CPU temperature monitor
    pub fn new(io_kit: Arc<Box<dyn IOKit>>) -> Self {
        Self { io_kit }
    }

    /// Get the current CPU temperature in Celsius
    pub async fn temperature(&self) -> Result<f64> {
        let info = self.io_kit.get_thermal_info()?;
        Ok(info.cpu_temp)
    }
}

/// Trait for temperature monitoring specific to this module
#[async_trait]
pub trait TemperatureMonitorExt {
    /// Check if temperature is at a critical level
    async fn is_critical(&self) -> Result<bool>;

    /// Get the critical temperature threshold
    async fn critical_threshold(&self) -> Result<f64>;
}

#[async_trait]
impl TemperatureMonitor for CpuTemperatureMonitor {
    async fn temperature(&self) -> Result<f64> {
        let info = self.io_kit.get_thermal_info()?;
        Ok(info.cpu_temp)
    }
}

#[async_trait]
impl TemperatureMonitorExt for CpuTemperatureMonitor {
    async fn is_critical(&self) -> Result<bool> {
        let temp = self.temperature().await?;
        Ok(temp >= CPU_CRITICAL_TEMPERATURE)
    }

    async fn critical_threshold(&self) -> Result<f64> {
        Ok(CPU_CRITICAL_TEMPERATURE)
    }
}

#[async_trait]
impl HardwareMonitor for CpuTemperatureMonitor {
    type MetricType = TemperatureType;

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let temperature = self.temperature().await?;
        Ok(Metric {
            value: TemperatureType::new_celsius(temperature),
            timestamp: SystemTime::now(),
        })
    }

    async fn name(&self) -> Result<String> {
        Ok("CPU Temperature Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("CPU".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("cpu0".to_string())
    }
}

// GPU Temperature Monitor
//
/// Monitor for GPU temperature
pub struct GpuTemperatureMonitor {
    /// IOKit service for accessing hardware information
    io_kit: Arc<Box<dyn IOKit>>,
}

impl GpuTemperatureMonitor {
    /// Create a new GPU temperature monitor
    pub fn new(io_kit: Arc<Box<dyn IOKit>>) -> Self {
        Self { io_kit }
    }

    /// Get the current GPU temperature in Celsius
    pub async fn temperature(&self) -> Result<f64> {
        let info = self.io_kit.get_thermal_info()?;
        info.gpu_temp.ok_or_else(|| crate::error::Error::NotAvailable {
            resource: "GPU Temperature".to_string(),
            reason: "Not supported on this device".to_string(),
        })
    }

    /// Check if GPU temperature is at a critical level
    pub async fn is_critical(&self) -> Result<bool> {
        match self.temperature().await {
            Ok(temp) => Ok(temp >= GPU_CRITICAL_TEMPERATURE),
            Err(_) => Ok(false), // If we can't read the temperature, assume it's not critical
        }
    }

    /// Get the critical temperature threshold for GPU
    pub async fn critical_threshold(&self) -> Result<f64> {
        Ok(GPU_CRITICAL_TEMPERATURE)
    }
}

#[async_trait]
impl HardwareMonitor for GpuTemperatureMonitor {
    type MetricType = TemperatureType;

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let temperature = self.temperature().await?;
        Ok(Metric {
            value: TemperatureType::new_celsius(temperature),
            timestamp: SystemTime::now(),
        })
    }

    async fn name(&self) -> Result<String> {
        Ok("GPU Temperature Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("GPU".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("gpu0".to_string())
    }
}

#[async_trait]
impl TemperatureMonitor for GpuTemperatureMonitor {
    async fn temperature(&self) -> Result<f64> {
        match self.io_kit.get_thermal_info() {
            Ok(info) => info.gpu_temp.ok_or_else(|| crate::error::Error::NotAvailable {
                resource: "GPU temperature".to_string(),
                reason: "Hardware info not available".to_string(),
            }),
            Err(_) => Err(crate::error::Error::NotAvailable {
                resource: "GPU temperature".to_string(),
                reason: "Hardware info not available".to_string(),
            }),
        }
    }
}

#[async_trait]
impl TemperatureMonitorExt for GpuTemperatureMonitor {
    async fn is_critical(&self) -> Result<bool> {
        match self.temperature().await {
            Ok(temp) => Ok(temp >= GPU_CRITICAL_TEMPERATURE),
            Err(_) => Ok(false),
        }
    }

    async fn critical_threshold(&self) -> Result<f64> {
        Ok(GPU_CRITICAL_TEMPERATURE)
    }
}

// Ambient Temperature Monitor
//
/// Monitor for ambient temperature
pub struct AmbientTemperatureMonitor {
    /// IOKit service for accessing hardware information
    io_kit: Arc<Box<dyn IOKit>>,
}

impl AmbientTemperatureMonitor {
    /// Create a new ambient temperature monitor
    pub fn new(io_kit: Arc<Box<dyn IOKit>>) -> Self {
        Self { io_kit }
    }

    /// Get the current ambient temperature in Celsius
    pub async fn temperature(&self) -> Result<f64> {
        let info = self.io_kit.get_thermal_info()?;
        info.ambient_temp.ok_or_else(|| crate::error::Error::NotAvailable {
            resource: "Ambient temperature".to_string(),
            reason: "Hardware info not available".to_string(),
        })
    }

    /// Check if ambient temperature is at a critical level
    pub async fn is_critical(&self) -> Result<bool> {
        match self.temperature().await {
            Ok(temp) => Ok(temp >= AMBIENT_CRITICAL_TEMPERATURE),
            Err(_) => Ok(false),
        }
    }

    /// Get the critical temperature threshold for ambient temperature
    pub async fn critical_threshold(&self) -> Result<f64> {
        Ok(AMBIENT_CRITICAL_TEMPERATURE)
    }

    /// Get the name of this temperature monitor
    pub async fn name(&self) -> Result<String> {
        Ok("Ambient Temperature Monitor".to_string())
    }
}

#[async_trait]
impl HardwareMonitor for AmbientTemperatureMonitor {
    type MetricType = TemperatureType;

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let temperature = self.temperature().await?;
        Ok(Metric {
            value: TemperatureType::new_celsius(temperature),
            timestamp: SystemTime::now(),
        })
    }

    async fn name(&self) -> Result<String> {
        self.name().await
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Ambient".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("ambient0".to_string())
    }
}

#[async_trait]
impl TemperatureMonitor for AmbientTemperatureMonitor {
    async fn temperature(&self) -> Result<f64> {
        let info = self.io_kit.get_thermal_info()?;
        info.ambient_temp.ok_or_else(|| crate::error::Error::NotAvailable {
            resource: "Ambient temperature".to_string(),
            reason: "Hardware info not available".to_string(),
        })
    }
}

#[async_trait]
impl TemperatureMonitorExt for AmbientTemperatureMonitor {
    async fn is_critical(&self) -> Result<bool> {
        match self.temperature().await {
            Ok(temp) => Ok(temp >= AMBIENT_CRITICAL_TEMPERATURE),
            Err(_) => Ok(false),
        }
    }

    async fn critical_threshold(&self) -> Result<f64> {
        Ok(AMBIENT_CRITICAL_TEMPERATURE)
    }
}

// Battery Temperature Monitor
//
/// Monitor for battery temperature
pub struct BatteryTemperatureMonitor {
    /// IOKit service for accessing hardware information
    io_kit: Arc<Box<dyn IOKit>>,
}

impl BatteryTemperatureMonitor {
    /// Create a new battery temperature monitor
    pub fn new(io_kit: Arc<Box<dyn IOKit>>) -> Self {
        Self { io_kit }
    }

    /// Get the current battery temperature in Celsius
    pub async fn temperature(&self) -> Result<f64> {
        let info = self.io_kit.get_thermal_info()?;
        info.battery_temp.ok_or_else(|| crate::error::Error::NotAvailable {
            resource: "Battery temperature".to_string(),
            reason: "Hardware info not available".to_string(),
        })
    }

    /// Check if battery temperature is at a critical level
    pub async fn is_critical(&self) -> Result<bool> {
        match self.temperature().await {
            Ok(temp) => Ok(temp >= BATTERY_CRITICAL_TEMPERATURE),
            Err(_) => Ok(false),
        }
    }

    /// Get the critical temperature threshold for battery
    pub async fn critical_threshold(&self) -> Result<f64> {
        Ok(BATTERY_CRITICAL_TEMPERATURE)
    }

    /// Get the name of this temperature monitor
    pub async fn name(&self) -> Result<String> {
        Ok("Battery Temperature Monitor".to_string())
    }
}

#[async_trait]
impl HardwareMonitor for BatteryTemperatureMonitor {
    type MetricType = TemperatureType;

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let temperature = self.temperature().await?;
        Ok(Metric {
            value: TemperatureType::new_celsius(temperature),
            timestamp: SystemTime::now(),
        })
    }

    async fn name(&self) -> Result<String> {
        self.name().await
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Battery".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("battery0".to_string())
    }
}

#[async_trait]
impl TemperatureMonitor for BatteryTemperatureMonitor {
    async fn temperature(&self) -> Result<f64> {
        let info = self.io_kit.get_thermal_info()?;
        info.battery_temp.ok_or_else(|| crate::error::Error::NotAvailable {
            resource: "Battery temperature".to_string(),
            reason: "Hardware info not available".to_string(),
        })
    }
}

#[async_trait]
impl TemperatureMonitorExt for BatteryTemperatureMonitor {
    async fn is_critical(&self) -> Result<bool> {
        match self.temperature().await {
            Ok(temp) => Ok(temp >= BATTERY_CRITICAL_TEMPERATURE),
            Err(_) => Ok(false),
        }
    }

    async fn critical_threshold(&self) -> Result<f64> {
        Ok(BATTERY_CRITICAL_TEMPERATURE)
    }
}

// SSD Temperature Monitor
//
/// Monitor for SSD temperature
pub struct SsdTemperatureMonitor {
    /// IOKit service for accessing hardware information
    io_kit: Arc<Box<dyn IOKit>>,
}

impl SsdTemperatureMonitor {
    /// Create a new SSD temperature monitor
    pub fn new(io_kit: Arc<Box<dyn IOKit>>) -> Self {
        Self { io_kit }
    }

    /// Get the current SSD temperature in Celsius
    pub async fn temperature(&self) -> Result<f64> {
        let info = self.io_kit.get_thermal_info()?;

        // SSD temp is not in the standard fields, try to get it from the dictionary
        info.get_number("SSD_TEMPERATURE")
            .or_else(|| info.get_number("DRIVE_TEMPERATURE"))
            .or_else(|| info.get_number("NVME_TEMPERATURE"))
            .ok_or_else(|| crate::error::Error::NotAvailable {
                resource: "SSD temperature".to_string(),
                reason: "Hardware info not available".to_string(),
            })
    }

    /// Check if SSD temperature is at a critical level
    pub async fn is_critical(&self) -> Result<bool> {
        match self.temperature().await {
            Ok(temp) => Ok(temp >= SSD_CRITICAL_TEMPERATURE),
            Err(_) => Ok(false),
        }
    }

    /// Get the critical temperature threshold for SSD
    pub async fn critical_threshold(&self) -> Result<f64> {
        Ok(SSD_CRITICAL_TEMPERATURE)
    }

    /// Get the name of this temperature monitor
    pub async fn name(&self) -> Result<String> {
        Ok("SSD Temperature Monitor".to_string())
    }
}

#[async_trait]
impl HardwareMonitor for SsdTemperatureMonitor {
    type MetricType = TemperatureType;

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let temperature = self.temperature().await?;
        Ok(Metric {
            value: TemperatureType::new_celsius(temperature),
            timestamp: SystemTime::now(),
        })
    }

    async fn name(&self) -> Result<String> {
        self.name().await
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("SSD".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("ssd0".to_string())
    }
}

#[async_trait]
impl TemperatureMonitor for SsdTemperatureMonitor {
    async fn temperature(&self) -> Result<f64> {
        let info = self.io_kit.get_thermal_info()?;

        // SSD temp is not in the standard fields, try to get it from the dictionary
        info.get_number("SSD_TEMPERATURE")
            .or_else(|| info.get_number("DRIVE_TEMPERATURE"))
            .or_else(|| info.get_number("NVME_TEMPERATURE"))
            .ok_or_else(|| crate::error::Error::NotAvailable {
                resource: "SSD temperature".to_string(),
                reason: "Hardware info not available".to_string(),
            })
    }
}

/// Common trait for temperature monitors
#[async_trait]
pub trait TemperatureMonitorTrait: Send + Sync {
    /// Get the current temperature in Celsius
    async fn temperature(&self) -> Result<f64>;

    /// Check if temperature is at a critical level
    async fn is_critical(&self) -> Result<bool>;

    /// Get the critical temperature threshold
    async fn critical_threshold(&self) -> Result<f64>;

    /// Get the name of the monitor
    async fn name(&self) -> Result<String>;
}

#[async_trait]
impl TemperatureMonitorTrait for CpuTemperatureMonitor {
    async fn temperature(&self) -> Result<f64> {
        <CpuTemperatureMonitor as TemperatureMonitor>::temperature(self).await
    }

    async fn is_critical(&self) -> Result<bool> {
        TemperatureMonitorExt::is_critical(self).await
    }

    async fn critical_threshold(&self) -> Result<f64> {
        TemperatureMonitorExt::critical_threshold(self).await
    }

    async fn name(&self) -> Result<String> {
        HardwareMonitor::name(self).await
    }
}

#[async_trait]
impl TemperatureMonitorTrait for GpuTemperatureMonitor {
    async fn temperature(&self) -> Result<f64> {
        <GpuTemperatureMonitor as TemperatureMonitor>::temperature(self).await
    }

    async fn is_critical(&self) -> Result<bool> {
        TemperatureMonitorExt::is_critical(self).await
    }

    async fn critical_threshold(&self) -> Result<f64> {
        TemperatureMonitorExt::critical_threshold(self).await
    }

    async fn name(&self) -> Result<String> {
        HardwareMonitor::name(self).await
    }
}

#[async_trait]
impl TemperatureMonitorTrait for AmbientTemperatureMonitor {
    async fn temperature(&self) -> Result<f64> {
        self.temperature().await
    }

    async fn is_critical(&self) -> Result<bool> {
        self.is_critical().await
    }

    async fn critical_threshold(&self) -> Result<f64> {
        self.critical_threshold().await
    }

    async fn name(&self) -> Result<String> {
        self.name().await
    }
}

#[async_trait]
impl TemperatureMonitorTrait for BatteryTemperatureMonitor {
    async fn temperature(&self) -> Result<f64> {
        self.temperature().await
    }

    async fn is_critical(&self) -> Result<bool> {
        self.is_critical().await
    }

    async fn critical_threshold(&self) -> Result<f64> {
        self.critical_threshold().await
    }

    async fn name(&self) -> Result<String> {
        self.name().await
    }
}

#[async_trait]
impl TemperatureMonitorTrait for SsdTemperatureMonitor {
    async fn temperature(&self) -> Result<f64> {
        self.temperature().await
    }

    async fn is_critical(&self) -> Result<bool> {
        self.is_critical().await
    }

    async fn critical_threshold(&self) -> Result<f64> {
        self.critical_threshold().await
    }

    async fn name(&self) -> Result<String> {
        self.name().await
    }
}

/// Creates a temperature monitor based on the specified type
pub fn create_monitor(monitor_type: &str, io_kit: Arc<Box<dyn IOKit>>) -> Option<Box<dyn TemperatureMonitorTrait>> {
    match monitor_type.to_lowercase().as_str() {
        "cpu" => Some(Box::new(CpuTemperatureMonitor::new(io_kit))),
        "gpu" => Some(Box::new(GpuTemperatureMonitor::new(io_kit))),
        "ambient" => Some(Box::new(AmbientTemperatureMonitor::new(io_kit))),
        "battery" => Some(Box::new(BatteryTemperatureMonitor::new(io_kit))),
        "ssd" => Some(Box::new(SsdTemperatureMonitor::new(io_kit))),
        _ => None,
    }
}
