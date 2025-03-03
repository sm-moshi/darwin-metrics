use crate::iokit::{IOKit, IOKitImpl};
use crate::{Error, Result};
use core_foundation::dictionary::{CFDictionaryGetValue, CFDictionaryRef};
use core_foundation::number::{CFBooleanGetValue, CFNumberGetValue, CFNumberRef, kCFNumberSInt64Type};
use std::ffi::{CString, c_void};
use std::time::Duration;
use scopeguard;
use core_foundation::boolean::kCFBooleanTrue;
use core_foundation::dictionary::{CFDictionaryAddValue, CFDictionaryCreateMutable};
use mockall::predicate::eq;

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

/// Power source type for the system
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PowerSource {
    /// Running on battery power
    Battery,
    /// Connected to AC power
    AC,
    /// Unknown power source
    Unknown,
}

/// Represents the current state of the system's battery
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
    iokit: Box<dyn IOKit>,
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
        if matching.is_null() {
            return Err(Error::SystemError(
                "Failed to create matching dictionary".into(),
            ));
        }

        // Get battery service
        let service = self.iokit.io_service_get_matching_service(matching);

        // Release the matching dictionary since IOKit takes ownership
        self.iokit.cf_release(matching as *mut _);

        if service == 0 {
            return Err(Error::ServiceNotFound);
        }

        // Get battery properties
        let properties = self.iokit.io_registry_entry_create_cf_properties(service)?;
        
        // Create a clone of self.iokit for the guard to avoid borrow checker issues
        let iokit = self.iokit.as_ref();
        let _guard = scopeguard::guard(service, move |s| iokit.io_object_release(s));
        let _props_guard = scopeguard::guard(properties as *mut _, move |p| iokit.cf_release(p));

        // Update battery presence
        self.is_present = Self::get_bool_value(properties, BATTERY_IS_PRESENT)?
            .unwrap_or(false);

        if !self.is_present {
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
        self.is_charging = Self::get_bool_value(properties, BATTERY_IS_CHARGING)?
            .unwrap_or(false);

        // Update power source
        let is_external = Self::get_bool_value(properties, BATTERY_POWER_SOURCE)?
            .unwrap_or(false);
        self.power_source = if is_external {
            PowerSource::AC
        } else {
            PowerSource::Battery
        };

        // Update capacity and percentage
        let current = Self::get_number_value(properties, BATTERY_CURRENT_CAPACITY)?
            .unwrap_or(0) as f64;
        let max = Self::get_number_value(properties, BATTERY_MAX_CAPACITY)?
            .unwrap_or(100) as f64;
        self.percentage = if max > 0.0 {
            (current / max * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        // Update health percentage
        let design = Self::get_number_value(properties, BATTERY_DESIGN_CAPACITY)?
            .unwrap_or(max as i64) as f64;
        self.health_percentage = if design > 0.0 {
            (max / design * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        // Update cycle count
        self.cycle_count = Self::get_number_value(properties, BATTERY_CYCLE_COUNT)?
            .unwrap_or(0) as u32;

        // Update time remaining (in minutes)
        let time = Self::get_number_value(properties, BATTERY_TIME_REMAINING)?
            .unwrap_or(0);
        self.time_remaining = Duration::from_secs((time.max(0) * 60) as u64);

        // Update temperature (convert from celsius * 100 to celsius)
        let temp = Self::get_number_value(properties, BATTERY_TEMPERATURE)?
            .unwrap_or(0) as f64;
        self.temperature = temp / 100.0;

        Ok(())
    }

    /// Get a boolean value from the battery properties dictionary
    fn get_bool_value(dict: CFDictionaryRef, key: &str) -> Result<Option<bool>> {
        unsafe {
            let key_str = CString::new(key)
                .map_err(|e| Error::SystemError(format!("Invalid key {}: {}", key, e)))?;
            let key_ptr = key_str.as_ptr() as *const c_void;
            let value = CFDictionaryGetValue(dict as *const _, key_ptr);
            if value.is_null() {
                return Ok(None);
            }

            Ok(Some(CFBooleanGetValue(value as _)))
        }
    }

    /// Get a number value from the battery properties dictionary
    fn get_number_value(dict: CFDictionaryRef, key: &str) -> Result<Option<i64>> {
        unsafe {
            let key_str = CString::new(key)
                .map_err(|e| Error::SystemError(format!("Invalid key {}: {}", key, e)))?;
            let key_ptr = key_str.as_ptr() as *const c_void;
            let value = CFDictionaryGetValue(dict as *const _, key_ptr);
            if value.is_null() {
                return Ok(None);
            }

            let mut result: i64 = 0;
            if CFNumberGetValue(
                value as CFNumberRef,
                kCFNumberSInt64Type,
                &mut result as *mut _ as *mut _,
            ) {
                Ok(Some(result))
            } else {
                Ok(None)
            }
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
        if matching.is_null() {
            return Err(Error::SystemError(
                "Failed to create matching dictionary".into(),
            ));
        }

        // Get battery service
        let service = self.iokit.io_service_get_matching_service(matching);

        // Release the matching dictionary since IOKit takes ownership
        self.iokit.cf_release(matching as *mut _);

        if service == 0 {
            return Err(Error::ServiceNotFound);
        }

        // Get battery properties
        match self.iokit.io_registry_entry_create_cf_properties(service) {
            Ok(_properties) => {
                // Release service since we have the properties
                self.iokit.io_object_release(service);

                // Return success for now, actual property parsing will be implemented later
                Ok(self.clone())
            }
            Err(e) => {
                // Release service on error
                self.iokit.io_object_release(service);
                Err(e)
            }
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
    use crate::iokit::MockIOKit;
    use std::ptr;
    use mockall::predicate::eq;

    #[test]
    fn test_battery_update() {
        let mut mock = MockIOKit::new();

        mock.expect_io_service_matching()
            .with(eq("AppleSmartBattery"))
            .returning(|_| ptr::null());

        mock.expect_io_service_get_matching_service()
            .returning(|_| 0);

        mock.expect_cf_release()
            .returning(|_| ());

        let mut battery = Battery::default();
        battery.iokit = Box::new(mock);

        let result = battery.update();
        assert!(matches!(result, Err(Error::ServiceNotFound)));
    }

    #[test]
    fn test_battery_update_with_service() {
        let mut mock = MockIOKit::new();

        mock.expect_io_service_matching()
            .with(eq("AppleSmartBattery"))
            .returning(|_| 0x1234 as *const _);

        mock.expect_io_service_get_matching_service()
            .returning(|_| 1234);

        mock.expect_cf_release()
            .times(2)
            .returning(|_| ());

        mock.expect_io_object_release()
            .returning(|_| ());

        mock.expect_io_registry_entry_create_cf_properties()
            .returning(|_| Ok(ptr::null_mut()));

        let mut battery = Battery::default();
        battery.iokit = Box::new(mock);

        let result = battery.update();
        assert!(result.is_ok());
    }

    #[test]
    fn test_battery_constructor() {
        let battery = Battery::with_values(
            true, false, 75.5, 90,
            PowerSource::Battery, 500, 85.0, 35.0
        );
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
        let battery = Battery::with_values(
            true, false, 75.5, 90,
            PowerSource::AC, 500, 85.0, 35.0
        );
        assert_eq!(battery.time_remaining_display(), "1 hours 30 minutes");
        assert!(!battery.is_low());
        assert!(!battery.is_critical());
        assert_eq!(battery.power_source_display(), "AC Power");
    }

    #[test]
    fn test_battery_health() {
        let battery = Battery::with_values(
            true, false, 75.5, 90,
            PowerSource::Battery, 1200, 65.0, 35.0
        );
        assert!(battery.is_health_poor());
        assert!(battery.has_high_cycle_count());

        let battery = Battery::with_values(
            true, false, 75.5, 90,
            PowerSource::Battery, 500, 85.0, 35.0
        );
        assert!(!battery.is_health_poor());
        assert!(!battery.has_high_cycle_count());
    }

    #[test]
    fn test_power_source_variants() {
        let battery_power = Battery::with_values(
            true, false, 75.5, 90,
            PowerSource::Battery, 500, 85.0, 35.0
        );
        let ac_power = Battery::with_values(
            true, true, 95.5, 0,
            PowerSource::AC, 500, 85.0, 35.0
        );
        let unknown_power = Battery::with_values(
            true, false, 75.5, 90,
            PowerSource::Unknown, 500, 85.0, 35.0
        );

        assert_eq!(battery_power.power_source_display(), "Battery Power");
        assert_eq!(ac_power.power_source_display(), "AC Power");
        assert_eq!(unknown_power.power_source_display(), "Unknown Power Source");
    }

    #[test]
    fn test_battery_percentage_bounds() {
        let battery = Battery::with_values(
            true, false, 150.0, 90,
            PowerSource::Battery, 500, 85.0, 35.0
        );
        assert!(battery.percentage <= 100.0);

        let battery = Battery::with_values(
            true, false, -10.0, 90,
            PowerSource::Battery, 500, 85.0, 35.0
        );
        assert!(battery.percentage >= 0.0);
    }

    #[test]
    fn test_time_remaining_edge_cases() {
        let battery = Battery::with_values(
            true, false, 75.5, 0,
            PowerSource::Battery, 500, 85.0, 35.0
        );
        assert_eq!(battery.time_remaining_display(), "0 minutes");

        let battery = Battery::with_values(
            true, false, 75.5, 180,
            PowerSource::Battery, 500, 85.0, 35.0
        );
        assert_eq!(battery.time_remaining_display(), "3 hours 0 minutes");
    }

    #[test]
    fn test_battery_health_edge_cases() {
        let battery = Battery::with_values(
            true, false, 75.5, 90,
            PowerSource::Battery, 0, 100.0, 35.0
        );
        assert!(!battery.has_high_cycle_count());

        let battery = Battery::with_values(
            true, false, 75.5, 90,
            PowerSource::Battery, 5000, 50.0, 35.0
        );
        assert!(battery.has_high_cycle_count());
        assert!(battery.is_health_poor());
    }

    #[test]
    fn test_battery_temperature_bounds() {
        let cold_battery = Battery::with_values(
            true, false, 75.5, 90,
            PowerSource::Battery, 500, 85.0, 0.0
        );
        assert!(cold_battery.temperature >= 0.0);

        let hot_battery = Battery::with_values(
            true, false, 75.5, 90,
            PowerSource::Battery, 500, 85.0, 100.0
        );
        assert!(hot_battery.temperature <= 100.0);
    }

    #[cfg(target_os = "macos")]
    mod macos_tests {
        use super::*;

        #[test]
        fn test_get_info_service_not_found() {
            let mut mock = MockIOKit::new();
            mock.expect_io_service_matching()
                .with(eq("AppleSmartBattery"))
                .returning(|_| 0x1234 as *const _);
            mock.expect_io_service_get_matching_service()
                .returning(|_| 0);
            mock.expect_cf_release()
                .returning(|_| ());

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
            let mut mock = MockIOKit::new();
            mock.expect_io_service_matching()
                .with(eq("AppleSmartBattery"))
                .returning(|_| ptr::null());
            mock.expect_io_service_get_matching_service()
                .returning(|_| 123);
            mock.expect_io_registry_entry_create_cf_properties()
                .returning(|_| Err(Error::SystemError("Failed to get properties".into())));
            mock.expect_io_object_release()
                .returning(|_| ());
            mock.expect_cf_release()
                .returning(|_| ());

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
}
