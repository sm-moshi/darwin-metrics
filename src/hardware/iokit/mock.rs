//! This module is only used to make the automock-generated MockIOKit available to tests in other modules.

use std::ffi::c_void;
use std::fmt::Debug;
use std::ops::Deref;
use std::time::Duration;

use objc2::{
    msg_send,
    rc::Retained,
    runtime::{AnyClass, AnyObject, NSObject},
    ClassType,
};
use objc2_foundation::{NSArray, NSDictionary, NSNumber, NSString};

use crate::error::Error;
use crate::{
    battery::BatteryInfo,
    error::Result,
    hardware::iokit::{FanInfo, GpuStats, IOKit, ThermalInfo, ThreadSafeAnyObject},
    utils::SafeDictionary,
};

#[derive(Debug, Clone)]
pub struct MockIOKit {
    obj: ThreadSafeAnyObject,
    physical_cores: usize,
    logical_cores: usize,
    core_usage: Vec<f64>,
    temperature: f64,
    battery_is_present: bool,
    battery_is_charging: bool,
    battery_time_remaining: Duration,
    battery_percentage: f64,
    battery_temperature: f64,
    battery_cycle_count: i64,
    battery_design_capacity: f64,
    battery_current_capacity: f64,
    thermal_info: ThermalInfo,
    gpu_stats: GpuStats,
}

impl Default for MockIOKit {
    fn default() -> Self {
        Self::new().expect("Failed to create MockIOKit")
    }
}

impl MockIOKit {
    pub fn new() -> Result<Self> {
        unsafe {
            let obj: *mut NSObject = msg_send![NSObject::class(), new];
            let retained = Retained::from_raw(obj).expect("Failed to create NSObject");
            Ok(Self {
                obj: ThreadSafeAnyObject::with_raw_handle(retained, obj as _),
                battery_time_remaining: Duration::from_secs(0),
                physical_cores: 0,
                logical_cores: 0,
                core_usage: vec![],
                temperature: 0.0,
                battery_is_present: true,
                battery_is_charging: false,
                battery_cycle_count: 0,
                battery_percentage: 100.0,
                battery_temperature: 0.0,
                battery_design_capacity: 100.0,
                battery_current_capacity: 80.0,
                thermal_info: ThermalInfo::default(),
                gpu_stats: GpuStats::default(),
            })
        }
    }

    pub fn with_physical_cores(mut self, cores: usize) -> Result<Self> {
        self.physical_cores = cores;
        Ok(self)
    }

    pub fn with_logical_cores(mut self, cores: usize) -> Result<Self> {
        self.logical_cores = cores;
        Ok(self)
    }

    pub fn with_core_usage(mut self, usage: Vec<f64>) -> Result<Self> {
        self.core_usage = usage;
        Ok(self)
    }

    pub fn with_temperature(mut self, temp: f64) -> Self {
        self.temperature = temp;
        self
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
    ) -> Self {
        self.battery_is_present = is_present;
        self.battery_is_charging = is_charging;
        self.battery_cycle_count = cycle_count;
        self.battery_percentage = percentage;
        self.battery_temperature = temperature;
        self.battery_time_remaining = Duration::from_secs(time_remaining as u64);
        self.battery_design_capacity = design_capacity;
        self.battery_current_capacity = current_capacity;
        self
    }

    pub fn with_time_remaining(mut self, time_remaining: i64) -> Result<Self> {
        self.battery_time_remaining = Duration::from_secs(time_remaining as u64);
        Ok(self)
    }

    pub fn with_thermal_info(mut self, info: ThermalInfo) -> Result<Self> {
        self.thermal_info = info;
        Ok(self)
    }

    pub fn with_gpu_stats(mut self, stats: GpuStats) -> Result<Self> {
        self.gpu_stats = stats;
        Ok(self)
    }

    /// Creates an empty NSDictionary for testing
    fn create_empty_dictionary() -> SafeDictionary {
        SafeDictionary::new()
    }
}

impl DictionaryAccess for MockIOKit {
    fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        None // Mock implementation
    }
}

impl IOKit for MockIOKit {
    fn io_service_matching(&self, name: &str) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn io_service_get_matching_service(&self, matching_dict: &SafeDictionary) -> Result<ThreadSafeAnyObject> {
        Ok(ThreadSafeAnyObject::default())
    }

    fn io_registry_entry_create_cf_properties(&self, entry: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_service_properties(&self, service: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_battery_info(&self) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_cpu_info(&self) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_thermal_info(&self) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_all_fans(&self) -> Result<Vec<SafeDictionary>> {
        Ok(vec![SafeDictionary::new()])
    }

    fn check_thermal_throttling(&self) -> Result<bool> {
        Ok(false)
    }

    fn get_cpu_power(&self) -> Result<f64> {
        Ok(0.0)
    }

    fn get_gpu_stats(&self) -> Result<GpuStats> {
        Ok(GpuStats::default())
    }

    fn get_fan_info(&self, _fan_index: u32) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_battery_temperature(&self) -> Result<f64> {
        Ok(0.0)
    }

    fn get_number_property(&self, _dict: &SafeDictionary, _key: &str) -> Result<f64> {
        Ok(0.0)
    }

    fn io_connect_call_method(&self, _connection: u32, _selector: u32, _input: &[u64], _output: &mut [u64]) -> Result<()> {
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
    }

    fn io_registry_entry_get_parent_entry(&self, _entry: &ThreadSafeAnyObject) -> Result<ThreadSafeAnyObject> {
        Ok(ThreadSafeAnyObject::default())
    }

    fn get_physical_cores(&self) -> Result<usize> {
        Ok(1)
    }

    fn get_logical_cores(&self) -> Result<usize> {
        Ok(1)
    }

    fn get_core_usage(&self) -> Result<Vec<f64>> {
        Ok(vec![0.0])
    }

    fn get_cpu_temperature(&self) -> Result<f64> {
        Ok(0.0)
    }

    fn get_service_matching(&self, name: &str) -> Result<Option<ThreadSafeAnyObject>> {
        Ok(None)
    }
}

impl Clone for MockIOKit {
    fn clone(&self) -> Self {
        Self {
            obj: self.obj.clone(),
            physical_cores: self.physical_cores,
            logical_cores: self.logical_cores,
            core_usage: self.core_usage.clone(),
            temperature: self.temperature,
            battery_is_present: self.battery_is_present,
            battery_is_charging: self.battery_is_charging,
            battery_time_remaining: self.battery_time_remaining,
            battery_percentage: self.battery_percentage,
            battery_temperature: self.battery_temperature,
            battery_cycle_count: self.battery_cycle_count,
            battery_design_capacity: self.battery_design_capacity,
            battery_current_capacity: self.battery_current_capacity,
            thermal_info: self.thermal_info.clone(),
            gpu_stats: self.gpu_stats.clone(),
        }
    }
}

fn create_test_object() -> Retained<NSObject> {
    let obj = NSObject::new();
    obj
}

// Mock method implementations
#[cfg(test)]
mod test_methods {
    use super::*;

    #[no_mangle]
    unsafe extern "C" fn setPhysicalCores(this: *mut AnyObject, cores: usize) {
        let value = NSNumber::new_i64(cores as i64);
        let key = NSString::from_str("physical_cores");
        msg_send![this, setAssociatedObject: &*value, forKey: &*key]
    }

    #[no_mangle]
    unsafe extern "C" fn setLogicalCores(this: *mut AnyObject, cores: usize) {
        let value = NSNumber::new_i64(cores as i64);
        let key = NSString::from_str("logical_cores");
        msg_send![this, setAssociatedObject: &*value, forKey: &*key]
    }

    #[no_mangle]
    unsafe extern "C" fn setCoreUsage(this: *mut AnyObject, usage: *const NSArray<NSNumber>) {
        let key = NSString::from_str("core_usage");
        msg_send![this, setAssociatedObject: usage, forKey: &*key]
    }

    #[no_mangle]
    unsafe extern "C" fn numberOfCores(this: *mut AnyObject) -> usize {
        let key = NSString::from_str("physical_cores");
        let value: *mut NSNumber = msg_send![this, associatedObjectForKey: &*key];
        let num: i64 = msg_send![value, integerValue];
        num as usize
    }

    #[no_mangle]
    unsafe extern "C" fn numberOfProcessorCores(this: *mut AnyObject) -> usize {
        let key = NSString::from_str("logical_cores");
        let value: *mut NSNumber = msg_send![this, associatedObjectForKey: &*key];
        let num: i64 = msg_send![value, integerValue];
        num as usize
    }

    #[no_mangle]
    unsafe extern "C" fn getCoreUsage(this: *mut AnyObject, core: usize) -> f64 {
        let key = NSString::from_str("core_usage");
        let usage_array: *mut NSArray<NSNumber> = msg_send![this, associatedObjectForKey: &*key];
        let count: usize = msg_send![usage_array, count];
        if core < count {
            let value: *mut NSNumber = msg_send![usage_array, objectAtIndex: core];
            msg_send![value, doubleValue]
        } else {
            0.0
        }
    }
}

unsafe impl Send for MockIOKit {}
unsafe impl Sync for MockIOKit {}
