// Temperature module
//
// This module is responsible for monitoring the temperature of various hardware
// components, including CPU, GPU, memory, battery, and SSD. It also provides
// information about the fan status and whether the system is throttling.

use std::sync::Arc;

// Submodules
mod constants;
mod monitors;
mod types;

// Re-export items from the submodules
pub use constants::*;
pub use monitors::*;
pub use types::*;

// Only import what we need for this file
use crate::{
    error::{Error, Result},
    hardware::iokit::IOKit,
    traits::hardware::ThermalMonitor,
};

use self::types::{Fan, MonitorType, ThermalInfo};

// Temperature struct implementation
pub struct Temperature {
    io_kit: Arc<dyn IOKit>,
}

impl Temperature {
    /// Creates a new instance of Temperature
    ///
    /// # Arguments
    ///
    /// * `io_kit` - An implementation of the IOKit trait
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - A new instance of Temperature
    pub fn new(io_kit: Arc<dyn IOKit>) -> Result<Self> {
        Ok(Self { io_kit })
    }

    /// Get all temperature metrics
    ///
    /// # Returns
    ///
    /// * `Result<ThermalMetrics>` - All temperature metrics
    pub async fn all_temperature_metrics(&self) -> Result<types::ThermalMetrics> {
        let thermal_info = self.io_kit.get_thermal_info()?;
        let fans = self.io_kit.get_all_fans().await?;
        
        // Convert FanInfo to Fan 
        let fan_list = fans.into_iter()
            .map(|info| Fan {
                id: info.index as u32,
                speed: info.speed_rpm,
                min_speed: info.min_speed.unwrap_or(0),
                max_speed: info.max_speed.unwrap_or(0),
                current_speed: info.current_speed,
                target_speed: info.target_speed,
            })
            .collect();
        
        Ok(types::ThermalMetrics {
            cpu_temp: Some(thermal_info.cpu_temp),
            gpu_temp: thermal_info.gpu_temp,
            memory_temp: None, // Not available
            battery_temp: thermal_info.battery_temp,
            ambient_temp: thermal_info.ambient_temp,
            ssd_temp: None, // Not available
            is_throttling: Some(thermal_info.thermal_throttling),
            fans: fan_list,
        })
    }

    /// Get temperature details
    ///
    /// # Returns
    ///
    /// * `Result<ThermalDetails>` - Temperature details
    pub async fn temperature_details(&self) -> Result<types::ThermalDetails> {
        let thermal_info = self.io_kit.get_thermal_info()?;
        let fans = self.io_kit.get_all_fans().await?;
        
        // Convert FanInfo to Fan
        let fan_list = fans.into_iter()
            .map(|info| Fan {
                id: info.index as u32,
                speed: info.speed_rpm,
                min_speed: info.min_speed.unwrap_or(0),
                max_speed: info.max_speed.unwrap_or(0),
                current_speed: info.current_speed,
                target_speed: info.target_speed,
            })
            .collect();
        
        Ok(types::ThermalDetails {
            cpu_temp: Some(thermal_info.cpu_temp),
            gpu_temp: thermal_info.gpu_temp,
            memory_temp: None, // Not available
            battery_temp: thermal_info.battery_temp,
            ambient_temp: thermal_info.ambient_temp,
            ssd_temp: None, // Not available
            is_throttling: Some(thermal_info.thermal_throttling),
            fans: fan_list,
        })
    }

    /// Get all fan details
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Fan>>` - All fan details
    pub async fn all_fan_details(&self) -> Result<Vec<Fan>> {
        let fan_info = self.io_kit.get_all_fans().await?;
        // Convert FanInfo to Fan
        let fans = fan_info.into_iter()
            .map(|info| Fan {
                id: info.index as u32,
                speed: info.speed_rpm,
                min_speed: info.min_speed.unwrap_or(0),
                max_speed: info.max_speed.unwrap_or(0),
                current_speed: info.current_speed,
                target_speed: info.target_speed,
            })
            .collect();
        Ok(fans)
    }

    /// Get temperature monitor
    ///
    /// # Arguments
    ///
    /// * `monitor_type` - The type of monitor to get
    ///
    /// # Returns
    ///
    /// * `Result<Box<dyn TemperatureMonitor>>` - The temperature monitor
    pub async fn get_temperature_monitor(
        &self,
        monitor_type: MonitorType,
    ) -> Result<Box<dyn crate::traits::hardware::TemperatureMonitor>> {
        match monitor_type {
            MonitorType::Cpu => Ok(Box::new(monitors::CpuTemperatureMonitor::new(
                self.io_kit.clone(),
            )?)),
            MonitorType::Gpu => Ok(Box::new(monitors::GpuTemperatureMonitor::new(
                self.io_kit.clone(),
            )?)),
            MonitorType::Memory => Ok(Box::new(monitors::MemoryTemperatureMonitor::new(
                self.io_kit.clone(),
            )?)),
            MonitorType::Battery => Ok(Box::new(monitors::BatteryTemperatureMonitor::new(
                self.io_kit.clone(),
            )?)),
            MonitorType::Ambient => Ok(Box::new(monitors::AmbientTemperatureMonitor::new(
                self.io_kit.clone(),
            )?)),
            MonitorType::Ssd => Ok(Box::new(monitors::SsdTemperatureMonitor::new(
                self.io_kit.clone(),
            )?)),
            _ => Err(Error::Temperature(format!(
                "Unsupported monitor type: {:?}",
                monitor_type
            ))),
        }
    }

    /// Get fans
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Fan>>` - All fans
    pub async fn get_fans(&self) -> Result<Vec<Fan>> {
        self.all_fan_details().await
    }

    /// Get thermal metrics
    ///
    /// # Returns
    ///
    /// * `Result<ThermalMetrics>` - Thermal metrics
    pub async fn get_thermal_metrics(&self) -> Result<types::ThermalMetrics> {
        self.all_temperature_metrics().await
    }
}

#[async_trait::async_trait]
impl ThermalMonitor for Temperature {
    async fn cpu_temperature(&self) -> Option<f64> {
        match self.io_kit.get_thermal_info() {
            Ok(info) => Some(info.cpu_temp),
            Err(_) => None,
        }
    }

    async fn gpu_temperature(&self) -> Option<f64> {
        match self.io_kit.get_thermal_info() {
            Ok(info) => info.gpu_temp,
            Err(_) => None,
        }
    }

    async fn memory_temperature(&self) -> Option<f64> {
        None // Not available in ThermalInfo
    }

    async fn battery_temperature(&self) -> Option<f64> {
        match self.io_kit.get_thermal_info() {
            Ok(info) => info.battery_temp,
            Err(_) => None,
        }
    }

    async fn ambient_temperature(&self) -> Option<f64> {
        match self.io_kit.get_thermal_info() {
            Ok(info) => info.ambient_temp,
            Err(_) => None,
        }
    }

    async fn ssd_temperature(&self) -> Option<f64> {
        None // Not available in ThermalInfo
    }

    async fn is_throttling(&self) -> bool {
        match self.io_kit.get_thermal_info() {
            Ok(info) => info.thermal_throttling,
            Err(_) => false,
        }
    }

    async fn get_fans(&self) -> Result<Vec<Fan>> {
        self.all_fan_details().await
    }

    async fn get_thermal_metrics(&self) -> Result<types::ThermalMetrics> {
        self.all_temperature_metrics().await
    }
} 