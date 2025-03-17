use darwin_metrics::hardware::iokit::{
    FanInfo, GpuStats, IOKit, IOKitImpl, MockIOKit, ThermalInfo
};
use darwin_metrics::error::Result;
use std::collections::HashMap;
use std::sync::Mutex;

use darwin_metrics::{
    error::{Error, Result},
    hardware::iokit::{FanInfo, GpuStats, IOKit, MockIOKit, ThermalInfo},
    utils::core::dictionary::SafeDictionary,
};

/// Builder for creating test IOKit instances
pub struct TestIOKitBuilder {
    physical_cores: usize,
    logical_cores: usize,
    core_usage: Vec<f64>,
    temperature: f64,
    battery_temp: Option<f64>,
    thermal_info: Option<ThermalInfo>,
    gpu_stats: Option<GpuStats>,
    fans: Vec<FanInfo>,
}

impl Default for TestIOKitBuilder {
    fn default() -> Self {
        Self {
            physical_cores: 4,
            logical_cores: 8,
            core_usage: vec![10.0, 20.0, 30.0, 40.0],
            temperature: 42.5,
            battery_temp: Some(35.0),
            thermal_info: None,
            gpu_stats: None,
            fans: Vec::new(),
        }
    }
}

impl TestIOKitBuilder {
    /// Create a new TestIOKitBuilder with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of physical cores
    pub fn with_physical_cores(mut self, cores: usize) -> Self {
        self.physical_cores = cores;
        self
    }

    /// Set the number of logical cores
    pub fn with_logical_cores(mut self, cores: usize) -> Self {
        self.logical_cores = cores;
        self
    }

    /// Set the core usage values
    pub fn with_core_usage(mut self, usage: Vec<f64>) -> Self {
        self.core_usage = usage;
        self
    }

    /// Set the CPU temperature
    pub fn with_temperature(mut self, temp: f64) -> Self {
        self.temperature = temp;
        self
    }

    /// Set the battery temperature
    pub fn with_battery_temp(mut self, temp: Option<f64>) -> Self {
        self.battery_temp = temp;
        self
    }

    /// Set the thermal info
    pub fn with_thermal_info(mut self, info: ThermalInfo) -> Self {
        self.thermal_info = Some(info);
        self
    }

    /// Set the GPU stats
    pub fn with_gpu_stats(mut self, stats: GpuStats) -> Self {
        self.gpu_stats = Some(stats);
        self
    }

    /// Add a fan to the list of fans
    pub fn with_fan(mut self, fan: FanInfo) -> Self {
        self.fans.push(fan);
        self
    }

    /// Build a MockIOKit instance with the configured values
    pub fn build(self) -> Result<MockIOKit> {
        let mut mock = MockIOKit::new()?;
        
        mock.set_physical_cores(self.physical_cores);
        mock.set_logical_cores(self.logical_cores);
        mock.set_core_usage(self.core_usage)?;
        mock.set_temperature(self.temperature);
        
        if let Some(temp) = self.battery_temp {
            mock.set_battery_temperature(Some(temp));
        }
        
        if let Some(info) = self.thermal_info {
            mock.set_thermal_info(info);
        }
        
        Ok(mock)
    }
} 