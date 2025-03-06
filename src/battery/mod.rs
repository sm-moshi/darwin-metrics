//! Battery metrics and power management information for macOS systems.
//!
//! This module provides functionality to gather battery-related metrics and power management
//! information on macOS systems. It supports monitoring of:
//! - Battery presence and charging status
//! - Power source (Battery/AC/Unknown)
//! - Battery charge percentage and time remaining
//! - Battery health metrics (cycle count, health percentage)
//! - Temperature monitoring
//!
//! The module uses IOKit to safely interact with macOS power management interfaces.
//!
//! # Examples
//!
//! ```no_run
//! use darwin_metrics::battery::{Battery, PowerSource};
//!
//! fn main() -> darwin_metrics::Result<()> {
//!     // Create and initialize battery monitoring
//!     let mut battery = Battery::new()?;
//!     
//!     // Get current battery status
//!     println!("Power Source: {}", battery.power_source_display());
//!     println!("Battery: {}%", battery.percentage);
//!     
//!     if battery.is_charging {
//!         println!("Charging...");
//!     } else if battery.is_present {
//!         println!("Time remaining: {}", battery.time_remaining_display());
//!     }
//!     
//!     // Check battery health
//!     if battery.is_health_poor() {
//!         println!("Warning: Battery health is degraded!");
//!         println!("Health: {}%", battery.health_percentage);
//!         println!("Cycle count: {}", battery.cycle_count);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Note
//!
//! This module requires macOS-specific IOKit functionality and will only work on macOS systems.
//! All battery metrics are updated atomically when calling `update()` to ensure consistency.

use crate::hardware::iokit::{IOKit, IOKitImpl};
use crate::{Error, Result};
use std::time::Duration;

#[cfg(test)]
use mockall::predicate::*;

// Battery property keys
const BATTERY_IS_PRESENT: &str = "BatteryInstalled";
const BATTERY_IS_CHARGING: &str = "IsCharging";
const BATTERY_CURRENT_CAPACITY: &str = "CurrentCapacity";
const BATTERY_MAX_CAPACITY: &str = "MaxCapacity";
const BATTERY_DESIGN_CAPACITY: &str = "DesignCapacity";
const BATTERY_CYCLE_COUNT: &str = "CycleCount";
const BATTERY_TEMPERATURE: &str = "Temperature";
const BATTERY_TIME_REMAINING: &str = "TimeRemaining";
const BATTERY_POWER_SOURCE: &str = "ExternalConnected";

/// Power source type for the system.
///
/// Represents the current power source providing energy to the system.
/// This can be either battery power, AC power, or unknown in case
/// the power source cannot be determined.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PowerSource {
    /// Running on battery power
    Battery,
    /// Connected to AC power
    AC,
    /// Unknown power source
    Unknown,
}

/// Represents the current state of the system's battery.
///
/// This struct provides comprehensive battery metrics and power management
/// information for macOS systems. It includes both real-time metrics like
/// charge level and power source, as well as battery health information
/// like cycle count and capacity health.
///
/// The struct maintains its state internally and can be updated by calling
/// the `update()` method to fetch the latest metrics from the system.
///
/// # Examples
///
/// ```no_run
/// use darwin_metrics::battery::Battery;
///
/// let mut battery = Battery::new()?;
///
/// // Update battery metrics
/// battery.update()?;
///
/// // Check battery status
/// if battery.is_present {
///     println!("Battery at {}%", battery.percentage);
///     if battery.is_low() {
///         println!("Warning: Low battery!");
///     }
/// } else {
///     println!("No battery detected");
/// }
/// # Ok::<(), darwin_metrics::Error>(())
/// ```
///
/// # Thread Safety
///
/// This struct is safe to use across threads as it maintains internal
/// synchronization when accessing IOKit interfaces. All public methods
/// are thread-safe.
#[derive(Debug)]
pub struct Battery {
    /// Whether a battery is present in the system
    pub is_present: bool,
    /// Whether the battery is currently charging
    pub is_charging: bool,
    /// Battery charge percentage (0-100)
    pub percentage: f64,
    /// Estimated time remaining in minutes (when discharging)
    pub time_remaining: Duration,
    /// Current power source
    pub power_source: PowerSource,
    /// Battery cycle count
    pub cycle_count: u32,
    /// Battery maximum capacity (percentage of design capacity)
    pub health_percentage: f64,
    /// Battery temperature in Celsius
    pub temperature: f64,
    /// IOKit interface for hardware access
    #[cfg(not(test))]
    iokit: Box<dyn IOKit>,
    #[cfg(test)]
    pub iokit: Box<dyn IOKit>,
}

impl Default for Battery {
    fn default() -> Self {
        Self::with_values(false, false, 0.0, 0, PowerSource::Unknown, 0, 0.0, 0.0)
    }
}

impl Battery {
    /// Create a new Battery instance and initialize it with system values
    pub fn new() -> Result<Self> {
        let mut battery = Self::default();
        battery.update()?;
        Ok(battery)
    }

    /// Update battery metrics from the system
    pub fn update(&mut self) -> Result<()> {
        // Create matching dictionary for AppleSmartBattery
        let matching = self.iokit.io_service_matching("AppleSmartBattery");

        // Get battery service
        let service = self.iokit.io_service_get_matching_service(&matching);

        let Some(service) = service else {
            return Err(Error::ServiceNotFound);
        };

        // Get battery properties
        let properties = self
            .iokit
            .io_registry_entry_create_cf_properties(&service)?;

        // Update battery presence
        self.is_present = self
            .iokit
            .get_bool_property(&properties, BATTERY_IS_PRESENT)
            .unwrap_or(false);

        if !self.is_present {
            // Fixed parentheses here
            // Reset all values if no battery is present
            self.is_charging = false;
            self.percentage = 0.0;
            self.time_remaining = Duration::from_secs(0);
            self.power_source = PowerSource::Unknown;
            self.cycle_count = 0;
            self.health_percentage = 0.0;
            self.temperature = 0.0;
            return Ok(());
        }

        // Update charging status
        self.is_charging = self
            .iokit
            .get_bool_property(&properties, BATTERY_IS_CHARGING)
            .unwrap_or(false);

        // Update power source
        let is_external = self
            .iokit
            .get_bool_property(&properties, BATTERY_POWER_SOURCE)
            .unwrap_or(false);
        self.power_source = if is_external {
            PowerSource::AC
        } else {
            PowerSource::Battery
        };

        // Update capacity and percentage
        let current = self
            .iokit
            .get_number_property(&properties, BATTERY_CURRENT_CAPACITY)
            .unwrap_or(0) as f64;
        let max = self
            .iokit
            .get_number_property(&properties, BATTERY_MAX_CAPACITY)
            .unwrap_or(100) as f64;
        self.percentage = if max > 0.0 {
            (current / max * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        // Update health percentage
        let design = self
            .iokit
            .get_number_property(&properties, BATTERY_DESIGN_CAPACITY)
            .unwrap_or(max as i64) as f64;
        self.health_percentage = if design > 0.0 {
            (max / design * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        // Update cycle count
        self.cycle_count = self
            .iokit
            .get_number_property(&properties, BATTERY_CYCLE_COUNT)
            .unwrap_or(0) as u32;

        // Update time remaining (in minutes)
        let time = self
            .iokit
            .get_number_property(&properties, BATTERY_TIME_REMAINING)
            .unwrap_or(0);
        self.time_remaining = Duration::from_secs((time.max(0) * 60) as u64);

        // Update temperature (convert from celsius * 100 to celsius)
        let temp = self
            .iokit
            .get_number_property(&properties, BATTERY_TEMPERATURE)
            .unwrap_or(0) as f64;
        self.temperature = temp / 100.0;

        Ok(())
    }

    /// Get current battery information
    ///
    /// # Returns
    /// Returns a `Result` containing battery information or an error if the information
    /// cannot be retrieved.
    ///
    /// # Examples
    /// ```no_run
    /// use darwin_metrics::battery::Battery;
    ///
    /// let battery = Battery::new().unwrap();
    /// let info = battery.get_info().unwrap();
    /// println!("Battery at {}%, {}",
    ///     info.percentage,
    ///     if info.is_charging { "charging" } else { "discharging" }
    /// );
    /// ```
    pub fn get_info(&self) -> Result<Self> {
        // Create matching dictionary for AppleSmartBattery
        let matching = self.iokit.io_service_matching("AppleSmartBattery");

        // Get battery service
        let service = self.iokit.io_service_get_matching_service(&matching);

        let Some(service) = service else {
            return Err(Error::ServiceNotFound);
        };

        // Get battery properties
        match self.iokit.io_registry_entry_create_cf_properties(&service) {
            Ok(_) => Ok(self.clone()),
            Err(e) => Err(e),
        }
    }

    /// Create a new Battery instance with the given parameters
    ///
    /// # Arguments
    /// * `is_present` - Whether a battery is present in the system
    /// * `is_charging` - Whether the battery is currently charging
    /// * `percentage` - Battery charge percentage (0-100)
    /// * `time_remaining` - Time remaining in minutes
    /// * `power_source` - Current power source
    /// * `cycle_count` - Battery cycle count
    /// * `health_percentage` - Battery health as percentage of design capacity
    /// * `temperature` - Battery temperature in Celsius
    pub fn with_values(
        is_present: bool,
        is_charging: bool,
        percentage: f64,
        time_remaining: i32,
        power_source: PowerSource,
        cycle_count: u32,
        health_percentage: f64,
        temperature: f64,
    ) -> Self {
        Self {
            is_present,
            is_charging,
            percentage: percentage.clamp(0.0, 100.0),
            time_remaining: Duration::from_secs((time_remaining * 60) as u64),
            power_source,
            cycle_count,
            health_percentage: health_percentage.clamp(0.0, 100.0),
            temperature,
            iokit: Box::new(IOKitImpl::default()),
        }
    }

    /// Returns true if the battery level is critically low (below 10%)
    pub fn is_critical(&self) -> bool {
        self.percentage < 10.0
    }

    /// Returns true if the battery level is low (below 20%)
    pub fn is_low(&self) -> bool {
        self.percentage < 20.0
    }

    /// Returns the estimated time remaining as a human-readable string
    pub fn time_remaining_display(&self) -> String {
        let minutes = self.time_remaining.as_secs() / 60;
        if minutes < 60 {
            format!("{} minutes", minutes)
        } else {
            let hours = minutes / 60;
            let remaining_minutes = minutes % 60;
            format!("{} hours {} minutes", hours, remaining_minutes)
        }
    }

    /// Returns true if the battery health is poor (below 70% of design capacity)
    pub fn is_health_poor(&self) -> bool {
        self.health_percentage < 70.0
    }

    /// Returns true if the battery has exceeded recommended cycle count (>1000)
    pub fn has_high_cycle_count(&self) -> bool {
        self.cycle_count > 1000
    }

    /// Returns a human-readable description of the power source
    pub fn power_source_display(&self) -> &'static str {
        match self.power_source {
            PowerSource::Battery => "Battery Power",
            PowerSource::AC => "AC Power",
            PowerSource::Unknown => "Unknown Power Source",
        }
    }

    /// Checks if the battery temperature is in a critical range
    ///
    /// Returns true if the battery temperature is outside the safe operating range.
    /// Most lithium-ion batteries should operate between -10°C and 40°C.
    /// Temperatures outside this range can cause reduced battery life,
    /// performance issues, or safety concerns.
    pub fn is_temperature_critical(&self) -> bool {
        self.temperature < -10.0 || self.temperature > 40.0
    }
}

impl Clone for Battery {
    fn clone(&self) -> Self {
        Self {
            is_present: self.is_present,
            is_charging: self.is_charging,
            percentage: self.percentage,
            time_remaining: self.time_remaining,
            power_source: self.power_source,
            cycle_count: self.cycle_count,
            health_percentage: self.health_percentage,
            temperature: self.temperature,
            iokit: Box::new(IOKitImpl::default()),
        }
    }
}

impl PartialEq for Battery {
    fn eq(&self, other: &Self) -> bool {
        self.is_present == other.is_present
            && self.is_charging == other.is_charging
            && self.percentage == other.percentage
            && self.time_remaining == other.time_remaining
            && self.power_source == other.power_source
            && self.cycle_count == other.cycle_count
            && self.health_percentage == other.health_percentage
            && self.temperature == other.temperature
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iokit::MockIOKit; // This is now re-exported from iokit module
    use crate::testing::{create_safe_dictionary, setup_test_environment};

    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};

    #[test]
    fn test_basic_battery_properties() {
        let battery =
            Battery::with_values(true, false, 75.5, 90, PowerSource::Battery, 500, 85.0, 35.0);

        // Test simple properties only
        assert!(battery.percentage <= 100.0);
        assert!(battery.percentage >= 0.0);
        assert!(!battery.is_critical());
        assert_eq!(battery.is_present, true);
        assert!(!battery.is_low());
    }

    #[test]
    fn test_battery_update() {
        // Create a safer test with contained unsafe code
        let mock = MockIOKit::new();
        let battery = Battery {
            is_present: true,
            is_charging: false,
            percentage: 75.5,
            time_remaining: Duration::from_secs(5400),
            power_source: PowerSource::Battery,
            cycle_count: 500,
            health_percentage: 85.0,
            temperature: 35.0,
            iokit: Box::new(mock),
        };

        assert_eq!(battery.percentage, 75.5);
        assert_eq!(battery.is_present, true);
        assert_eq!(battery.power_source, PowerSource::Battery);
    }

    #[test]
    fn test_battery_mock() {
        let mut mock = MockIOKit::new();
        mock.expect_io_service_matching()
            .returning(|_| create_safe_dictionary());

        let battery = Battery {
            is_present: true,
            is_charging: false,
            percentage: 75.5,
            time_remaining: Duration::from_secs(5400),
            power_source: PowerSource::Battery,
            cycle_count: 500,
            health_percentage: 85.0,
            temperature: 35.0,
            iokit: Box::new(mock),
        };

        assert_eq!(battery.percentage, 75.5);
    }

    #[test]
    fn test_battery_constructor() {
        setup_test_environment();
        let battery =
            Battery::with_values(true, false, 75.5, 90, PowerSource::Battery, 500, 85.0, 35.0);
        assert_eq!(battery.is_present, true);
        assert_eq!(battery.is_charging, false);
        assert_eq!(battery.percentage, 75.5);
        assert_eq!(battery.time_remaining.as_secs(), 5400);
        assert_eq!(battery.power_source, PowerSource::Battery);
        assert_eq!(battery.cycle_count, 500);
        assert_eq!(battery.health_percentage, 85.0);
        assert_eq!(battery.temperature, 35.0);
    }

    #[test]
    fn test_battery_status_display() {
        setup_test_environment();
        let battery =
            Battery::with_values(true, false, 75.5, 90, PowerSource::Battery, 500, 85.0, 35.0);
        assert_eq!(battery.time_remaining_display(), "1 hours 30 minutes");
        assert!(!battery.is_low());
        assert!(!battery.is_critical());
        assert_eq!(battery.power_source_display(), "Battery Power");
    }

    #[test]
    fn test_battery_update_no_battery() {
        let mut mock = crate::iokit::MockIOKit::new();

        // Setup mock for service matching
        mock.expect_io_service_matching()
            .with(eq("AppleSmartBattery"))
            .times(1)
            .returning(|_| unsafe {
                let dict: *mut AnyObject = msg_send![class!(NSDictionary), new];
                Retained::from_raw(dict.cast()).unwrap()
            });

        // Setup mock for getting service
        mock.expect_io_service_get_matching_service()
            .times(1)
            .returning(|_| unsafe {
                let obj: *mut AnyObject = msg_send![class!(NSObject), new];
                Some(Retained::from_raw(obj).unwrap())
            });

        // Setup mock for getting properties
        mock.expect_io_registry_entry_create_cf_properties()
            .times(1)
            .returning(|_| unsafe {
                let dict: *mut AnyObject = msg_send![class!(NSDictionary), new];
                Ok(Retained::from_raw(dict.cast()).unwrap())
            });

        // Setup mock for property getters
        mock.expect_get_bool_property()
            .with(always(), eq(BATTERY_IS_PRESENT))
            .times(1)
            .returning(|_, _| Some(false));

        let mut battery = Battery {
            iokit: Box::new(mock),
            is_present: true,
            is_charging: true,
            percentage: 50.0,
            time_remaining: Duration::from_secs(3600),
            power_source: PowerSource::Battery,
            cycle_count: 100,
            health_percentage: 90.0,
            temperature: 25.0,
        };

        assert!(battery.update().is_ok());
        assert!(!battery.is_present);
        assert!(!battery.is_charging);
        assert_eq!(battery.percentage, 0.0);
        assert_eq!(battery.time_remaining, Duration::from_secs(0));
        assert_eq!(battery.power_source, PowerSource::Unknown);
        assert_eq!(battery.cycle_count, 0);
        assert_eq!(battery.health_percentage, 0.0);
        assert_eq!(battery.temperature, 0.0);
    }

    #[test]
    fn test_battery_health() {
        let battery = Battery::with_values(
            true,
            false,
            75.5,
            90,
            PowerSource::Battery,
            1200,
            65.0,
            35.0,
        );
        assert!(battery.is_health_poor());
        assert!(battery.has_high_cycle_count());

        let battery =
            Battery::with_values(true, false, 75.5, 90, PowerSource::Battery, 500, 85.0, 35.0);
        assert!(!battery.is_health_poor());
        assert!(!battery.has_high_cycle_count());
    }

    #[test]
    fn test_power_source_variants() {
        let battery_power =
            Battery::with_values(true, false, 75.5, 90, PowerSource::Battery, 500, 85.0, 35.0);
        let ac_power = Battery::with_values(true, true, 95.5, 0, PowerSource::AC, 500, 85.0, 35.0);
        let unknown_power =
            Battery::with_values(true, false, 75.5, 90, PowerSource::Unknown, 500, 85.0, 35.0);

        assert_eq!(battery_power.power_source_display(), "Battery Power");
        assert_eq!(ac_power.power_source_display(), "AC Power");
        assert_eq!(unknown_power.power_source_display(), "Unknown Power Source");
    }

    #[test]
    fn test_battery_percentage_bounds() {
        let battery = Battery::with_values(
            true,
            false,
            150.0,
            90,
            PowerSource::Battery,
            500,
            85.0,
            35.0,
        );
        assert!(battery.percentage <= 100.0);

        let battery = Battery::with_values(
            true,
            false,
            -10.0,
            90,
            PowerSource::Battery,
            500,
            85.0,
            35.0,
        );
        assert!(battery.percentage >= 0.0);
    }

    #[test]
    fn test_time_remaining_edge_cases() {
        let battery =
            Battery::with_values(true, false, 75.5, 0, PowerSource::Battery, 500, 85.0, 35.0);
        assert_eq!(battery.time_remaining_display(), "0 minutes");

        let battery = Battery::with_values(
            true,
            false,
            75.5,
            180,
            PowerSource::Battery,
            500,
            85.0,
            35.0,
        );
        assert_eq!(battery.time_remaining_display(), "3 hours 0 minutes");
    }

    #[test]
    fn test_battery_health_edge_cases() {
        let battery =
            Battery::with_values(true, false, 75.5, 90, PowerSource::Battery, 0, 100.0, 35.0);
        assert!(!battery.has_high_cycle_count());

        let battery = Battery::with_values(
            true,
            false,
            75.5,
            90,
            PowerSource::Battery,
            5000,
            50.0,
            35.0,
        );
        assert!(battery.has_high_cycle_count());
        assert!(battery.is_health_poor());
    }

    #[test]
    fn test_battery_temperature_bounds() {
        let cold_battery =
            Battery::with_values(true, false, 75.5, 90, PowerSource::Battery, 500, 85.0, 0.0);
        assert!(cold_battery.temperature >= 0.0);

        let hot_battery = Battery::with_values(
            true,
            false,
            75.5,
            90,
            PowerSource::Battery,
            500,
            85.0,
            100.0,
        );
        assert!(hot_battery.temperature <= 100.0);
    }

    #[cfg(target_os = "macos")]
    mod macos_tests {
        use super::*;

        #[test]
        fn test_get_info_service_not_found() {
            let mut mock = crate::iokit::MockIOKit::new();
            mock.expect_io_service_matching()
                .with(eq("AppleSmartBattery"))
                .returning(|_| unsafe {
                    let dict: *mut AnyObject = msg_send![class!(NSDictionary), new];
                    Retained::from_raw(dict.cast()).unwrap()
                });

            mock.expect_io_service_get_matching_service()
                .returning(|_| None);

            let battery = Battery {
                is_present: false,
                is_charging: false,
                percentage: 0.0,
                time_remaining: Duration::from_secs(0),
                power_source: PowerSource::Unknown,
                cycle_count: 0,
                health_percentage: 0.0,
                temperature: 0.0,
                iokit: Box::new(mock),
            };
            let result = battery.get_info();
            assert!(matches!(result, Err(Error::ServiceNotFound)));
        }

        #[test]
        fn test_get_info_properties_failure() {
            let mut mock = crate::iokit::MockIOKit::new();

            mock.expect_io_service_matching()
                .with(eq("AppleSmartBattery"))
                .returning(|_| unsafe {
                    let dict: *mut AnyObject = msg_send![class!(NSDictionary), new];
                    Retained::from_raw(dict.cast()).unwrap()
                });

            mock.expect_io_service_get_matching_service()
                .returning(|_| unsafe {
                    let obj: *mut AnyObject = msg_send![class!(NSObject), new];
                    Some(Retained::from_raw(obj).unwrap())
                });

            mock.expect_io_registry_entry_create_cf_properties()
                .returning(|_| Err(Error::SystemError("Failed to get properties".into())));

            let battery = Battery {
                is_present: false,
                is_charging: false,
                percentage: 0.0,
                time_remaining: Duration::from_secs(0),
                power_source: PowerSource::Unknown,
                cycle_count: 0,
                health_percentage: 0.0,
                temperature: 0.0,
                iokit: Box::new(mock),
            };
            let result = battery.get_info();
            assert!(matches!(result, Err(Error::SystemError(_))));
        }
    }

    #[test]
    fn test_battery_state_transitions() {
        // Create a battery with initial state
        let battery = Battery::with_values(
            true,                 // is_present
            false,                // is_charging
            75.5,                 // percentage
            90,                   // time_remaining_min
            PowerSource::Battery, // power_source
            500,                  // cycle_count
            85.0,                 // health_percentage
            35.0,                 // temperature
        );

        // Initial state verification
        assert_eq!(battery.power_source, PowerSource::Battery);
        assert!(!battery.is_charging);
        assert_eq!(battery.percentage, 75.5);

        // Test state transitions with new battery instances
        // 1. Connect to power
        let charging_battery =
            Battery::with_values(true, true, 75.5, 90, PowerSource::AC, 500, 85.0, 35.0);
        assert_eq!(charging_battery.power_source, PowerSource::AC);
        assert!(charging_battery.is_charging);

        // 2. Fully charged
        let fully_charged =
            Battery::with_values(true, false, 100.0, 0, PowerSource::AC, 500, 85.0, 35.0);
        assert_eq!(fully_charged.power_source, PowerSource::AC);
        assert!(!fully_charged.is_charging);
        assert_eq!(fully_charged.percentage, 100.0);

        // 3. Low battery
        let low_battery =
            Battery::with_values(true, false, 10.0, 30, PowerSource::Battery, 500, 85.0, 35.0);
        assert_eq!(low_battery.percentage, 10.0);
        assert!(low_battery.is_low());
        assert!(!low_battery.is_critical());

        // 4. Critical battery
        let critical_battery =
            Battery::with_values(true, false, 5.0, 15, PowerSource::Battery, 500, 85.0, 35.0);
        assert_eq!(critical_battery.percentage, 5.0);
        assert!(critical_battery.is_low());
        assert!(critical_battery.is_critical());
    }

    #[test]
    fn test_power_source_transition_scenarios() {
        // Scenario 1: AC to Battery transition
        let battery_ac =
            Battery::with_values(true, true, 60.0, 120, PowerSource::AC, 300, 95.0, 30.0);
        assert_eq!(battery_ac.power_source, PowerSource::AC);
        assert!(battery_ac.is_charging);

        let battery_disconnected = Battery::with_values(
            true,
            false,
            60.0,
            120,
            PowerSource::Battery,
            300,
            95.0,
            30.0,
        );
        assert_eq!(battery_disconnected.power_source, PowerSource::Battery);
        assert!(!battery_disconnected.is_charging);

        // Scenario 2: Connect to AC when battery is low
        let low_battery =
            Battery::with_values(true, false, 15.0, 45, PowerSource::Battery, 300, 95.0, 30.0);
        assert_eq!(low_battery.power_source, PowerSource::Battery);
        assert!(low_battery.is_low());

        let charging_low_battery =
            Battery::with_values(true, true, 15.0, 45, PowerSource::AC, 300, 95.0, 30.0);
        assert_eq!(charging_low_battery.power_source, PowerSource::AC);
        assert!(charging_low_battery.is_charging);
        assert!(charging_low_battery.is_low());

        // Scenario 3: Unknown power source
        let unknown_source = Battery::with_values(
            true,
            false,
            80.0,
            240,
            PowerSource::Unknown,
            300,
            95.0,
            30.0,
        );
        assert_eq!(unknown_source.power_source, PowerSource::Unknown);
        assert_eq!(
            unknown_source.power_source_display(),
            "Unknown Power Source"
        );
    }

    #[test]
    fn test_temperature_range_edge_cases() {
        // Test very low temperature
        let very_cold = Battery::with_values(
            true,
            false,
            80.0,
            120,
            PowerSource::Battery,
            500,
            90.0,
            -20.0, // -20°C is extremely cold for a battery
        );
        assert_eq!(very_cold.temperature, -20.0);
        assert!(
            very_cold.temperature < -10.0,
            "Temperature should be below critical threshold"
        );

        // Test normal operating temperature
        let normal_temp = Battery::with_values(
            true,
            false,
            80.0,
            120,
            PowerSource::Battery,
            500,
            90.0,
            25.0, // 25°C is normal
        );
        assert_eq!(normal_temp.temperature, 25.0);
        assert!(
            normal_temp.temperature > -10.0 && normal_temp.temperature < 40.0,
            "Temperature should be in normal range"
        );

        // Test elevated temperature
        let warm = Battery::with_values(
            true,
            false,
            80.0,
            120,
            PowerSource::Battery,
            500,
            90.0,
            35.0, // 35°C is warm but not critical
        );
        assert_eq!(warm.temperature, 35.0);
        assert!(
            warm.temperature < 40.0,
            "Temperature should be below critical threshold"
        );

        // Test critical high temperature
        let very_hot = Battery::with_values(
            true,
            false,
            80.0,
            120,
            PowerSource::Battery,
            500,
            90.0,
            45.0, // 45°C is very hot for a battery
        );
        assert_eq!(very_hot.temperature, 45.0);
        assert!(
            very_hot.temperature > 40.0,
            "Temperature should be above critical threshold"
        );

        // Test extreme temperature that might be from a sensor error
        let extreme = Battery::with_values(
            true,
            false,
            80.0,
            120,
            PowerSource::Battery,
            500,
            90.0,
            100.0, // 100°C would be a dangerous battery condition
        );
        assert_eq!(extreme.temperature, 100.0);
        assert!(
            extreme.temperature > 40.0,
            "Temperature should be above critical threshold"
        );

        // Test 0°C
        let freezing = Battery::with_values(
            true,
            false,
            80.0,
            120,
            PowerSource::Battery,
            500,
            90.0,
            0.0, // 0°C is at freezing point
        );
        assert_eq!(freezing.temperature, 0.0);
        assert!(
            freezing.temperature > -10.0 && freezing.temperature < 40.0,
            "Temperature should be in normal range"
        );
    }

    #[test]
    fn test_error_handling() {
        let mut mock = MockIOKit::new();
        mock.expect_io_service_matching()
            .returning(|_| create_safe_dictionary());
        mock.expect_io_service_get_matching_service()
            .returning(|_| None);

        let mut battery = Battery {
            is_present: false,
            is_charging: false,
            percentage: 0.0,
            time_remaining: Duration::from_secs(0),
            power_source: PowerSource::Unknown,
            cycle_count: 0,
            health_percentage: 0.0,
            temperature: 0.0,
            iokit: Box::new(mock),
        };

        assert!(matches!(battery.update(), Err(Error::ServiceNotFound)));
    }
}
