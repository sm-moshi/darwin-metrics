//! Temperature monitoring module
//!
//! This module provides comprehensive temperature monitoring capabilities for macOS systems.
//! It tracks temperatures for various components including CPU, GPU, battery, and ambient sensors,
//! as well as fan speeds and thermal throttling detection.
//!
//! # Features
//!
//! - CPU, GPU, and battery temperature monitoring
//! - Ambient temperature sensing
//! - Fan speed monitoring and control
//! - Thermal throttling detection
//! - Configurable temperature thresholds

/// Temperature monitoring constants
pub mod constants;

/// Temperature monitoring types
pub mod types;

/// Temperature monitoring implementations
pub mod monitors;

/// Temperature monitoring factory
pub mod factory;

pub use constants::*;
pub use types::*;
pub use monitors::*;
pub use factory::*;

use crate::error::Result;
use crate::hardware::iokit::{IOKit, IOKitImpl};
use crate::traits::{FanMonitor, ThermalMonitor};
use std::sync::Arc;

/// Temperature monitoring for various components
#[derive(Debug, Clone)]
pub struct Temperature {
    iokit: Arc<Box<dyn IOKit>>,
    config: TemperatureConfig,
    factory: TemperatureMonitorFactory,
}

impl Temperature {
    /// Create a new Temperature monitor with default configuration
    pub fn new() -> Result<Self> {
        let iokit = Arc::new(Box::new(IOKitImpl::new()?));
        Ok(Self::with_iokit(iokit))
    }
    
    /// Create a new Temperature monitor with custom configuration
    pub fn with_config(config: TemperatureConfig) -> Result<Self> {
        let iokit = Arc::new(Box::new(IOKitImpl::new()?));
        let factory = TemperatureMonitorFactory::new(iokit.clone());
        Ok(Self {
            iokit,
            config,
            factory,
        })
    }
    
    /// Create a new Temperature monitor with a custom IOKit implementation
    pub fn with_iokit(iokit: Arc<Box<dyn IOKit>>) -> Self {
        let factory = TemperatureMonitorFactory::new(iokit.clone());
        Self {
            iokit,
            config: TemperatureConfig::default(),
            factory,
        }
    }
    
    /// Get a CPU temperature monitor
    pub fn cpu_monitor(&self) -> CpuTemperatureMonitor {
        CpuTemperatureMonitor::new(self.iokit.clone())
    }
    
    /// Get a GPU temperature monitor
    pub fn gpu_monitor(&self) -> GpuTemperatureMonitor {
        GpuTemperatureMonitor::new(self.iokit.clone())
    }
    
    /// Get an ambient temperature monitor
    pub fn ambient_monitor(&self) -> AmbientTemperatureMonitor {
        AmbientTemperatureMonitor::new(self.iokit.clone())
    }
    
    /// Get a battery temperature monitor
    pub fn battery_monitor(&self) -> BatteryTemperatureMonitor {
        BatteryTemperatureMonitor::new(self.iokit.clone())
    }
    
    /// Get an SSD temperature monitor
    pub fn ssd_monitor(&self) -> SsdTemperatureMonitor {
        SsdTemperatureMonitor::new(self.iokit.clone())
    }
    
    /// Get a fan monitor
    pub fn fan_monitor(&self) -> FanMonitor {
        FanMonitor::new(self.iokit.clone())
    }
    
    /// Get all available temperature monitors
    pub fn get_all_monitors(&self) -> Vec<Box<dyn TemperatureMonitorTrait>> {
        self.factory.create_all()
    }
    
    /// Create a specific temperature monitor by type
    pub fn create_monitor(&self, monitor_type: &str) -> Result<Box<dyn TemperatureMonitorTrait>> {
        self.factory.create(monitor_type)
    }
}

#[async_trait::async_trait]
impl ThermalMonitor for Temperature {
    async fn cpu_temperature(&self) -> Result<Option<f64>> {
        let monitor = self.cpu_monitor();
        Ok(Some(monitor.temperature().await?))
    }

    async fn gpu_temperature(&self) -> Result<Option<f64>> {
        let monitor = self.gpu_monitor();
        Ok(Some(monitor.temperature().await?))
    }

    async fn memory_temperature(&self) -> Result<Option<f64>> {
        // Memory temperature monitoring is not available on Mac
        Ok(None)
    }

    async fn battery_temperature(&self) -> Result<Option<f64>> {
        let monitor = self.battery_monitor();
        Ok(Some(monitor.temperature().await?))
    }

    async fn ambient_temperature(&self) -> Result<Option<f64>> {
        let monitor = self.ambient_monitor();
        Ok(Some(monitor.temperature().await?))
    }

    async fn ssd_temperature(&self) -> Result<Option<f64>> {
        let monitor = self.ssd_monitor();
        Ok(Some(monitor.temperature().await?))
    }

    async fn is_throttling(&self) -> Result<bool> {
        // Check if any temperature monitor is reporting critical temperatures
        let cpu_temp = self.cpu_temperature().await?.unwrap_or(0.0);
        let gpu_temp = self.gpu_temperature().await?.unwrap_or(0.0);
        let battery_temp = self.battery_temperature().await?.unwrap_or(0.0);
        let ssd_temp = self.ssd_temperature().await?.unwrap_or(0.0);
        
        // Check if any temperature exceeds the throttling threshold
        Ok(cpu_temp > self.config.throttling_threshold 
           || gpu_temp > self.config.throttling_threshold 
           || battery_temp > self.config.throttling_threshold
           || ssd_temp > SSD_CRITICAL_TEMPERATURE)
    }

    async fn get_fans(&self) -> Result<Vec<Fan>> {
        let fan_monitor = self.fan_monitor();
        let speed = fan_monitor.speed_rpm().await?;
        let min_speed = fan_monitor.min_speed().await?;
        let max_speed = fan_monitor.max_speed().await?;
        let utilization = fan_monitor.percentage().await?;
        let name = fan_monitor.fan_name().await?;
        
        Ok(vec![Fan {
            name,
            speed: speed as f64,
            min_speed: min_speed as f64,
            max_speed: max_speed as f64,
            utilization,
        }])
    }

    async fn get_thermal_metrics(&self) -> Result<ThermalMetrics> {
        let cpu_temp = self.cpu_temperature().await?;
        let gpu_temp = self.gpu_temperature().await?;
        let memory_temp = self.memory_temperature().await?;
        let battery_temp = self.battery_temperature().await?;
        let ambient_temp = self.ambient_temperature().await?;
        let ssd_temp = self.ssd_temperature().await?;
        let is_throttling = self.is_throttling().await?;
        let fans = self.get_fans().await?;
        
        Ok(ThermalMetrics {
            cpu_temperature: cpu_temp,
            gpu_temperature: gpu_temp,
            memory_temperature: memory_temp,
            battery_temperature: battery_temp,
            ambient_temperature: ambient_temp,
            ssd_temperature: ssd_temp,
            is_throttling,
            fans,
        })
    }
} 