/// The IOKit service name for the battery
pub const BATTERY_SERVICE: &str = "AppleSmartBattery";

/// Battery property keys
pub mod keys {
    pub const BATTERY_PRESENT: &str = "BatteryInstalled";
    pub const BATTERY_POWER_SOURCE: &str = "ExternalConnected";
    pub const BATTERY_CHARGING: &str = "IsCharging";
    pub const BATTERY_CYCLE_COUNT: &str = "CycleCount";
    pub const BATTERY_TEMPERATURE: &str = "Temperature";
    pub const BATTERY_TIME_REMAINING: &str = "TimeRemaining";
    pub const BATTERY_CURRENT_CAPACITY: &str = "CurrentCapacity";
    pub const BATTERY_MAX_CAPACITY: &str = "MaxCapacity";
    pub const BATTERY_DESIGN_CAPACITY: &str = "DesignCapacity";
}
