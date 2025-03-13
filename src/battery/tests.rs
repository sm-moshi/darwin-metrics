#[cfg(test)]
use super::*;
use crate::hardware::iokit::{FanInfo, GpuStats, ThermalInfo};
use crate::utils::test_utils::{create_test_dictionary, create_test_object};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSDictionary, NSObject, NSString};
use std::os::raw::c_char;
use std::sync::Once;

static INIT: Once = Once::new();

fn setup_test() {
    // Ensure IOKit is initialized only once
    INIT.call_once(|| {
        // Any global IOKit initialization if needed
    });
}

// Manual mock implementation of IOKit for testing
#[derive(Debug, Clone)]
struct MockIOKit {
    is_battery_present: bool,
}

impl Default for MockIOKit {
    fn default() -> Self {
        Self {
            is_battery_present: false,
        }
    }
}

impl IOKit for MockIOKit {
    fn io_service_matching(&self, _name: &str) -> Result<ThreadSafeNSDictionary> {
        Ok(ThreadSafeNSDictionary::empty())
    }

    fn io_service_get_matching_service(&self, _matching: &ThreadSafeNSDictionary) -> Result<ThreadSafeNSDictionary> {
        Ok(ThreadSafeNSDictionary::empty())
    }

    fn io_registry_entry_create_cf_properties(&self, _service: &ThreadSafeNSDictionary) -> Result<ThreadSafeNSDictionary> {
        let dict = NSDictionary::new();
        Ok(ThreadSafeNSDictionary::new(dict))
    }

    fn get_gpu_stats(&self) -> Result<GpuStats> {
        Ok(GpuStats {
            utilization: 30.0,
            memory_used: 1024,
            memory_total: 4096,
            perf_cap: 100.0,
            perf_threshold: 0.0,
            name: String::from("Test GPU"),
        })
    }

    fn get_fan_info(&self, _fan_index: u32) -> Result<FanInfo> {
        Ok(FanInfo {
            speed_rpm: 1500.0,
            min_speed: 500.0,
            max_speed: 5000.0,
            percentage: 30.0,
        })
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo> {
        Ok(ThermalInfo {
            cpu_temp: 45.0,
            gpu_temp: 40.0,
            fan_speed: vec![1500.0, 1600.0],
            heatsink_temp: 35.0,
            ambient_temp: 25.0,
            battery_temp: 32.0,
            is_throttling: false,
            cpu_power: 15.0,
        })
    }

    fn get_cpu_temperature(&self) -> Result<f64> {
        Ok(45.0)
    }

    fn io_connect_call_method(
        &self,
        _connection: u32,
        _selector: u32,
        _input: &[u8],
        _input_cnt: u32,
        _output: &mut [u8],
        _output_cnt: &mut u32,
    ) -> Result<()> {
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
    }
}

#[test]
fn test_battery_new() -> Result<()> {
    setup_test();
    let battery = Battery::new(Box::new(MockIOKit::default()))?;
    assert!(battery.is_present()?);
    Ok(())
}

#[test]
fn test_battery_update_present() -> Result<()> {
    setup_test();
    let mut battery = Battery::new(Box::new(MockIOKit::default()))?;
    assert!(battery.is_present()?);
    battery.update()?;
    assert!(battery.is_present()?);
    Ok(())
}

#[test]
fn test_battery_with_values() -> Result<()> {
    setup_test();
    let battery = Battery::with_values(
        true,
        false,
        100,
        85.0,
        35.0,
        Duration::from_secs(3600),
        45.0,
        100.0,
        85.0,
    )?;

    assert!(battery.is_present()?);
    assert!(!battery.is_charging()?);
    assert_eq!(battery.cycle_count()?, 100);
    assert_eq!(battery.percentage()?, 85);
    assert_eq!(battery.temperature()?, 35.0);
    assert_eq!(battery.power_draw()?, 45.0);
    assert_eq!(battery.design_capacity()?, 100);
    assert_eq!(battery.current_capacity()?, 85);

    Ok(())
}

#[test]
fn test_battery_get_info() -> Result<()> {
    setup_test();
    let battery = Battery::with_values(
        true,
        false,
        100,
        85.0,
        35.0,
        Duration::from_secs(3600),
        45.0,
        100.0,
        85.0,
    )?;

    let info = battery.get_info()?;
    assert!(info.present);
    assert_eq!(info.percentage, 85);
    assert_eq!(info.cycle_count, 100);
    assert!(!info.is_charging);
    assert_eq!(info.temperature, 35.0);
    assert_eq!(info.power_draw, 45.0);

    Ok(())
}

#[test]
fn test_battery_is_critically_low() -> Result<()> {
    setup_test();
    let battery = Battery::with_values(
        true,
        false,
        100,
        4.0,
        35.0,
        Duration::from_secs(3600),
        45.0,
        100.0,
        85.0,
    )?;

    assert!(battery.is_critically_low()?);

    let battery = Battery::with_values(
        true,
        false,
        100,
        85.0,
        35.0,
        Duration::from_secs(3600),
        45.0,
        100.0,
        85.0,
    )?;

    assert!(!battery.is_critically_low()?);

    Ok(())
}

#[test]
fn test_battery_is_low() -> Result<()> {
    setup_test();
    let battery = Battery::with_values(
        true,
        false,
        100,
        15.0,
        35.0,
        Duration::from_secs(3600),
        45.0,
        100.0,
        85.0,
    )?;

    assert!(battery.is_low()?);

    let battery = Battery::with_values(
        true,
        false,
        100,
        85.0,
        35.0,
        Duration::from_secs(3600),
        45.0,
        100.0,
        85.0,
    )?;

    assert!(!battery.is_low()?);

    Ok(())
}

#[test]
fn test_battery_time_remaining_display() -> Result<()> {
    setup_test();
    let battery = Battery::with_values(
        true,
        false,
        100,
        85.0,
        35.0,
        Duration::from_secs(5400), // 1h 30m
        45.0,
        100.0,
        85.0,
    )?;

    assert_eq!(battery.time_remaining_display()?, "1h 30m");

    let battery = Battery::with_values(
        true,
        false,
        100,
        85.0,
        35.0,
        Duration::from_secs(2700), // 45m
        45.0,
        100.0,
        85.0,
    )?;

    assert_eq!(battery.time_remaining_display()?, "45m");

    Ok(())
}

#[test]
fn test_battery_is_health_critical() -> Result<()> {
    setup_test();
    let battery = Battery::with_values(
        true,
        false,
        100,
        75.0,
        35.0,
        Duration::from_secs(3600),
        45.0,
        100.0,
        75.0,
    )?;

    assert!(battery.is_health_critical()?);

    let battery = Battery::with_values(
        true,
        false,
        100,
        85.0,
        35.0,
        Duration::from_secs(3600),
        45.0,
        100.0,
        85.0,
    )?;

    assert!(!battery.is_health_critical()?);

    Ok(())
}

#[test]
fn test_battery_is_cycle_count_critical() -> Result<()> {
    setup_test();
    let battery = Battery::with_values(
        true,
        false,
        1200,
        85.0,
        35.0,
        Duration::from_secs(3600),
        45.0,
        100.0,
        85.0,
    )?;

    assert!(battery.is_cycle_count_critical()?);

    let battery = Battery::with_values(
        true,
        false,
        300,
        85.0,
        35.0,
        Duration::from_secs(3600),
        45.0,
        100.0,
        85.0,
    )?;

    assert!(!battery.is_cycle_count_critical()?);

    Ok(())
}

#[test]
fn test_battery_power_source_display() -> Result<()> {
    setup_test();
    let battery = Battery::with_values(
        true,
        false,
        100,
        85.0,
        35.0,
        Duration::from_secs(3600),
        45.0,
        100.0,
        85.0,
    )?;

    assert_eq!(battery.power_source_display()?, "Battery");

    Ok(())
}

#[test]
fn test_battery_is_temperature_critical() -> Result<()> {
    setup_test();
    let battery = Battery::with_values(
        true,
        false,
        100,
        85.0,
        46.0,
        Duration::from_secs(3600),
        45.0,
        100.0,
        85.0,
    )?;

    assert!(battery.is_temperature_critical()?);

    let battery = Battery::with_values(
        true,
        false,
        100,
        85.0,
        35.0,
        Duration::from_secs(3600),
        45.0,
        100.0,
        85.0,
    )?;

    assert!(!battery.is_temperature_critical()?);

    Ok(())
}

#[test]
fn test_battery_clone() -> Result<()> {
    setup_test();
    let battery = Battery::with_values(
        true,
        false,
        100,
        85.0,
        35.0,
        Duration::from_secs(3600),
        45.0,
        100.0,
        85.0,
    )?;

    let cloned = battery.clone();

    assert_eq!(cloned.is_present()?, battery.is_present()?);
    assert_eq!(cloned.percentage()?, battery.percentage()?);
    assert_eq!(cloned.cycle_count()?, battery.cycle_count()?);
    assert_eq!(cloned.temperature()?, battery.temperature()?);

    Ok(())
}
