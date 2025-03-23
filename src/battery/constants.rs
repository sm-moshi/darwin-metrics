//! # Battery Constants
//!
//! This module defines constants used for battery monitoring and management.
//! These constants are primarily used for interacting with the macOS IOKit
//! framework to retrieve battery information.
//!
//! ## Constants
//!
//! The module provides key strings for accessing various battery properties
//! through the IOKit registry.

/// The IOKit service name for the battery
pub const BATTERY_SERVICE: &str = "AppleSmartBattery";

/// Battery property keys
pub mod keys {
    /// Key string for checking if a battery is installed
    pub const BATTERY_PRESENT: &str = "BatteryInstalled";

    /// Key string for checking if external power is connected
    pub const BATTERY_POWER_SOURCE: &str = "ExternalConnected";

    /// Key string for checking if the battery is currently charging
    pub const BATTERY_CHARGING: &str = "IsCharging";

    /// Key string for retrieving the battery's cycle count
    pub const BATTERY_CYCLE_COUNT: &str = "CycleCount";

    /// Key string for retrieving the battery's current temperature
    pub const BATTERY_TEMPERATURE: &str = "Temperature";

    /// Key string for retrieving the estimated time remaining on battery power
    pub const BATTERY_TIME_REMAINING: &str = "TimeRemaining";

    /// Key string for retrieving the battery's current capacity
    pub const BATTERY_CURRENT_CAPACITY: &str = "CurrentCapacity";

    /// Key string for retrieving the battery's maximum capacity
    pub const BATTERY_MAX_CAPACITY: &str = "MaxCapacity";

    /// Key string for retrieving the battery's design capacity
    pub const BATTERY_DESIGN_CAPACITY: &str = "DesignCapacity";
}
