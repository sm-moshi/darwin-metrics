use super::*;
use crate::hardware::iokit::{FanInfo, GpuStats, ThermalInfo};
use crate::utils::test_utils::{create_test_dictionary, create_test_object};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSDictionary, NSObject, NSString};
use std::os::raw::c_char;

// Manual mock implementation of IOKit for testing
#[derive(Debug)]
struct MockIOKit {
    is_battery_present: bool,
}

impl IOKit for MockIOKit {
    fn io_service_matching(
        &self,
        _service_name: &str,
    ) -> Retained<NSDictionary<NSString, NSObject>> {
        create_test_dictionary()
    }

    fn io_service_get_matching_service(
        &self,
        _matching: &NSDictionary<NSString, NSObject>,
    ) -> Option<Retained<AnyObject>> {
        Some(create_test_object().into())
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
            BATTERY_TEMPERATURE => Some(3200),   // 32.00 degrees
            _ => None,
        }
    }

    fn get_bool_property(
        &self,
        _dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<bool> {
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
        Ok(create_test_object().into())
    }

    fn io_registry_entry_get_parent(&self, _entry: &AnyObject) -> Option<Retained<AnyObject>> {
        Some(create_test_object().into())
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
        Ok(FanInfo { speed_rpm: 1500, min_speed: 500, max_speed: 5000, percentage: 30.0 })
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
    let mock_iokit = MockIOKit { is_battery_present: is_present };

    Battery {
        is_present,
        is_charging: true,
        percentage: 75.0,
        time_remaining: Duration::from_secs(180 * 60),
        power_source: PowerSource::AC,
        cycle_count: 250,
        health_percentage: 90.909_090_909_090_92,
        temperature: 32.0,
        iokit: Box::new(mock_iokit),
    }
}

#[test]
fn test_battery_new() {
    // This test will use the default implementation We'll just verify that it doesn't panic
    let battery = Battery::default();
    assert!(!battery.is_present);
    assert_eq!(battery.percentage, 0.0);
}

#[test]
fn test_battery_update_present() {
    let mut battery = create_mock_battery(true);
    let result = battery.update();

    assert!(result.is_ok(), "Update should succeed");
    assert!(battery.is_present);
    assert!(battery.is_charging);
    assert_eq!(battery.percentage, 75.0);
    assert_eq!(battery.time_remaining, Duration::from_secs(180 * 60));
    assert_eq!(battery.power_source, PowerSource::AC);
    assert_eq!(battery.cycle_count, 250);
    // Use approximate comparison for floating point values
    assert!((battery.health_percentage - 90.909_090_909_090_92).abs() < 0.000_001);
    assert_eq!(battery.temperature, 32.0);
}

#[test]
fn test_battery_update_not_present() {
    let mut battery = create_mock_battery(false);
    let result = battery.update();

    assert!(result.is_ok(), "Update should succeed");
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
fn test_battery_get_info() {
    let mut battery = create_mock_battery(true);
    battery.update().unwrap();

    let info = battery.get_info().unwrap();
    assert!(info.is_present);
    assert_eq!(info.percentage, 75.0);
    assert_eq!(info.power_source, PowerSource::AC);
}

#[test]
fn test_battery_with_values() {
    let battery =
        Battery::with_values(true, true, 85.5, 120, PowerSource::Battery, 300, 95.0, 30.5);

    assert!(battery.is_present);
    assert!(battery.is_charging);
    assert_eq!(battery.percentage, 85.5);
    assert_eq!(battery.time_remaining, Duration::from_secs(120 * 60));
    assert_eq!(battery.power_source, PowerSource::Battery);
    assert_eq!(battery.cycle_count, 300);
    assert_eq!(battery.health_percentage, 95.0);
    assert_eq!(battery.temperature, 30.5);
}

#[test]
fn test_battery_is_critical() {
    let battery = Battery { percentage: 5.0, ..Battery::default() };

    assert!(battery.is_critical());

    let battery = Battery { percentage: 15.0, ..Battery::default() };

    assert!(!battery.is_critical());
}

#[test]
fn test_battery_is_low() {
    let battery = Battery { percentage: 15.0, ..Battery::default() };
    assert!(battery.is_low());

    let battery = Battery { percentage: 25.0, ..Battery::default() };
    assert!(!battery.is_low());
}

#[test]
fn test_battery_time_remaining_display() {
    let battery = Battery { time_remaining: Duration::from_secs(150 * 60), ..Battery::default() };

    // Test 2 hours and 30 minutes
    assert_eq!(battery.time_remaining_display(), "2h 30m");

    // Test 45 minutes
    let battery = Battery { time_remaining: Duration::from_secs(45 * 60), ..Battery::default() };
    assert_eq!(battery.time_remaining_display(), "45m");

    // Test 0 minutes
    let battery = Battery { time_remaining: Duration::from_secs(0), ..Battery::default() };
    assert_eq!(battery.time_remaining_display(), "0m");
}

#[test]
fn test_battery_is_health_poor() {
    let battery = Battery { health_percentage: 70.0, ..Battery::default() };
    assert!(battery.is_health_poor());

    let battery = Battery { health_percentage: 85.0, ..Battery::default() };
    assert!(!battery.is_health_poor());
}

#[test]
fn test_battery_has_high_cycle_count() {
    let battery = Battery { cycle_count: 1100, ..Battery::default() };
    assert!(battery.has_high_cycle_count());

    let battery = Battery { cycle_count: 500, ..Battery::default() };
    assert!(!battery.has_high_cycle_count());
}

#[test]
fn test_battery_power_source_display() {
    let battery = Battery { power_source: PowerSource::AC, ..Battery::default() };
    assert_eq!(battery.power_source_display(), "AC Power");

    let battery = Battery { power_source: PowerSource::Battery, ..Battery::default() };
    assert_eq!(battery.power_source_display(), "Battery");

    let battery = Battery { power_source: PowerSource::Unknown, ..Battery::default() };
    assert_eq!(battery.power_source_display(), "Unknown");
}

#[test]
fn test_battery_is_temperature_critical() {
    let battery = Battery { temperature: 45.0, ..Battery::default() };
    assert!(battery.is_temperature_critical());

    let battery = Battery { temperature: 35.0, ..Battery::default() };
    assert!(!battery.is_temperature_critical());
}

#[test]
fn test_battery_clone() {
    let mut original = create_mock_battery(true);
    original.update().unwrap();

    let cloned = original.clone();

    assert!(cloned.is_present == original.is_present);
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

    assert!(battery1 == battery2);

    // Modify one property
    battery2.percentage = 50.0;
    assert!(battery1 != battery2);
}
