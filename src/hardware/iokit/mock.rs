//! This module is only used to make the automock-generated MockIOKit available to tests in other modules.

use super::{ThreadSafeAnyObject, ThreadSafeNSDictionary};
use crate::utils::test_utils::create_test_object as other_create_test_object;
use crate::{
    error::{Error, Result},
    hardware::iokit::{FanInfo, GpuStats, IOKit, ThermalInfo},
};
use objc2::{rc::Retained, runtime::AnyObject};
use objc2_foundation::{NSDictionary, NSObject, NSString};
use std::time::Duration;

#[derive(Debug)]
pub struct MockIOKit {
    pub battery_is_present: bool,
    pub battery_is_charging: bool,
    pub battery_cycle_count: u32,
    pub battery_health_percentage: f64,
    pub battery_temperature: f64,
    pub battery_time_remaining: Duration,
    pub battery_power_draw: f64,
    pub battery_design_capacity: f64,
    pub battery_current_capacity: f64,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub core_usage: Vec<f64>,
    pub cpu_temperature: f64,
}

impl MockIOKit {
    pub fn new() -> Self {
        Self {
            battery_is_present: true,
            battery_is_charging: false,
            battery_cycle_count: 0,
            battery_health_percentage: 100.0,
            battery_temperature: 30.0,
            battery_time_remaining: Duration::from_secs(3600),
            battery_power_draw: 10.0,
            battery_design_capacity: 100.0,
            battery_current_capacity: 80.0,
            physical_cores: 4,
            logical_cores: 8,
            core_usage: vec![0.0; 4],
            cpu_temperature: 45.0,
        }
    }

    pub fn set_physical_cores(&mut self, cores: usize) {
        self.physical_cores = cores;
    }

    pub fn set_logical_cores(&mut self, cores: usize) {
        self.logical_cores = cores;
    }

    pub fn set_core_usage(&mut self, usage: Vec<f64>) {
        self.core_usage = usage;
    }

    pub fn set_temperature(&mut self, temp: f64) {
        self.cpu_temperature = temp;
    }
}

impl IOKit for MockIOKit {
    fn get_thermal_info(&self) -> Result<ThermalInfo> {
        Ok(ThermalInfo {
            cpu_temp: self.cpu_temperature,
            gpu_temp: 55.0,
            fan_speed: 2000,
            heatsink_temp: 50.0,
            ambient_temp: 25.0,
            battery_temp: self.battery_temperature,
            thermal_throttling: false,
            dict: ThreadSafeNSDictionary::empty(),
        })
    }

    fn io_service_matching(&self, _name: &str) -> Result<ThreadSafeNSDictionary> {
        Ok(ThreadSafeNSDictionary::empty())
    }

    fn io_service_get_matching_service(
        &self,
        _matching: &ThreadSafeNSDictionary,
    ) -> Result<ThreadSafeAnyObject> {
        Ok(ThreadSafeAnyObject::new(create_test_object()))
    }

    fn io_registry_entry_create_cf_properties(
        &self,
        _service: &ThreadSafeAnyObject,
    ) -> Result<ThreadSafeNSDictionary> {
        Ok(ThreadSafeNSDictionary::empty())
    }

    fn get_gpu_stats(&self) -> Result<GpuStats> {
        Ok(GpuStats {
            utilization: 50.0,
            memory_used: 1024,
            memory_total: 4096,
            perf_cap: 100.0,
            perf_threshold: 90.0,
            name: "Mock GPU".to_string(),
        })
    }

    fn get_fan_info(&self, _fan_index: u32) -> Result<FanInfo> {
        Ok(FanInfo { speed_rpm: 2000, min_speed: 0, max_speed: 5000, percentage: 40.0 })
    }

    fn get_service_matching(&self, _name: &str) -> Result<Option<ThreadSafeAnyObject>> {
        Ok(Some(ThreadSafeAnyObject::new(create_test_object())))
    }

    fn get_service_properties(
        &self,
        _service: &ThreadSafeAnyObject,
    ) -> Result<ThreadSafeNSDictionary> {
        Ok(ThreadSafeNSDictionary::empty())
    }

    fn get_cpu_temperature(&self) -> Result<f64> {
        Ok(self.cpu_temperature)
    }

    fn get_battery_temperature(&self) -> Result<Option<f64>> {
        Ok(Some(self.battery_temperature))
    }

    fn get_battery_info(&self) -> Result<ThreadSafeNSDictionary> {
        Ok(ThreadSafeNSDictionary::empty())
    }

    fn get_cpu_info(&self) -> Result<ThreadSafeNSDictionary> {
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
    ) -> Result<()> {
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
    }

    fn get_all_fans(&self) -> Result<Vec<FanInfo>> {
        Ok(vec![FanInfo {
            speed_rpm: 2000,
            min_speed: 0,
            max_speed: 5000,
            percentage: 40.0,
        }])
    }

    fn check_thermal_throttling(&self) -> Result<bool> {
        Ok(false)
    }

    fn get_cpu_power(&self) -> Result<f64> {
        Ok(25.0)
    }

    fn get_number_property(
        &self,
        _dict: &NSDictionary<NSString, NSObject>,
        _key: &str,
    ) -> Option<f64> {
        Some(42.0)
    }

    fn io_registry_entry_get_parent_entry(
        &self,
        _entry: &ThreadSafeAnyObject,
        _plane: &str,
    ) -> Result<ThreadSafeAnyObject> {
        Ok(ThreadSafeAnyObject::new(create_test_object()))
    }
}

impl Clone for MockIOKit {
    fn clone(&self) -> Self {
        Self {
            battery_is_present: self.battery_is_present,
            battery_is_charging: self.battery_is_charging,
            battery_cycle_count: self.battery_cycle_count,
            battery_health_percentage: self.battery_health_percentage,
            battery_temperature: self.battery_temperature,
            battery_time_remaining: self.battery_time_remaining,
            battery_power_draw: self.battery_power_draw,
            battery_design_capacity: self.battery_design_capacity,
            battery_current_capacity: self.battery_current_capacity,
            physical_cores: self.physical_cores,
            logical_cores: self.logical_cores,
            core_usage: self.core_usage.clone(),
            cpu_temperature: self.cpu_temperature,
        }
    }
}

fn create_test_object() -> Retained<NSObject> {
    let dict = NSDictionary::<NSString, NSObject>::new();
    unsafe { 
        let obj_ptr = dict.as_ref() as *const NSObject as *mut NSObject;
        Retained::from_raw(obj_ptr).expect("Failed to create Retained<NSObject>") 
    }
}
