use std::time::Duration;

use crate::core::types::Percentage;

/// Represents the current power source for the system
#[derive(Debug, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub enum PowerSource {
    /// Running on battery power
    Battery,
    /// Running on AC power
    AC,
    /// Power source could not be determined
    Unknown,
}

/// Information about the battery's current state
#[derive(Debug, Clone)]
pub struct BatteryInfo {
    /// Whether a battery is present
    pub present: bool,
    /// Current battery charge percentage (0-100)
    pub percentage: i64,
    /// Number of charge cycles
    pub cycle_count: i64,
    /// Whether the battery is currently charging
    pub is_charging: bool,
    /// Whether external power is connected
    pub is_external: bool,
    /// Battery temperature in Celsius
    pub temperature: f64,
    /// Current power draw in watts
    pub power_draw: f64,
    /// Design capacity in mAh
    pub design_capacity: i64,
    /// Current maximum capacity in mAh
    pub current_capacity: i64,
    /// Estimated time remaining until empty/full
    pub time_remaining: Option<Duration>,
}

/// Represents battery power information
#[derive(Debug, Clone)]
pub struct BatteryPower {
    /// Power consumption in watts
    pub watts: f64,
}

impl BatteryPower {
    pub fn new(watts: f64) -> Self {
        Self { watts }
    }
}

/// Represents battery capacity information
#[derive(Debug, Clone)]
pub struct BatteryCapacity {
    /// Current capacity percentage (0-100)
    pub current: f64,
    /// Maximum capacity percentage (0-100)
    pub maximum: f64,
    /// Design capacity percentage (0-100)
    pub design: f64,
    /// Battery cycle count
    pub cycle_count: u32,
}

impl BatteryCapacity {
    pub fn new(current: f64, maximum: f64, design: f64, cycle_count: u32) -> Self {
        Self {
            current,
            maximum,
            design,
            cycle_count,
        }
    }

    /// Get the health percentage (current max capacity / design capacity)
    pub fn health_percentage(&self) -> f64 {
        (self.maximum / self.design) * 100.0
    }

    /// Get the current charge percentage
    pub fn charge_percentage(&self) -> Percentage {
        Percentage::from_f64(self.current)
    }
}
