use super::*;
use crate::hardware::iokit::{FanInfo, GpuStats, ThermalInfo, ThreadSafeAnyObject, ThreadSafeNSDictionary};
use crate::Error;
use crate::GpuMetrics; // Import GpuMetrics
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
    thermal_info: Arc<dyn Fn(ThreadSafeAnyObject) -> Result<ThermalInfo, Error> + Send + Sync>,
    fan_info: Arc<dyn Fn(ThreadSafeAnyObject) -> Result<Vec<FanInfo>, Error> + Send + Sync>,
}

impl std::fmt::Debug for MockIOKitClone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockIOKitClone").field("thermal_info", &"<function>").field("fan_info", &"<function>").finish()
    }
}

impl MockIOKitClone {
    fn new() -> Self {
        Self {
            thermal_info: Arc::new(|_| Ok(ThermalInfo::new(ThreadSafeNSDictionary::empty()))),
            fan_info: Arc::new(|_| {
                Ok(vec![
                    FanInfo { speed_rpm: 2000, min_speed: 1000, max_speed: 4000, percentage: 33.3 },
                    FanInfo { speed_rpm: 2500, min_speed: 1200, max_speed: 5000, percentage: 40.0 },
                ])
            }),
        }
    }

    fn with_thermal_info<F>(self, f: F) -> Self
    where
        F: Fn(ThreadSafeAnyObject) -> Result<ThermalInfo, Error> + Send + Sync + 'static,
    {
        Self { thermal_info: Arc::new(f), fan_info: self.fan_info }
    }

    fn with_fan_info<F>(self, f: F) -> Self
    where
        F: Fn(ThreadSafeAnyObject) -> Result<Vec<FanInfo>, Error> + Send + Sync + 'static,
    {
        Self { thermal_info: self.thermal_info, fan_info: Arc::new(f) }
    }
}

impl IOKit for MockIOKitClone {
    fn io_service_matching(&self, _name: &str) -> Result<ThreadSafeNSDictionary, Error> {
        Ok(ThreadSafeNSDictionary::empty())
    }

    fn io_service_get_matching_service(
        &self,
        _matching: &ThreadSafeNSDictionary,
    ) -> Result<ThreadSafeAnyObject, Error> {
        Ok(ThreadSafeAnyObject::new(create_test_object()))
    }

    fn io_registry_entry_create_cf_properties(
        &self,
        _entry: &ThreadSafeAnyObject,
    ) -> Result<ThreadSafeNSDictionary, Error> {
        Ok(ThreadSafeNSDictionary::empty())
    }

    fn io_connect_call_method(
        &self,
        _connection: u32,
        _selector: u32,
        _input: &[u8],
        _input_cnt: u32,
        _output: &mut [u8],
        _output_cnt: &mut u32,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn get_number_property(&self, _dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<f64> {
        match key {
            "CurrentCapacity" => Some(85.0),
            "CycleCount" => Some(100.0),
            "Temperature" => Some(35.0),
            "Voltage" => Some(12.0),
            "Amperage" => Some(1.5),
            "DesignCapacity" => Some(100.0),
            _ => None,
        }
    }

    fn get_service_properties(&self, _service: &ThreadSafeAnyObject) -> Result<ThreadSafeNSDictionary, Error> {
        Ok(ThreadSafeNSDictionary::empty())
    }

    fn get_service_matching(&self, _name: &str) -> Result<Option<ThreadSafeAnyObject>, Error> {
        Ok(None)
    }

    fn get_cpu_temperature(&self) -> Result<f64, Error> {
        Ok((*self.thermal_info)(ThreadSafeAnyObject::new(create_test_object()))?.cpu_temp)
    }

    fn get_gpu_stats(&self) -> Result<GpuStats, Error> {
        unimplemented!("Not needed for tests")
    }

    fn get_fan_info(&self, fan_index: u32) -> Result<FanInfo, Error> {
        let fans = (*self.fan_info)(ThreadSafeAnyObject::new(create_test_object()))?;
        fans.get(fan_index as usize)
            .cloned()
            .ok_or_else(|| Error::iokit_error(-1, format!("Fan index out of bounds: {}", fan_index)))
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo, Error> {
        (*self.thermal_info)(ThreadSafeAnyObject::new(create_test_object()))
    }

    fn get_all_fans(&self) -> Result<Vec<FanInfo>, Error> {
        (*self.fan_info)(ThreadSafeAnyObject::new(create_test_object()))
    }

    fn check_thermal_throttling(&self) -> Result<bool, Error> {
        Ok(false)
    }

    fn get_cpu_power(&self) -> Result<f64, Error> {
        Ok(25.0)
    }

    fn get_battery_temperature(&self) -> Result<Option<f64>, Error> {
        Ok(Some(35.0))
    }

    fn get_battery_info(&self) -> Result<ThreadSafeNSDictionary, Error> {
        Ok(ThreadSafeNSDictionary::empty())
    }

    fn get_cpu_info(&self) -> Result<ThreadSafeNSDictionary, Error> {
        Ok(ThreadSafeNSDictionary::empty())
    }

    fn io_registry_entry_get_parent_entry(
        &self,
        _entry: &ThreadSafeAnyObject,
        _plane: &str,
    ) -> Result<ThreadSafeAnyObject, Error> {
        Ok(ThreadSafeAnyObject::new(create_test_object()))
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
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

// Rest of the code remains the same
