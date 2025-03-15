#[cfg(test)]
use super::*;
use crate::hardware::iokit::{FanInfo, GpuStats, ThermalInfo, IOKit, ThreadSafeAnyObject};
use crate::utils::safe_dictionary::SafeDictionary;
use crate::utils::test_utils::{create_test_dictionary, create_test_object};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSDictionary, NSObject, NSString};
use std::os::raw::c_char;
use std::sync::Once;
use crate::error::{Error, Result};

static INIT: Once = Once::new();

fn setup_test() {
    // Ensure IOKit is initialized only once
    INIT.call_once(|| {
        // Any global IOKit initialization if needed
    });
}

// Manual mock implementation of IOKit for testing
#[derive(Debug)]
struct MockIOKit {
    battery_info: Option<BatteryInfo>,
}

impl MockIOKit {
    pub fn new() -> Result<Self> {
        Ok(Self { battery_info: None })
    }

    pub fn with_battery_info(
        mut self,
        is_present: bool,
        is_charging: bool,
        cycle_count: i64,
        percentage: f64,
        temperature: f64,
        time_remaining: i64,
        design_capacity: f64,
        current_capacity: f64,
    ) -> Result<Self> {
        self.battery_info = Some(BatteryInfo {
            present: is_present,
            percentage: percentage as i64,
            cycle_count,
            is_charging,
            is_external: !is_charging,
            temperature,
            power_draw: 0.0,
            design_capacity: design_capacity as i64,
            current_capacity: current_capacity as i64,
            time_remaining: Some(Duration::from_secs(time_remaining as u64)),
        });
        Ok(self)
    }
}

impl Default for MockIOKit {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Clone for MockIOKit {
    fn clone(&self) -> Self {
        Self {
            battery_info: self.battery_info.clone(),
        }
    }
}

impl IOKit for MockIOKit {
    fn io_service_matching(&self, _name: &str) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_service_matching(&self, _name: &str) -> Result<Option<ThreadSafeAnyObject>> {
        Ok(None)
    }

    fn io_service_get_matching_service(&self, _matching: &SafeDictionary) -> Result<ThreadSafeAnyObject> {
        Ok(ThreadSafeAnyObject::new(NSObject::new()))
    }

    fn io_registry_entry_create_cf_properties(&self, _entry: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_cpu_temperature(&self, _plane: &str) -> Result<f64> {
        Ok(42.0)
    }

    fn get_all_fans(&self) -> Result<Vec<FanInfo>> {
        Ok(vec![])
    }

    fn check_thermal_throttling(&self, _plane: &str) -> Result<bool> {
        Ok(false)
    }

    fn get_cpu_power(&self) -> Result<f64> {
        Ok(15.0)
    }

    fn get_battery_temperature(&self) -> Result<Option<f64>> {
        Ok(self.battery_info.as_ref().map(|info| info.temperature))
    }

    fn get_battery_info(&self) -> Result<SafeDictionary> {
        let mut dict = SafeDictionary::new();
        if let Some(info) = &self.battery_info {
            dict.set_bool("BatteryInstalled", info.present);
            dict.set_bool("IsCharging", info.is_charging);
            dict.set_bool("ExternalConnected", info.is_external);
            dict.set_number("CycleCount", info.cycle_count as f64);
            dict.set_number("CurrentCapacity", info.percentage as f64);
            dict.set_number("MaxCapacity", info.current_capacity as f64);
            dict.set_number("DesignCapacity", info.design_capacity as f64);
            dict.set_number("Temperature", info.temperature);
        }
        Ok(dict)
    }

    fn get_cpu_info(&self) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_number_property(&self, _dict: &SafeDictionary, _key: &str) -> Result<f64> {
        Ok(0.0)
    }

    fn io_connect_call_method(&self, _connection: u32, _selector: u32, _input: &[u64], _output: &mut [u64]) -> Result<()> {
        Ok(())
    }

    fn io_registry_entry_get_parent_entry(&self, _entry: &ThreadSafeAnyObject) -> Result<ThreadSafeAnyObject> {
        Ok(ThreadSafeAnyObject::new(NSObject::new()))
    }

    fn get_service_properties(&self, _service: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        self.get_battery_info()
    }

    fn get_physical_cores(&self) -> Result<usize> {
        Ok(4)
    }

    fn get_logical_cores(&self) -> Result<usize> {
        Ok(8)
    }

    fn get_core_usage(&self) -> Result<Vec<f64>> {
        Ok(vec![0.5, 0.6, 0.7, 0.8])
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo> {
        let mut dict = SafeDictionary::new();
        dict.set_number("CPU_0_DIE_TEMP", 42.0);
        dict.set_number("GPU_0_DIE_TEMP", 55.0);
        dict.set_number("FAN_0_SPEED", 2000.0);
        dict.set_number("HS_0_TEMP", 40.0);
        dict.set_number("AMBIENT_TEMP", 25.0);
        dict.set_number("BATTERY_TEMP", self.get_battery_temperature()?.unwrap_or(35.0));
        dict.set_bool("THERMAL_THROTTLING", false);
        Ok(ThermalInfo::new(dict))
    }

    fn get_gpu_stats(&self) -> Result<GpuStats> {
        Ok(GpuStats::default())
    }

    fn get_fan_info(&self, _fan_index: u32) -> Result<FanInfo> {
        Ok(FanInfo {
            speed_rpm: 2000,
            min_speed: 1000,
            max_speed: 4000,
            percentage: 50.0,
        })
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
    }
}

pub struct BatteryValues {
    pub is_present: bool,
    pub is_charging: bool,
    pub cycle_count: i64,
    pub percentage: f64,
    pub temperature: f64,
    pub design_capacity: f64,
    pub current_capacity: f64,
}

impl Battery {
    pub fn with_values(values: BatteryValues) -> Result<Self> {
        let mock_iokit = MockIOKit::new()?.with_battery_info(
            values.is_present,
            values.is_charging,
            values.cycle_count,
            values.percentage,
            values.temperature,
            0, // time remaining not used in tests
            values.design_capacity,
            values.current_capacity,
        )?;

        Self::new(Box::new(mock_iokit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_values(
        is_present: bool,
        is_charging: bool,
        cycle_count: i64,
        percentage: f64,
        temperature: f64,
        design_capacity: f64,
        current_capacity: f64,
    ) -> BatteryValues {
        BatteryValues {
            is_present,
            is_charging,
            cycle_count,
            percentage,
            temperature,
            design_capacity,
            current_capacity,
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
        let values = create_test_values(true, false, 100, 85.0, 35.0, 100.0, 85.0);
        let battery = Battery::with_values(values)?;

        assert!(battery.is_present()?);
        assert!(!battery.is_charging()?);
        assert_eq!(battery.cycle_count()?, 100);
        assert_eq!(battery.percentage()?, 85);
        assert_eq!(battery.temperature()?, 35.0);
        assert_eq!(battery.design_capacity()?, 100);
        assert_eq!(battery.current_capacity()?, 85);

        Ok(())
    }

    #[test]
    fn test_battery_is_critically_low() -> Result<()> {
        setup_test();
        let values = create_test_values(true, false, 100, 4.0, 35.0, 100.0, 85.0);
        let battery = Battery::with_values(values)?;
        assert!(battery.is_critically_low()?);

        let values = create_test_values(true, false, 100, 85.0, 35.0, 100.0, 85.0);
        let battery = Battery::with_values(values)?;
        assert!(!battery.is_critically_low()?);

        Ok(())
    }

    #[test]
    fn test_battery_is_low() -> Result<()> {
        setup_test();
        let values = create_test_values(true, false, 100, 15.0, 35.0, 100.0, 85.0);
        let battery = Battery::with_values(values)?;
        assert!(battery.is_low()?);

        let values = create_test_values(true, false, 100, 85.0, 35.0, 100.0, 85.0);
        let battery = Battery::with_values(values)?;
        assert!(!battery.is_low()?);

        Ok(())
    }

    #[test]
    fn test_battery_is_health_critical() -> Result<()> {
        setup_test();
        let values = create_test_values(true, false, 100, 75.0, 35.0, 100.0, 75.0);
        let battery = Battery::with_values(values)?;
        assert!(battery.is_health_critical()?);

        let values = create_test_values(true, false, 100, 85.0, 35.0, 100.0, 85.0);
        let battery = Battery::with_values(values)?;
        assert!(!battery.is_health_critical()?);

        Ok(())
    }

    #[test]
    fn test_battery_is_cycle_count_critical() -> Result<()> {
        setup_test();
        let values = create_test_values(true, false, 1200, 85.0, 35.0, 100.0, 85.0);
        let battery = Battery::with_values(values)?;
        assert!(battery.is_cycle_count_critical()?);

        let values = create_test_values(true, false, 300, 85.0, 35.0, 100.0, 85.0);
        let battery = Battery::with_values(values)?;
        assert!(!battery.is_cycle_count_critical()?);

        Ok(())
    }

    #[test]
    fn test_battery_power_source_display() -> Result<()> {
        setup_test();
        let values = create_test_values(true, false, 100, 85.0, 35.0, 100.0, 85.0);
        let battery = Battery::with_values(values)?;
        assert_eq!(battery.power_source_display()?, "Battery");
        Ok(())
    }

    #[test]
    fn test_battery_is_temperature_critical() -> Result<()> {
        setup_test();
        let values = create_test_values(true, false, 100, 85.0, 46.0, 100.0, 85.0);
        let battery = Battery::with_values(values)?;
        assert!(battery.is_temperature_critical()?);

        let values = create_test_values(true, false, 100, 85.0, 35.0, 100.0, 85.0);
        let battery = Battery::with_values(values)?;
        assert!(!battery.is_temperature_critical()?);

        Ok(())
    }

    #[test]
    fn test_battery_clone() -> Result<()> {
        setup_test();
        let values = create_test_values(true, false, 100, 85.0, 35.0, 100.0, 85.0);
        let battery = Battery::with_values(values)?;
        let cloned = battery.clone();

        assert_eq!(cloned.is_present()?, battery.is_present()?);
        assert_eq!(cloned.percentage()?, battery.percentage()?);
        assert_eq!(cloned.cycle_count()?, battery.cycle_count()?);
        assert_eq!(cloned.temperature()?, battery.temperature()?);

        Ok(())
    }
}
