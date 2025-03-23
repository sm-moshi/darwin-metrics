//! # Battery Module
//!
//! The battery module provides functionality for monitoring and collecting metrics
//! from macOS system batteries. It includes support for tracking battery health,
//! power consumption, temperature, and other battery-related metrics.
//!
//! ## Features
//!
//! * Battery health monitoring
//! * Power consumption tracking
//! * Temperature monitoring
//! * Charge cycle counting
//! * Battery status information
//!
//! ## Example
//!
//! ```rust
//! use darwin_metrics::battery::Battery;
//! use darwin_metrics::System;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let system = System::new()?;
//!     let battery = Battery::new(system.io_kit())?;
//!     
//!     let health = battery.health().await?;
//!     println!("Battery Health: {}%", health.percentage());
//!     
//!     Ok(())
//! }
//! ```

use std::sync::Arc;

use crate::{
    error::Result,
    hardware::iokit::IOKit,
};

// Import modules
pub mod constants;
/// Battery type definitions including BatteryPower, BatteryCapacity, and BatteryInfo
pub mod types;

// Re-export monitor structs for easier access
pub use monitors::{BatteryCapacityMonitor, BatteryHealthMonitor, BatteryPowerMonitor, BatteryTemperatureMonitor};

/// Configuration for battery monitoring
#[derive(Debug, Clone)]
pub struct BatteryConfig {
    /// Threshold for battery temperature (in Celsius)
    pub temperature_threshold: f64,
    /// Threshold for battery percentage (0-100)
    pub percentage_threshold: f64,
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            temperature_threshold: 45.0,
            percentage_threshold: 20.0,
        }
    }
}

/// Main battery struct for managing battery state
pub struct Battery {
    iokit: Arc<dyn IOKit>,
    device_id: String,
    config: BatteryConfig,
}

impl Battery {
    /// Creates a new battery monitor
    pub fn new(iokit: Arc<dyn IOKit>) -> Result<Self> {
        Ok(Self {
            iokit,
            device_id: "main".to_string(),
            config: BatteryConfig::default(),
        })
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
/// Battery monitoring implementations for capacity, health, power, and temperature
pub mod monitors;
