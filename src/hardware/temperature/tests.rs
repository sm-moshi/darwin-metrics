use super::*;
use crate::hardware::iokit::{FanInfo, GpuStats, ThermalInfo, ThreadSafeAnyObject};
use crate::utils::safe_dictionary::SafeDictionary;
use crate::Error;
use crate::GpuMetrics;
use objc2::rc::Retained;
use objc2_foundation::{NSDictionary, NSObject, NSString};
use std::sync::Arc;

// Helper function to create a test object
fn create_test_object() -> Retained<NSObject> {
    let dict = NSDictionary::<NSString, NSObject>::new();
    unsafe {
        let obj_ptr = dict.as_ref() as *const NSObject as *mut NSObject;
        Retained::from_raw(obj_ptr).expect("Failed to create Retained<NSObject>")
    }
}

// Custom mock implementation of IOKit that implements Clone
#[derive(Clone)]
struct MockIOKitClone {
    thermal_info: Arc<dyn Fn() -> Result<ThermalInfo, Error> + Send + Sync>,
    fan_info: Arc<dyn Fn() -> Result<Vec<FanInfo>, Error> + Send + Sync>,
}

impl std::fmt::Debug for MockIOKitClone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockIOKitClone").field("thermal_info", &"<function>").field("fan_info", &"<function>").finish()
    }
}

impl MockIOKitClone {
    fn new() -> Self {
        let entries = [
            ("CPU_0_DIE_TEMP", 45.0),
            ("GPU_0_DIE_TEMP", 40.0),
            ("FAN_0_SPEED", 1500.0),
            ("HS_0_TEMP", 35.0),
            ("AMBIENT_TEMP", 25.0),
            ("THERMAL_THROTTLING", 0.0),
            ("BATTERY_TEMP", 32.0),
        ];
        let dict = crate::utils::test_utils::create_test_dictionary_with_entries(&entries);
        let thermal_info = Arc::new(ThermalInfo::new(SafeDictionary::from(dict.into())));

        Self {
            thermal_info: Arc::new(move || Ok((*thermal_info).clone())),
            fan_info: Arc::new(|| {
                Ok(vec![
                    FanInfo { speed_rpm: 2000, min_speed: 1000, max_speed: 4000, percentage: 33.3 },
                    FanInfo { speed_rpm: 2500, min_speed: 1200, max_speed: 5000, percentage: 40.0 },
                ])
            }),
        }
    }

    fn with_thermal_info<F>(self, f: F) -> Self
    where
        F: Fn() -> Result<ThermalInfo, Error> + Send + Sync + 'static,
    {
        Self { thermal_info: Arc::new(f), fan_info: self.fan_info }
    }

    fn with_fan_info<F>(self, f: F) -> Self
    where
        F: Fn() -> Result<Vec<FanInfo>, Error> + Send + Sync + 'static,
    {
        Self { thermal_info: self.thermal_info, fan_info: Arc::new(f) }
    }
}

impl IOKit for MockIOKitClone {
    fn io_service_matching(&self, _name: &str) -> Result<SafeDictionary, Error> {
        Ok(SafeDictionary::new())
    }

    fn io_service_get_matching_service(&self, _matching: &SafeDictionary) -> Result<ThreadSafeAnyObject, Error> {
        Ok(ThreadSafeAnyObject::new(NSObject::new()))
    }

    fn io_registry_entry_create_cf_properties(&self, _entry: &ThreadSafeAnyObject) -> Result<SafeDictionary, Error> {
        Ok(SafeDictionary::new())
    }

    fn io_connect_call_method(
        &self,
        _connection: u32,
        _selector: u32,
        _input: &[u64],
        _output: &mut [u64],
    ) -> Result<(), Error> {
        Ok(())
    }

    fn get_number_property(&self, _dict: &SafeDictionary, _key: &str) -> Result<f64, Error> {
        Ok(0.0)
    }

    fn get_service_properties(&self, _service: &ThreadSafeAnyObject) -> Result<SafeDictionary, Error> {
        Ok(SafeDictionary::new())
    }

    fn get_service_matching(&self, _name: &str) -> Result<Option<ThreadSafeAnyObject>, Error> {
        Ok(None)
    }

    fn get_cpu_temperature(&self, _plane: &str) -> Result<f64, Error> {
        Ok(0.0)
    }

    fn get_gpu_stats(&self) -> Result<GpuStats, Error> {
        unimplemented!("Not needed for tests")
    }

    fn get_fan_info(&self, fan_index: u32) -> Result<FanInfo, Error> {
        let fans = (*self.fan_info)()?;
        fans.get(fan_index as usize)
            .cloned()
            .ok_or_else(|| Error::IOKitError { code: 0, message: format!("Fan index out of bounds: {}", fan_index) })
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo, Error> {
        (*self.thermal_info)()
    }

    fn get_all_fans(&self) -> Result<Vec<FanInfo>, Error> {
        (*self.fan_info)()
    }

    fn check_thermal_throttling(&self, _plane: &str) -> Result<bool, Error> {
        Ok(false)
    }

    fn get_cpu_power(&self) -> Result<f64, Error> {
        Ok(25.0)
    }

    fn get_battery_temperature(&self) -> Result<Option<f64>, Error> {
        Ok(Some(0.0))
    }

    fn get_battery_info(&self) -> Result<SafeDictionary, Error> {
        Ok(SafeDictionary::new())
    }

    fn get_cpu_info(&self) -> Result<SafeDictionary, Error> {
        Ok(SafeDictionary::new())
    }

    fn io_registry_entry_get_parent_entry(&self, _entry: &ThreadSafeAnyObject) -> Result<ThreadSafeAnyObject, Error> {
        Ok(ThreadSafeAnyObject::new(NSObject::new()))
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
    }

    fn get_physical_cores(&self) -> Result<usize, Error> {
        Ok(4)
    }

    fn get_logical_cores(&self) -> Result<usize, Error> {
        Ok(8)
    }

    fn get_core_usage(&self) -> Result<Vec<f64>, Error> {
        Ok(vec![0.5, 0.6, 0.7, 0.8])
    }
}

/// Formats memory metrics for display
fn display_memory_info(metrics: &GpuMetrics) {
    println!("Memory Information:");
    println!("------------------");
    println!("Total Memory: {}", metrics.memory.total);
    println!(
        "Used Memory: {} ({:.1}%)",
        metrics.memory.used,
        (metrics.memory.used as f64 / metrics.memory.total as f64) * 100.0
    );
    if let Some(temp) = metrics.temperature {
        println!("GPU Temperature: {}°C", temp);
    }
    println!("GPU Name: {}", metrics.name);
    println!("Characteristics: {:?}", metrics.characteristics);
}

#[test]
fn test_thermal_info() {
    let entries = [
        ("CPU_0_DIE_TEMP", 45.0),
        ("GPU_0_DIE_TEMP", 55.0),
        ("FAN_0_SPEED", 2000.0),
        ("HS_0_TEMP", 50.0),
        ("AMBIENT_TEMP", 25.0),
        ("THERMAL_THROTTLING", 0.0),
        ("BATTERY_TEMP", 35.0),
    ];
    let dict = crate::utils::test_utils::create_test_dictionary_with_entries(&entries);
    let thermal_info = Arc::new(ThermalInfo::new(SafeDictionary::from(dict)));

    let expected_cpu_temp = 45.0;
    let expected_gpu_temp = 35.0;
    let expected_fan_speed = 2000;
    let expected_ambient_temp = 25.0;
    let expected_battery_temp = 30.0;
    let expected_heatsink_temp = 40.0;

    assert_eq!(thermal_info.cpu_temp, expected_cpu_temp);
    assert_eq!(thermal_info.gpu_temp, Some(expected_gpu_temp));
    assert_eq!(thermal_info.fan_speed, expected_fan_speed);
    assert_eq!(thermal_info.ambient_temp, Some(expected_ambient_temp));
    assert_eq!(thermal_info.battery_temp, Some(expected_battery_temp));
    assert_eq!(thermal_info.heatsink_temp, Some(expected_heatsink_temp));
}

// Rest of the code remains the same
