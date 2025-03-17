mod constants;
pub mod monitors;
pub mod types;

use crate::{
    core::metrics::hardware::{BatteryCapacityMonitorTrait, TemperatureMonitor},
    error::Result,
    hardware::{
        battery::monitors::{
            BatteryCapacityMonitor, BatteryHealthMonitor, BatteryPowerMonitor, BatteryTemperatureMonitor,
        },
        iokit::IOKit,
    },
};
use constants::*;

use objc2::class;
use objc2::runtime::AnyClass;
use std::sync::Once;

static INIT: Once = Once::new();

fn ensure_classes_registered() {
    INIT.call_once(|| unsafe {
        let _: &AnyClass = class!(NSObject);
        let _: &AnyClass = class!(NSMutableDictionary);
        let _: &AnyClass = class!(NSNumber);
        let _: &AnyClass = class!(NSString);
    });
}

/// A battery monitoring interface for macOS systems
pub struct Battery {
    iokit: Box<dyn IOKit>,
}

impl Battery {
    /// Create a new battery monitor using the system's IOKit interface
    pub fn new_system() -> Result<Self> {
        ensure_classes_registered();
        Ok(Self { iokit: Box::new(crate::hardware::iokit::IOKitImpl::new()?) })
    }

    /// Create a new battery monitor with a custom IOKit implementation
    pub fn new(iokit: Box<dyn IOKit>) -> Result<Self> {
        ensure_classes_registered();
        Ok(Self { iokit })
    }

    /// Check if a battery is present in the system
    pub fn is_present(&self) -> bool {
        self.iokit.get_battery_info().ok().and_then(|info| info.get_bool(keys::BATTERY_PRESENT)).unwrap_or(false)
    }

    /// Get a monitor for battery power state
    pub fn power_monitor(&self) -> Result<BatteryPowerMonitor> {
        Ok(BatteryPowerMonitor::new(self.iokit.clone_box()))
    }

    /// Get a monitor for battery health
    pub fn health_monitor(&self) -> Result<BatteryHealthMonitor> {
        Ok(BatteryHealthMonitor::new(self.iokit.clone_box()))
    }

    /// Get a monitor for battery capacity metrics
    pub fn capacity_monitor(&self) -> Result<BatteryCapacityMonitor> {
        Ok(BatteryCapacityMonitor::new("battery0".to_string()))
    }

    /// Get a monitor for battery temperature metrics
    pub fn temperature_monitor(&self) -> Result<BatteryTemperatureMonitor> {
        Ok(BatteryTemperatureMonitor::new(self.iokit.clone_box()))
    }

    // Convenience methods that delegate to monitors

    /// Get the current battery percentage
    pub async fn percentage(&self) -> Result<f64> {
        let current = self.capacity_monitor()?.current_capacity().await? as f64;
        let max = self.capacity_monitor()?.maximum_capacity().await? as f64;
        Ok((current / max) * 100.0)
    }

    /// Get the battery cycle count
    pub async fn cycle_count(&self) -> Result<i64> {
        let count = self.capacity_monitor()?.cycle_count().await? as i64;
        Ok(count)
    }

    /// Get the battery temperature in Celsius
    pub async fn temperature(&self) -> Result<f64> {
        let temp = self.temperature_monitor()?.temperature().await?;
        Ok(temp)
    }

    /// Get the current battery capacity
    pub async fn current_capacity(&self) -> Result<f64> {
        let capacity = self.capacity_monitor()?.current_capacity().await?;
        Ok(capacity as f64)
    }

    /// Get the design capacity of the battery
    pub async fn design_capacity(&self) -> Result<f64> {
        let capacity = self.capacity_monitor()?.design_capacity().await?;
        Ok(capacity as f64)
    }

    /// Check if the battery is currently charging
    pub async fn is_charging(&self) -> Result<bool> {
        self.power_monitor()?.is_charging().await
    }

    /// Get the estimated time remaining until the battery is empty/full
    pub async fn time_remaining(&self) -> Result<Option<std::time::Duration>> {
        let power_monitor = self.power_monitor()?;
        let minutes = power_monitor.time_remaining().await?;

        if minutes <= 0 {
            Ok(None)
        } else {
            Ok(Some(std::time::Duration::from_secs(minutes as u64 * 60)))
        }
    }
}

// Re-export monitor structs for easier access
pub use monitors::{
    BatteryCapacityMonitor, BatteryHealthMonitor, BatteryPowerMonitor, BatteryTemperatureMonitor
};
