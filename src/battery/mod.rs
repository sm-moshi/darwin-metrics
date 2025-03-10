use std::time::Duration;

use crate::{
    error::{Error, Result},
    hardware::iokit::{IOKit, IOKitImpl},
};

const BATTERY_IS_PRESENT: &str = "BatteryInstalled";
const BATTERY_IS_CHARGING: &str = "IsCharging";
const BATTERY_CURRENT_CAPACITY: &str = "CurrentCapacity";
const BATTERY_MAX_CAPACITY: &str = "MaxCapacity";
const BATTERY_DESIGN_CAPACITY: &str = "DesignCapacity";
const BATTERY_CYCLE_COUNT: &str = "CycleCount";
const BATTERY_TEMPERATURE: &str = "Temperature";
const BATTERY_TIME_REMAINING: &str = "TimeRemaining";
const BATTERY_POWER_SOURCE: &str = "ExternalConnected";

#[derive(Debug, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub enum PowerSource {
    Battery,
    AC,
    Unknown,
}

#[derive(Debug)]
pub struct Battery {
    pub is_present: bool,
    pub is_charging: bool,
    pub percentage: f64,
    pub time_remaining: Duration,
    pub power_source: PowerSource,
    pub cycle_count: u32,
    pub health_percentage: f64,
    pub temperature: f64,

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
    pub fn new() -> Result<Self> {
        let mut battery = Self::default();
        battery.update()?;
        Ok(battery)
    }

    pub fn update(&mut self) -> Result<()> {
        let matching = self.iokit.io_service_matching("AppleSmartBattery");
        let service = self.iokit.io_service_get_matching_service(&matching);

        let Some(service) = service else {
            return Err(Error::service_not_found("Battery service not found".to_string()));
        };

        let properties = self.iokit.io_registry_entry_create_cf_properties(&service)?;

        self.is_present =
            self.iokit.get_bool_property(&properties, BATTERY_IS_PRESENT).unwrap_or(false);

        if !self.is_present {
            self.is_charging = false;
            self.percentage = 0.0;
            self.time_remaining = Duration::from_secs(0);
            self.power_source = PowerSource::Unknown;
            self.cycle_count = 0;
            self.health_percentage = 0.0;
            self.temperature = 0.0;
            return Ok(());
        }

        self.is_charging =
            self.iokit.get_bool_property(&properties, BATTERY_IS_CHARGING).unwrap_or(false);
        let is_external =
            self.iokit.get_bool_property(&properties, BATTERY_POWER_SOURCE).unwrap_or(false);
        self.power_source = if is_external { PowerSource::AC } else { PowerSource::Battery };

        let current =
            self.iokit.get_number_property(&properties, BATTERY_CURRENT_CAPACITY).unwrap_or(0)
                as f64;
        let max =
            self.iokit.get_number_property(&properties, BATTERY_MAX_CAPACITY).unwrap_or(100) as f64;
        self.percentage = if max > 0.0 { (current / max * 100.0).clamp(0.0, 100.0) } else { 0.0 };

        let design = self
            .iokit
            .get_number_property(&properties, BATTERY_DESIGN_CAPACITY)
            .unwrap_or(max as i64) as f64;
        self.health_percentage =
            if design > 0.0 { (max / design * 100.0).clamp(0.0, 100.0) } else { 0.0 };

        self.cycle_count =
            self.iokit.get_number_property(&properties, BATTERY_CYCLE_COUNT).unwrap_or(0) as u32;

        let time = self.iokit.get_number_property(&properties, BATTERY_TIME_REMAINING).unwrap_or(0);
        self.time_remaining = Duration::from_secs((time.max(0) * 60) as u64);

        let temp =
            self.iokit.get_number_property(&properties, BATTERY_TEMPERATURE).unwrap_or(0) as f64;
        self.temperature = temp / 100.0;

        Ok(())
    }

    pub fn get_info(&self) -> Result<Self> {
        let matching = self.iokit.io_service_matching("AppleSmartBattery");
        let service = self.iokit.io_service_get_matching_service(&matching);

        let Some(service) = service else {
            return Err(Error::ServiceNotFound("Battery service not found".to_string()));
        };

        match self.iokit.io_registry_entry_create_cf_properties(&service) {
            Ok(_) => Ok(self.clone()),
            Err(e) => Err(e),
        }
    }

    #[allow(clippy::too_many_arguments)]
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
            iokit: Box::new(IOKitImpl),
        }
    }

    pub fn is_critical(&self) -> bool {
        self.percentage < 10.0
    }

    pub fn is_low(&self) -> bool {
        self.percentage < 20.0
    }

    pub fn time_remaining_display(&self) -> String {
        let minutes = self.time_remaining.as_secs() / 60;
        if minutes < 60 {
            format!("{minutes}m")
        } else {
            let hours = minutes / 60;
            let remaining_minutes = minutes % 60;
            format!("{hours}h {remaining_minutes}m")
        }
    }

    pub fn is_health_poor(&self) -> bool {
        self.health_percentage < 80.0
    }

    pub fn has_high_cycle_count(&self) -> bool {
        self.cycle_count > 1000
    }

    pub fn power_source_display(&self) -> &'static str {
        match self.power_source {
            PowerSource::Battery => "Battery",
            PowerSource::AC => "AC Power",
            PowerSource::Unknown => "Unknown",
        }
    }

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
            iokit: Box::new(IOKitImpl),
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
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2_foundation::{NSDictionary, NSObject, NSString};
    // No unused imports
    use std::os::raw::c_char;
    use crate::utils::test_utils::{create_test_dictionary, create_test_object};
    use crate::hardware::iokit::{FanInfo, GpuStats, ThermalInfo};

    // Manual mock implementation of IOKit for testing
    #[derive(Debug)]
    struct MockIOKit {
        is_battery_present: bool,
    }

    impl IOKit for MockIOKit {
        fn io_service_matching(&self, _service_name: &str) -> Retained<NSDictionary<NSString, NSObject>> {
            create_test_dictionary()
        }

        fn io_service_get_matching_service(
            &self,
            _matching: &NSDictionary<NSString, NSObject>,
        ) -> Option<Retained<AnyObject>> {
            Some(create_test_object())
        }

        fn io_registry_entry_create_cf_properties(
            &self,
            _entry: &AnyObject,
        ) -> Result<Retained<NSDictionary<NSString, NSObject>>> {
            Ok(create_test_dictionary())
        }

        fn io_object_release(&self, _obj: &AnyObject) {}

        fn get_string_property(
            &self,
            _dict: &NSDictionary<NSString, NSObject>,
            _key: &str,
        ) -> Option<String> {
            None
        }

        fn get_number_property(
            &self,
            _dict: &NSDictionary<NSString, NSObject>,
            key: &str,
        ) -> Option<i64> {
            match key {
                BATTERY_CURRENT_CAPACITY => Some(75),
                BATTERY_MAX_CAPACITY => Some(100),
                BATTERY_DESIGN_CAPACITY => Some(110),
                BATTERY_CYCLE_COUNT => Some(250),
                BATTERY_TIME_REMAINING => Some(180), // 180 minutes
                BATTERY_TEMPERATURE => Some(3200), // 32.00 degrees
                _ => None,
            }
        }

        fn get_bool_property(&self, _dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<bool> {
            match key {
                BATTERY_IS_PRESENT => Some(self.is_battery_present),
                BATTERY_IS_CHARGING => Some(true),
                BATTERY_POWER_SOURCE => Some(true),
                _ => None,
            }
        }

        fn get_dict_property(
            &self,
            _dict: &NSDictionary<NSString, NSObject>,
            _key: &str,
        ) -> Option<Retained<NSDictionary<NSString, NSObject>>> {
            Some(create_test_dictionary())
        }

        fn get_service(&self, _name: &str) -> Result<Retained<AnyObject>> {
            Ok(create_test_object())
        }

        fn io_registry_entry_get_parent(&self, _entry: &AnyObject) -> Option<Retained<AnyObject>> {
            Some(create_test_object())
        }

        // Temperature related methods
        fn get_cpu_temperature(&self) -> Result<f64> {
            Ok(45.0)
        }

        fn get_gpu_temperature(&self) -> Result<f64> {
            Ok(40.0)
        }

        fn get_gpu_stats(&self) -> Result<GpuStats> {
            Ok(GpuStats {
                utilization: 30.0,
                perf_cap: 100.0,
                perf_threshold: 0.0,
                memory_used: 1024,
                memory_total: 4096,
                name: String::from("Test GPU"),
            })
        }

        // Fan related methods
        fn get_fan_speed(&self) -> Result<u32> {
            Ok(1500)
        }

        fn get_fan_count(&self) -> Result<u32> {
            Ok(2)
        }

        fn get_fan_info(&self, _fan_index: u32) -> Result<FanInfo> {
            Ok(FanInfo {
                speed_rpm: 1500,
                min_speed: 500,
                max_speed: 5000,
                percentage: 30.0,
            })
        }

        fn get_all_fans(&self) -> Result<Vec<FanInfo>> {
            Ok(vec![self.get_fan_info(0)?, self.get_fan_info(1)?])
        }

        // Advanced thermal methods
        fn get_heatsink_temperature(&self) -> Result<f64> {
            Ok(35.0)
        }

        fn get_ambient_temperature(&self) -> Result<f64> {
            Ok(25.0)
        }

        fn get_battery_temperature(&self) -> Result<f64> {
            Ok(32.0)
        }

        fn get_cpu_power(&self) -> Result<f64> {
            Ok(15.0)
        }

        fn check_thermal_throttling(&self) -> Result<bool> {
            Ok(false)
        }

        fn get_thermal_info(&self) -> Result<ThermalInfo> {
            Ok(ThermalInfo {
                cpu_temp: 45.0,
                gpu_temp: 40.0,
                heatsink_temp: Some(35.0),
                ambient_temp: Some(25.0),
                battery_temp: Some(32.0),
                is_throttling: false,
                cpu_power: Some(15.0),
            })
        }

        fn read_smc_key(&self, _key: [c_char; 4]) -> Result<f64> {
            Ok(42.0)
        }
    }

    // Helper function to create a battery with mock IOKit
    fn create_mock_battery(is_present: bool) -> Battery {
        let mock_iokit = MockIOKit {
            is_battery_present: is_present,
        };
        
        Battery {
            is_present,
            is_charging: true,
            percentage: 75.0,
            time_remaining: Duration::from_secs(180 * 60),
            power_source: PowerSource::AC,
            cycle_count: 250,
            health_percentage: 90.90909090909092,
            temperature: 32.0,
            iokit: Box::new(mock_iokit),
        }
    }

    #[test]
    fn test_battery_new() {
        // This test will use the default implementation
        // We'll just verify that it doesn't panic
        let battery = Battery::default();
        assert_eq!(battery.is_present, false);
        assert_eq!(battery.percentage, 0.0);
    }

    #[test]
    fn test_battery_update_present() {
        let mut battery = create_mock_battery(true);
        let result = battery.update();
        
        assert!(result.is_ok(), "Update should succeed");
        assert_eq!(battery.is_present, true);
        assert_eq!(battery.is_charging, true);
        assert_eq!(battery.percentage, 75.0);
        assert_eq!(battery.time_remaining, Duration::from_secs(180 * 60));
        assert_eq!(battery.power_source, PowerSource::AC);
        assert_eq!(battery.cycle_count, 250);
        // Use approximate comparison for floating point values
        assert!((battery.health_percentage - 90.90909090909092).abs() < 0.000001);
        assert_eq!(battery.temperature, 32.0);
    }

    #[test]
    fn test_battery_update_not_present() {
        let mut battery = create_mock_battery(false);
        let result = battery.update();
        
        assert!(result.is_ok(), "Update should succeed");
        assert_eq!(battery.is_present, false);
        assert_eq!(battery.is_charging, false);
        assert_eq!(battery.percentage, 0.0);
        assert_eq!(battery.time_remaining, Duration::from_secs(0));
        assert_eq!(battery.power_source, PowerSource::Unknown);
        assert_eq!(battery.cycle_count, 0);
        assert_eq!(battery.health_percentage, 0.0);
        assert_eq!(battery.temperature, 0.0);
    }

    #[test]
    fn test_battery_get_info() {
        let mut battery = create_mock_battery(true);
        battery.update().unwrap();
        
        let info = battery.get_info().unwrap();
        assert_eq!(info.is_present, true);
        assert_eq!(info.percentage, 75.0);
        assert_eq!(info.power_source, PowerSource::AC);
    }

    #[test]
    fn test_battery_with_values() {
        let battery = Battery::with_values(
            true, 
            true, 
            85.5, 
            120, 
            PowerSource::Battery, 
            300, 
            95.0, 
            30.5
        );
        
        assert_eq!(battery.is_present, true);
        assert_eq!(battery.is_charging, true);
        assert_eq!(battery.percentage, 85.5);
        assert_eq!(battery.time_remaining, Duration::from_secs(120 * 60));
        assert_eq!(battery.power_source, PowerSource::Battery);
        assert_eq!(battery.cycle_count, 300);
        assert_eq!(battery.health_percentage, 95.0);
        assert_eq!(battery.temperature, 30.5);
    }

    #[test]
    fn test_battery_is_critical() {
        let mut battery = Battery::default();
        battery.percentage = 5.0;
        assert!(battery.is_critical());
        
        battery.percentage = 10.0;
        assert!(!battery.is_critical());
    }

    #[test]
    fn test_battery_is_low() {
        let mut battery = Battery::default();
        battery.percentage = 15.0;
        assert!(battery.is_low());
        
        battery.percentage = 25.0;
        assert!(!battery.is_low());
    }

    #[test]
    fn test_battery_time_remaining_display() {
        let mut battery = Battery::default();
        
        // Test 2 hours and 30 minutes
        battery.time_remaining = Duration::from_secs(150 * 60);
        assert_eq!(battery.time_remaining_display(), "2h 30m");
        
        // Test 45 minutes
        battery.time_remaining = Duration::from_secs(45 * 60);
        assert_eq!(battery.time_remaining_display(), "45m");
        
        // Test 0 minutes
        battery.time_remaining = Duration::from_secs(0);
        assert_eq!(battery.time_remaining_display(), "0m");
    }

    #[test]
    fn test_battery_is_health_poor() {
        let mut battery = Battery::default();
        
        battery.health_percentage = 70.0;
        assert!(battery.is_health_poor());
        
        battery.health_percentage = 85.0;
        assert!(!battery.is_health_poor());
    }

    #[test]
    fn test_battery_has_high_cycle_count() {
        let mut battery = Battery::default();
        
        battery.cycle_count = 1100;
        assert!(battery.has_high_cycle_count());
        
        battery.cycle_count = 500;
        assert!(!battery.has_high_cycle_count());
    }

    #[test]
    fn test_battery_power_source_display() {
        let mut battery = Battery::default();
        
        battery.power_source = PowerSource::AC;
        assert_eq!(battery.power_source_display(), "AC Power");
        
        battery.power_source = PowerSource::Battery;
        assert_eq!(battery.power_source_display(), "Battery");
        
        battery.power_source = PowerSource::Unknown;
        assert_eq!(battery.power_source_display(), "Unknown");
    }

    #[test]
    fn test_battery_is_temperature_critical() {
        let mut battery = Battery::default();
        
        battery.temperature = 45.0;
        assert!(battery.is_temperature_critical());
        
        battery.temperature = 35.0;
        assert!(!battery.is_temperature_critical());
    }

    #[test]
    fn test_battery_clone() {
        let mut original = create_mock_battery(true);
        original.update().unwrap();
        
        let cloned = original.clone();
        
        assert_eq!(cloned.is_present, original.is_present);
        assert_eq!(cloned.percentage, original.percentage);
        assert_eq!(cloned.power_source, original.power_source);
        assert_eq!(cloned.cycle_count, original.cycle_count);
        assert_eq!(cloned.health_percentage, original.health_percentage);
        assert_eq!(cloned.temperature, original.temperature);
    }

    #[test]
    fn test_battery_eq() {
        let mut battery1 = create_mock_battery(true);
        battery1.update().unwrap();
        
        let mut battery2 = create_mock_battery(true);
        battery2.update().unwrap();
        
        assert_eq!(battery1, battery2);
        
        // Modify one property
        battery2.percentage = 50.0;
        assert_ne!(battery1, battery2);
    }
}