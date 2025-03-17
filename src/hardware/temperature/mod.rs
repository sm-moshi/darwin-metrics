//! Temperature monitoring module
//!
//! This module provides comprehensive temperature monitoring capabilities for macOS systems.
//! It tracks temperatures for various components including CPU, GPU, battery, and ambient sensors,
//! as well as fan speeds and thermal throttling status.
//!
//! # Features
//!
//! - CPU, GPU, and battery temperature monitoring
//! - Ambient temperature sensing
//! - Fan speed monitoring and control
//! - Thermal throttling detection
//! - Configurable temperature thresholds
//!
//! # Examples
//!
//! Basic temperature monitoring:
//!
//! ```no_run
//! use darwin_metrics::hardware::temperature::{Temperature, TemperatureMonitor};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let temp = Temperature::new()?;
//!     let cpu_monitor = temp.cpu_monitor();
//!     let gpu_monitor = temp.gpu_monitor();
//!
//!     println!("CPU Temperature: {:.1}°C", cpu_monitor.temperature().await?);
//!     println!("GPU Temperature: {:.1}°C", gpu_monitor.temperature().await?);
//!     
//!     Ok(())
//! }
//! ```

/// Temperature monitoring constants
pub mod constants;

mod monitors;
mod types;

pub use monitors::*;
pub use types::*;

use crate::error::Result;
use crate::hardware::iokit::{IOKit, IOKitImpl};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Temperature {
    iokit: Arc<Box<dyn IOKit>>,
    config: TemperatureConfig,
}

impl Temperature {
    /// Creates a new Temperature instance with default configuration
    pub fn new() -> Result<Self> {
        let iokit_impl = IOKitImpl::new()?;
        Ok(Self { iokit: Arc::new(Box::new(iokit_impl)), config: TemperatureConfig::default() })
    }

    /// Creates a new Temperature instance with custom configuration
    pub fn with_config(config: TemperatureConfig) -> Result<Self> {
        let iokit_impl = IOKitImpl::new()?;
        Ok(Self { iokit: Arc::new(Box::new(iokit_impl)), config })
    }

    /// Get a monitor for CPU temperature
    pub fn cpu_monitor(&self) -> CpuTemperatureMonitor {
        CpuTemperatureMonitor::new(Arc::clone(&self.iokit))
    }

    /// Get a monitor for GPU temperature
    pub fn gpu_monitor(&self) -> GpuTemperatureMonitor {
        GpuTemperatureMonitor::new(Arc::clone(&self.iokit))
    }

    /// Get a monitor for ambient temperature
    pub fn ambient_monitor(&self) -> AmbientTemperatureMonitor {
        AmbientTemperatureMonitor::new(Arc::clone(&self.iokit))
    }

    /// Get a monitor for battery temperature
    pub fn battery_monitor(&self) -> BatteryTemperatureMonitor {
        BatteryTemperatureMonitor::new(Arc::clone(&self.iokit))
    }

    /// Get a monitor for a specific fan
    pub fn fan_monitor(&self, index: usize) -> FanMonitor {
        FanMonitor::new(Arc::clone(&self.iokit), index)
    }

    /// Get the current configuration
    pub fn config(&self) -> &TemperatureConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: TemperatureConfig) {
        self.config = config;
    }

    /// Get comprehensive thermal metrics
    pub async fn get_thermal_metrics(&self) -> Result<ThermalMetrics> {
        let cpu_temp = self.cpu_monitor().temperature().await.ok();
        let gpu_temp = self.gpu_monitor().temperature().await.ok();
        let ambient_temp = self.ambient_monitor().temperature().await.ok();
        let battery_temp = self.battery_monitor().temperature().await.ok();

        let mut fans = Vec::new();
        let fan_info = self.iokit.get_all_fans()?;
        for (i, info) in fan_info.iter().enumerate() {
            fans.push(Fan {
                name: format!("Fan {}", i),
                speed_rpm: info.speed_rpm,
                min_speed: info.min_speed,
                max_speed: info.max_speed,
                percentage: if info.max_speed > info.min_speed {
                    ((info.speed_rpm - info.min_speed) as f64 / (info.max_speed - info.min_speed) as f64) * 100.0
                } else {
                    0.0
                },
            });
        }

        Ok(ThermalMetrics {
            cpu_temperature: cpu_temp,
            gpu_temperature: gpu_temp,
            heatsink_temperature: None, // Not implemented yet
            ambient_temperature: ambient_temp,
            battery_temperature: battery_temp,
            is_throttling: self.iokit.check_thermal_throttling("IOService")?,
            cpu_power: None, // Not implemented yet
            fans,
            last_refresh: std::time::Instant::now(),
        })
    }
}

impl Default for Temperature {
    fn default() -> Self {
        Self::new().expect("Failed to create Temperature instance")
    }
}
