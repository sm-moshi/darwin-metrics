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
use crate::traits::{FanMonitor, TemperatureMonitor};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::SystemTime;

//
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
impl crate::traits::FanMonitor for FanMonitor {
    /// Get the current fan speed in RPM
    async fn speed_rpm(&self) -> Result<u32> {
        let fans = self.io_kit.get_all_fans()?;
        fans.get(self.index).map(|fan| fan.speed_rpm).ok_or_else(|| crate::error::Error::NotAvailable {
            resource: format!("Fan {}", self.index),
            reason: "Not found".to_string(),
        })
    }

    /// Get the minimum fan speed in RPM
    async fn min_speed(&self) -> Result<u32> {
        let fans = self.io_kit.get_all_fans()?;
        fans.get(self.index).map(|fan| fan.min_speed).ok_or_else(|| crate::error::Error::NotAvailable {
            resource: format!("Fan {}", self.index),
            reason: "Not found".to_string(),
        })
    }

    /// Get the maximum fan speed in RPM
    async fn max_speed(&self) -> Result<u32> {
        let fans = self.io_kit.get_all_fans()?;
        fans.get(self.index).map(|fan| fan.max_speed).ok_or_else(|| crate::error::Error::NotAvailable {
            resource: format!("Fan {}", self.index),
            reason: "Not found".to_string(),
        })
    }

    /// Get the current fan utilization as a percentage (0-100%)
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

    /// Get the fan name
    async fn fan_name(&self) -> Result<String> {
        Ok(format!("Fan {}", self.index))
    }
}

//
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

    /// Check if CPU temperature is at a critical level
    pub async fn is_critical(&self) -> Result<bool> {
        let temp = self.temperature().await?;
        Ok(temp >= CPU_CRITICAL_TEMPERATURE)
    }

    /// Get the critical temperature threshold for CPU
    pub async fn critical_threshold(&self) -> Result<f64> {
        Ok(CPU_CRITICAL_TEMPERATURE)
    }
}

#[async_trait]
impl TemperatureMonitor for CpuTemperatureMonitor {
    /// Get the current temperature metric
    async fn get_metric(&self) -> Result<Metric<TemperatureType>> {
        let temperature = self.temperature().await?;
        Ok(Metric {
            value: TemperatureType::from_celsius(temperature),
            timestamp: SystemTime::now(),
        })
    }

    /// Get the name of the temperature monitor
    async fn name(&self) -> Result<String> {
        Ok("CPU Temperature Monitor".to_string())
    }

    /// Get the type of hardware component being monitored
    async fn hardware_type(&self) -> Result<String> {
        Ok("CPU".to_string())
    }

    /// Get the unique device identifier
    async fn device_id(&self) -> Result<String> {
        Ok("cpu0".to_string())
    }
}

//
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
impl TemperatureMonitor for GpuTemperatureMonitor {
    /// Get the current temperature metric
    async fn get_metric(&self) -> Result<Metric<TemperatureType>> {
        let temperature = self.temperature().await?;
        Ok(Metric {
            value: TemperatureType::from_celsius(temperature),
            timestamp: SystemTime::now(),
        })
    }

    /// Get the name of the temperature monitor
    async fn name(&self) -> Result<String> {
        Ok("GPU Temperature Monitor".to_string())
    }

    /// Get the type of hardware component being monitored
    async fn hardware_type(&self) -> Result<String> {
        Ok("GPU".to_string())
    }

    /// Get the unique device identifier
    async fn device_id(&self) -> Result<String> {
        Ok("gpu0".to_string())
    }
}

//
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
            resource: "Ambient Temperature".to_string(),
            reason: "Not supported on this device".to_string(),
        })
    }

    /// Check if ambient temperature is at a critical level
    pub async fn is_critical(&self) -> Result<bool> {
        match self.temperature().await {
            Ok(temp) => Ok(temp >= AMBIENT_CRITICAL_TEMPERATURE),
            Err(_) => Ok(false), // If we can't read the temperature, assume it's not critical
        }
    }

    /// Get the critical temperature threshold for ambient temperature
    pub async fn critical_threshold(&self) -> Result<f64> {
        Ok(AMBIENT_CRITICAL_TEMPERATURE)
    }
}

#[async_trait]
impl TemperatureMonitor for AmbientTemperatureMonitor {
    /// Get the current temperature metric
    async fn get_metric(&self) -> Result<Metric<TemperatureType>> {
        let temperature = self.temperature().await?;
        Ok(Metric {
            value: TemperatureType::from_celsius(temperature),
            timestamp: SystemTime::now(),
        })
    }

    /// Get the name of the temperature monitor
    async fn name(&self) -> Result<String> {
        Ok("Ambient Temperature Monitor".to_string())
    }

    /// Get the type of hardware component being monitored
    async fn hardware_type(&self) -> Result<String> {
        Ok("Ambient".to_string())
    }

    /// Get the unique device identifier
    async fn device_id(&self) -> Result<String> {
        Ok("ambient0".to_string())
    }
}

//
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
            resource: "Battery Temperature".to_string(),
            reason: "Not supported on this device".to_string(),
        })
    }

    /// Check if battery temperature is at a critical level
    pub async fn is_critical(&self) -> Result<bool> {
        match self.temperature().await {
            Ok(temp) => Ok(temp >= BATTERY_CRITICAL_TEMPERATURE),
            Err(_) => Ok(false), // If we can't read the temperature, assume it's not critical
        }
    }

    /// Get the critical temperature threshold for battery
    pub async fn critical_threshold(&self) -> Result<f64> {
        Ok(BATTERY_CRITICAL_TEMPERATURE)
    }
}

#[async_trait]
impl TemperatureMonitor for BatteryTemperatureMonitor {
    /// Get the current temperature metric
    async fn get_metric(&self) -> Result<Metric<TemperatureType>> {
        let temperature = self.temperature().await?;
        Ok(Metric {
            value: TemperatureType::from_celsius(temperature),
            timestamp: SystemTime::now(),
        })
    }

    /// Get the name of the temperature monitor
    async fn name(&self) -> Result<String> {
        Ok("Battery Temperature Monitor".to_string())
    }

    /// Get the type of hardware component being monitored
    async fn hardware_type(&self) -> Result<String> {
        Ok("Battery".to_string())
    }

    /// Get the unique device identifier
    async fn device_id(&self) -> Result<String> {
        Ok("battery0".to_string())
    }
}

//
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
        info.ssd_temp.ok_or_else(|| crate::error::Error::NotAvailable {
            resource: "SSD Temperature".to_string(),
            reason: "Not supported on this device".to_string(),
        })
    }

    /// Check if SSD temperature is at a critical level
    pub async fn is_critical(&self) -> Result<bool> {
        match self.temperature().await {
            Ok(temp) => Ok(temp >= SSD_CRITICAL_TEMPERATURE),
            Err(_) => Ok(false), // If we can't read the temperature, assume it's not critical
        }
    }

    /// Get the critical temperature threshold for SSD
    pub async fn critical_threshold(&self) -> Result<f64> {
        Ok(SSD_CRITICAL_TEMPERATURE)
    }
}

#[async_trait]
impl TemperatureMonitor for SsdTemperatureMonitor {
    /// Get the current temperature metric
    async fn get_metric(&self) -> Result<Metric<TemperatureType>> {
        let temperature = self.temperature().await?;
        Ok(Metric {
            value: TemperatureType::from_celsius(temperature),
            timestamp: SystemTime::now(),
        })
    }

    /// Get the name of the temperature monitor
    async fn name(&self) -> Result<String> {
        Ok("SSD Temperature Monitor".to_string())
    }

    /// Get the type of hardware component being monitored
    async fn hardware_type(&self) -> Result<String> {
        Ok("SSD".to_string())
    }

    /// Get the unique device identifier
    async fn device_id(&self) -> Result<String> {
        Ok("ssd0".to_string())
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
