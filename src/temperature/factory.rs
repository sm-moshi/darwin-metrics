use std::sync::Arc;

use crate::error::Result;
use crate::hardware::iokit::IOKit;
use crate::temperature::monitors::*;
use crate::traits::TemperatureMonitor;

/// Temperature Monitor Factory
///
/// The factory pattern for creating different types of temperature monitors.
#[derive(Debug, Clone)]
pub struct TemperatureMonitorFactory {
    /// IOKit service for accessing hardware information
    io_kit: Arc<Box<dyn IOKit>>,
}

impl TemperatureMonitorFactory {
    /// Create a new temperature monitor factory
    pub fn new(io_kit: Arc<Box<dyn IOKit>>) -> Self {
        Self { io_kit }
    }

    /// Create a temperature monitor of the specified type
    pub fn create(&self, monitor_type: &str) -> Result<Box<dyn TemperatureMonitor>> {
        match monitor_type {
            "cpu" => Ok(Box::new(CpuTemperatureMonitor::new(self.io_kit.clone()))),
            "gpu" => Ok(Box::new(GpuTemperatureMonitor::new(self.io_kit.clone()))),
            "ambient" => Ok(Box::new(AmbientTemperatureMonitor::new(self.io_kit.clone()))),
            "battery" => Ok(Box::new(BatteryTemperatureMonitor::new(self.io_kit.clone()))),
            "ssd" => Ok(Box::new(SsdTemperatureMonitor::new(self.io_kit.clone()))),
            _ => Err(crate::error::Error::InvalidMonitorType(monitor_type.to_string())),
        }
    }

    /// Create all available temperature monitors
    pub fn create_all(&self) -> Vec<Box<dyn TemperatureMonitor>> {
        vec![
            Box::new(CpuTemperatureMonitor::new(self.io_kit.clone())),
            Box::new(GpuTemperatureMonitor::new(self.io_kit.clone())),
            Box::new(AmbientTemperatureMonitor::new(self.io_kit.clone())),
            Box::new(BatteryTemperatureMonitor::new(self.io_kit.clone())),
            Box::new(SsdTemperatureMonitor::new(self.io_kit.clone())),
        ]
    }
}
