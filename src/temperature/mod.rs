// Temperature module
//
// This module is responsible for monitoring the temperature of various hardware
// components, including CPU, GPU, memory, battery, and SSD. It also provides
// information about the fan status and whether the system is throttling.

use std::sync::Arc;

pub use constants::*;
pub use factory::*;
pub use monitors::*;
pub use types::*;

use crate::core::metrics::hardware::ThermalMonitor;
use crate::error::{Error, Result};
use crate::hardware::iokit::{IOKit, IOKitImpl};
use crate::system::System;
use crate::temperature::types::{Fan, ThermalLevel, ThermalMetrics};
use crate::traits::FanMonitor as FanMonitorTrait;

/// Temperature monitoring constants
pub mod constants;

// Submodules
mod constants;
mod monitors;
mod types;

/// Temperature monitoring implementations
pub mod monitors;

/// Temperature monitoring factory
pub mod factory;

/// Temperature monitoring for various components
#[derive(Debug, Clone)]
pub struct TemperatureFactory {
    iokit: Arc<Box<dyn IOKit>>,
    config: TemperatureConfig,
    factory: TemperatureMonitorFactory,
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

    /// Create a new Temperature monitor with a custom IOKit implementation
    pub fn with_iokit(iokit: Arc<Box<dyn IOKit>>) -> Self {
        let factory = TemperatureMonitorFactory::new(iokit.clone());
        Self {
            iokit,
            config: Default::default(),
            factory,
        }
    }

    /// Get a CPU temperature monitor
    pub fn cpu_monitor(&self) -> Box<dyn TemperatureMonitorTrait> {
        Box::new(CpuTemperatureMonitor::new(self.iokit.clone()))
    }

    /// Get a GPU temperature monitor
    pub fn gpu_monitor(&self) -> Box<dyn TemperatureMonitorTrait> {
        Box::new(GpuTemperatureMonitor::new(self.iokit.clone()))
    }

    /// Get an ambient temperature monitor
    pub fn ambient_monitor(&self) -> Box<dyn TemperatureMonitorTrait> {
        Box::new(AmbientTemperatureMonitor::new(self.iokit.clone()))
    }

    /// Get a battery temperature monitor
    pub fn battery_monitor(&self) -> Box<dyn TemperatureMonitorTrait> {
        Box::new(BatteryTemperatureMonitor::new(self.iokit.clone()))
    }

    /// Get an SSD temperature monitor
    pub fn ssd_monitor(&self) -> Box<dyn TemperatureMonitorTrait> {
        Box::new(SsdTemperatureMonitor::new(self.iokit.clone()))
    }

    /// Get a fan monitor
    pub fn fan_monitor(&self) -> FanMonitor {
        FanMonitor::new(self.iokit.clone(), 0)
    }

    /// Get all available temperature monitors
    pub async fn get_all_monitors(&self) -> Result<Vec<Box<dyn TemperatureMonitorTrait>>> {
        let monitors = self.factory.create_all();
        let mut result = Vec::new();

        for monitor in monitors {
            let monitor_type = monitor.name().await?;
            if let Some(m) = monitors::create_monitor(&monitor_type, self.iokit.clone()) {
                result.push(m);
            }
        }

        Ok(result)
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

/// Main temperature module implementation
pub struct Temperature {
    /// IOKit interface for hardware monitoring access
    iokit: Arc<Box<dyn IOKit>>,
    /// Temperature module configuration
    config: TemperatureConfig,
    /// Factory for creating specialized temperature monitors
    factory: TemperatureMonitorFactory,
}

impl Temperature {
    /// Create a new Temperature monitor with default configuration
    pub fn new() -> Result<Self> {
        let iokit_impl = IOKitImpl::new()?;
        let iokit: Arc<Box<dyn IOKit>> = Arc::new(Box::new(iokit_impl));
        let factory = TemperatureMonitorFactory::new(iokit.clone());
        Ok(Self {
            iokit,
            config: Default::default(),
            factory,
        })
    }

    /// Create a new Temperature monitor with custom configuration
    pub fn with_config(config: TemperatureConfig) -> Result<Self> {
        let iokit_impl = IOKitImpl::new()?;
        let iokit: Arc<Box<dyn IOKit>> = Arc::new(Box::new(iokit_impl));
        let factory = TemperatureMonitorFactory::new(iokit.clone());
        Ok(Self { iokit, config, factory })
    }

    /// Create a new Temperature monitor with a custom IOKit implementation
    pub fn with_iokit(iokit: Arc<Box<dyn IOKit>>) -> Self {
        let factory = TemperatureMonitorFactory::new(iokit.clone());
        Self {
            iokit,
            config: Default::default(),
            factory,
        }
    }

    /// Get a CPU temperature monitor
    pub fn cpu_monitor(&self) -> Box<dyn TemperatureMonitorTrait> {
        Box::new(CpuTemperatureMonitor::new(self.iokit.clone()))
    }

    /// Get a GPU temperature monitor
    pub fn gpu_monitor(&self) -> Box<dyn TemperatureMonitorTrait> {
        Box::new(GpuTemperatureMonitor::new(self.iokit.clone()))
    }

    /// Get an ambient temperature monitor
    pub fn ambient_monitor(&self) -> Box<dyn TemperatureMonitorTrait> {
        Box::new(AmbientTemperatureMonitor::new(self.iokit.clone()))
    }

    /// Get a battery temperature monitor
    pub fn battery_monitor(&self) -> Box<dyn TemperatureMonitorTrait> {
        Box::new(BatteryTemperatureMonitor::new(self.iokit.clone()))
    }

    /// Get an SSD temperature monitor
    pub fn ssd_monitor(&self) -> Box<dyn TemperatureMonitorTrait> {
        Box::new(SsdTemperatureMonitor::new(self.iokit.clone()))
    }

    /// Get a fan monitor
    pub fn fan_monitor(&self) -> FanMonitor {
        FanMonitor::new(self.iokit.clone(), 0)
    }

    /// Get all available temperature monitors
    pub async fn get_all_monitors(&self) -> Result<Vec<Box<dyn TemperatureMonitorTrait>>> {
        let monitors = self.factory.create_all();
        let mut result = Vec::new();

        for monitor in monitors {
            let monitor_type = monitor.name().await?;
            if let Some(m) = monitors::create_monitor(&monitor_type, self.iokit.clone()) {
                result.push(m);
            }
        }

        Ok(result)
    }

    /// Create a specific temperature monitor by type
    pub fn create_monitor(&self, monitor_type: &str) -> Result<Box<dyn TemperatureMonitorTrait>> {
        match monitors::create_monitor(monitor_type, self.iokit.clone()) {
            Some(monitor) => Ok(monitor),
            None => Err(Error::NotAvailable {
                resource: format!("Temperature monitor '{}'", monitor_type),
                reason: "Monitor type not available".to_string(),
            }),
        }
    }

    /// Create a new Temperature instance with a system reference
    pub fn new_with_system(system: &System) -> Result<Self> {
        let iokit = system.io_kit();
        let factory = TemperatureMonitorFactory::new(iokit.clone());
        Ok(Self {
            iokit,
            config: Default::default(),
            factory,
        })
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

    async fn is_throttling(&self) -> Result<bool> {
        // Check if any temperature monitor is reporting critical temperatures
        let cpu_temp = self.cpu_temperature().await?.unwrap_or(0.0);
        let gpu_temp = self.gpu_temperature().await?.unwrap_or(0.0);
        let battery_temp = self.battery_temperature().await?.unwrap_or(0.0);

        // Check if any temperature exceeds the throttling threshold
        Ok(cpu_temp > self.config.throttling_threshold
            || gpu_temp > self.config.throttling_threshold
            || battery_temp > self.config.throttling_threshold)
    }

    async fn get_fans(&self) -> Result<Vec<Fan>> {
        let monitors = self.get_all_monitors().await?;
        let mut fans = Vec::new();
        for monitor in monitors {
            if let Ok(fan) = monitor.fan().await {
                fans.push(fan);
            }
        }
        Ok(fans)
    }

    async fn get_thermal_metrics(&self) -> Result<ThermalMetrics> {
        let fans = self.get_fans().await?;
        let cpu_temp = self.cpu_temperature().await?;
        let gpu_temp = self.gpu_temperature().await?;
        let memory_temp = self.memory_temperature().await?;
        let battery_temp = self.battery_temperature().await?;
        let ambient_temp = self.ambient_temperature().await?;
        let is_throttling = self.is_throttling().await?;

        let thermal_level = if let Some(temp) = cpu_temp {
            if temp >= CPU_CRITICAL_TEMPERATURE {
                ThermalLevel::Critical
            } else if temp >= WARNING_TEMPERATURE_THRESHOLD {
                ThermalLevel::Warning
            } else {
                ThermalLevel::Normal
            }
        } else {
            ThermalLevel::Normal
        };

        Ok(ThermalMetrics {
            fan_speeds: fans.iter().map(|f| f.speed_rpm).collect(),
            thermal_level,
            memory_temperature: memory_temp,
            is_throttling,
            fans,
            cpu_temperature: cpu_temp,
            gpu_temperature: gpu_temp,
            battery_temperature: battery_temp,
            ssd_temperature: None,
            ambient_temperature: ambient_temp,
        })
    }
}

fn create_fan(name: String, speed: u32, min_speed: u32, max_speed: u32) -> Fan {
    Fan {
        name,
        speed_rpm: speed,
        min_speed,
        max_speed,
        target_speed: 0, // Default to 0 as we don't have target speed information
    }
}

pub fn create_thermal_metrics(
    fans: Vec<Fan>,
    cpu_temp: Option<f64>,
    gpu_temp: Option<f64>,
    memory_temp: Option<f64>,
    battery_temp: Option<f64>,
    ambient_temp: Option<f64>,
    ssd_temp: Option<f64>,
    is_throttling: bool,
) -> ThermalMetrics {
    let fans_info: Vec<Fan> = fans.into_iter().map(|f| f).collect();

    let fan_speeds: Vec<u32> = fans_info.iter().map(|f| f.speed_rpm).collect();

    // Determine thermal level based on temperatures
    let thermal_level = if let Some(cpu) = cpu_temp {
        if cpu >= CPU_CRITICAL_TEMPERATURE {
            ThermalLevel::Critical
        } else if cpu >= WARNING_TEMPERATURE_THRESHOLD {
            ThermalLevel::Warning
        } else {
            ThermalLevel::Normal
        }
    } else {
        ThermalLevel::Normal
    };

    ThermalMetrics {
        fan_speeds,
        thermal_level,
        memory_temperature: memory_temp,
        is_throttling,
        fans: fans_info,
        cpu_temperature: cpu_temp,
        gpu_temperature: gpu_temp,
        battery_temperature: battery_temp,
        ssd_temperature: ssd_temp,
        ambient_temperature: ambient_temp,
    }
}
