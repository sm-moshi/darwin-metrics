use std::sync::Arc;

use crate::error::Result;
use crate::hardware::iokit::IOKit;

// Import modules
pub mod constants;
pub mod types;

// Re-export monitor structs for easier access
pub use monitors::{BatteryCapacityMonitor, BatteryHealthMonitor, BatteryPowerMonitor, BatteryTemperatureMonitor};

/// Main battery struct for managing battery state
pub struct Battery {
    iokit: Arc<dyn IOKit>,
    device_id: String,
}

impl Battery {
    /// Creates a new Battery instance with the provided IOKit implementation
    pub fn new(iokit: Arc<dyn IOKit>) -> Self {
        Self {
            iokit,
            device_id: "main".to_string(),
        }
    }

    /// Gets the IOKit instance used by this battery
    pub fn iokit(&self) -> &Arc<dyn IOKit> {
        &self.iokit
    }

    /// Gets the device ID for this battery
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// Creates a battery capacity monitor for this battery
    pub fn capacity_monitor(&self) -> BatteryCapacityMonitor {
        BatteryCapacityMonitor::new(self.device_id.clone())
    }

    /// Creates a battery health monitor for this battery
    pub fn health_monitor(&self) -> BatteryHealthMonitor {
        BatteryHealthMonitor::new(self.device_id.clone())
    }

    /// Creates a battery power monitor for this battery
    pub fn power_monitor(&self) -> BatteryPowerMonitor {
        BatteryPowerMonitor::new(self.device_id.clone())
    }

    /// Creates a battery temperature monitor for this battery
    pub fn temperature_monitor(&self) -> BatteryTemperatureMonitor {
        BatteryTemperatureMonitor::new(self.device_id.clone())
    }
}

// Include the monitors module directly
pub mod monitors;
