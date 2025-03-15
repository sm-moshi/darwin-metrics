//! This module is only used to make the automock-generated MockIOKit available to tests in other modules.

use std::sync::Arc;
use objc2::{
    class,
    declare::ClassBuilder,
    runtime::{AnyClass, AnyObject, Sel},
    ClassType, msg_send, sel,
    rc::Retained,
};
use objc2_foundation::{NSArray, NSNumber, NSObject, NSString};

use crate::{
    error::{Error, Result},
    hardware::iokit::{
            DictionaryAccess, FanInfo, GpuStats, IOKit, NSObject as _, ThreadSafeAnyObject,
            ThermalInfo,
        },
    utils::safe_dictionary::SafeDictionary,
};

use std::fmt::Debug;
use std::ffi::CString;

// Type aliases for method implementations with proper HRTBs
type PhysicalCoresImpl = for<'a> extern "C" fn(&'a NSObject, Sel, usize);
type LogicalCoresImpl = for<'a> extern "C" fn(&'a NSObject, Sel, usize);
type CoreUsageImpl = for<'a, 'b> extern "C" fn(&'a NSObject, Sel, &'b NSArray<NSNumber>);
type NumberOfCoresImpl = for<'a> extern "C" fn(&'a NSObject, Sel) -> usize;
type GetCoreUsageImpl = for<'a> extern "C" fn(&'a NSObject, Sel, usize) -> f64;

// Method implementations
extern "C" fn set_physical_cores_impl<'a>(this: &'a NSObject, _cmd: Sel, cores: usize) {
    let value = NSNumber::new_i64(cores as i64);
    let key = NSString::from_str("physical_cores");
    unsafe {
        let _: () = msg_send![this, setObject: &*value, forKey: &*key];
    }
}

extern "C" fn set_logical_cores_impl<'a>(this: &'a NSObject, _cmd: Sel, cores: usize) {
    let value = NSNumber::new_i64(cores as i64);
    let key = NSString::from_str("logical_cores");
    unsafe {
        let _: () = msg_send![this, setObject: &*value, forKey: &*key];
    }
}

extern "C" fn set_core_usage_impl<'a, 'b>(this: &'a NSObject, _cmd: Sel, usage: &'b NSArray<NSNumber>) {
    let key = NSString::from_str("core_usage");
    unsafe {
        let _: () = msg_send![this, setObject: usage, forKey: &*key];
    }
}

extern "C" fn number_of_cores_impl<'a>(this: &'a NSObject, _cmd: Sel) -> usize {
    let key = NSString::from_str("physical_cores");
    unsafe {
        let value: Option<&NSNumber> = msg_send![this, objectForKey: &*key];
        value.map_or(0, |num| {
            let val: i64 = msg_send![num, integerValue];
            val as usize
        })
    }
}

extern "C" fn number_of_processor_cores_impl<'a>(this: &'a NSObject, _cmd: Sel) -> usize {
    let key = NSString::from_str("logical_cores");
    unsafe {
        let value: Option<&NSNumber> = msg_send![this, objectForKey: &*key];
        value.map_or(0, |num| {
            let val: i64 = msg_send![num, integerValue];
            val as usize
        })
    }
}

extern "C" fn get_core_usage_impl<'a>(this: &'a NSObject, _cmd: Sel, core: usize) -> f64 {
    let key = NSString::from_str("core_usage");
    unsafe {
        let usage_array: Option<&NSArray<NSNumber>> = msg_send![this, objectForKey: &*key];
        if let Some(array) = usage_array {
            let count: usize = msg_send![array, count];
            if core < count {
                let value: Option<&NSNumber> = msg_send![array, objectAtIndex: core];
                if let Some(num) = value {
                    return msg_send![num, doubleValue];
                }
            }
        }
        0.0
    }
}

#[derive(Debug)]
pub struct MockIOKit {
    obj: ThreadSafeAnyObject,
}

impl MockIOKit {
    pub fn new() -> Result<Self> {
        let instance = Self::create_instance()?;
        Ok(Self {
            obj: ThreadSafeAnyObject::new(instance),
        })
    }

    pub fn with_physical_cores(self, cores: usize) -> Result<Self> {
        self.set_physical_cores(cores);
        Ok(self)
    }

    pub fn with_logical_cores(self, cores: usize) -> Result<Self> {
        self.set_logical_cores(cores);
        Ok(self)
    }

    pub fn with_core_usage(self, usage: Vec<f64>) -> Result<Self> {
        self.set_core_usage(usage);
        Ok(self)
    }

    pub fn with_temperature(self, temp: f64) -> Result<Self> {
        let mut info = self.get_thermal_info()?;
        info.cpu_temp = temp;
        Ok(self)
    }

    pub fn with_battery_info(
        self,
        is_present: bool,
        is_charging: bool,
        cycle_count: i64,
        percentage: f64,
        temperature: f64,
        time_remaining: i64,
        design_capacity: f64,
        current_capacity: f64,
    ) -> Result<Self> {
        let mut dict = SafeDictionary::new();
        dict.set_bool("present", is_present);
        dict.set_bool("is_charging", is_charging);
        dict.set_i64("cycle_count", cycle_count);
        dict.set_f64("percentage", percentage);
        dict.set_f64("temperature", temperature);
        dict.set_i64("time_remaining", time_remaining);
        dict.set_f64("design_capacity", design_capacity);
        dict.set_f64("current_capacity", current_capacity);
        Ok(self)
    }

    fn create_instance() -> Result<Retained<NSObject>> {
        let class_name = CString::new("MockIOKit").map_err(|e| Error::io_error("Failed to create class name", e.into()))?;
        let mut builder = ClassBuilder::new(&class_name, NSObject::class())
            .ok_or_else(|| Error::iokit_error(0, "Failed to create class builder"))?;

        unsafe {
            builder.add_method(
                sel!(setPhysicalCores:),
                set_physical_cores_impl as extern "C" fn(&NSObject, Sel, usize),
            );

            builder.add_method(
                sel!(setLogicalCores:),
                set_logical_cores_impl as extern "C" fn(&NSObject, Sel, usize),
            );

            builder.add_method(
                sel!(setCoreUsage:),
                set_core_usage_impl as extern "C" fn(&NSObject, Sel, &NSArray<NSNumber>),
            );

            builder.add_method(
                sel!(numberOfCores),
                number_of_cores_impl as extern "C" fn(&NSObject, Sel) -> usize,
            );

            builder.add_method(
                sel!(numberOfProcessorCores),
                number_of_processor_cores_impl as extern "C" fn(&NSObject, Sel) -> usize,
            );

            builder.add_method(
                sel!(getCoreUsage:),
                get_core_usage_impl as extern "C" fn(&NSObject, Sel, usize) -> f64,
            );

            let class = builder.register();
            let instance: Retained<NSObject> = msg_send![class, new];
            Ok(instance)
        }
    }

    pub fn set_physical_cores(&self, cores: usize) {
        unsafe {
            let _: () = msg_send![&*self.obj.obj.lock().unwrap(), setPhysicalCores: cores];
        }
    }

    pub fn set_logical_cores(&self, cores: usize) {
        unsafe {
            let _: () = msg_send![&*self.obj.obj.lock().unwrap(), setLogicalCores: cores];
        }
    }

    pub fn set_core_usage(&self, usage: Vec<f64>) -> Result<()> {
        let numbers: Vec<Retained<NSNumber>> = usage.iter().map(|&u| NSNumber::new_f64(u)).collect();
        let refs: Vec<&NSNumber> = numbers.iter().map(|n| n.as_ref()).collect();
        let array = NSArray::<NSNumber>::from_slice(&refs);
        unsafe {
            let _: () = msg_send![&*self.obj.obj.lock().unwrap(), setCoreUsage: &*array];
            Ok(())
        }
    }

    pub fn get_physical_cores(&self) -> usize {
        unsafe {
            let result: usize = msg_send![&*self.obj.obj.lock().unwrap(), numberOfCores];
            result
        }
    }

    pub fn get_logical_cores(&self) -> usize {
        unsafe {
            let result: usize = msg_send![&*self.obj.obj.lock().unwrap(), numberOfProcessorCores];
            result
        }
    }

    pub fn get_core_usage(&self, core: usize) -> f64 {
        unsafe {
            let result: f64 = msg_send![&*self.obj.obj.lock().unwrap(), getCoreUsage: core];
            result
        }
    }
}

impl DictionaryAccess for MockIOKit {
    fn get_dictionary(&self, _key: &str) -> Option<SafeDictionary> {
        None // Mock implementation
    }
}

impl IOKit for MockIOKit {
    fn io_service_matching(&self, _name: &str) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn io_service_get_matching_service(&self, _matching_dict: &SafeDictionary) -> Result<ThreadSafeAnyObject> {
        Ok(ThreadSafeAnyObject::new(NSObject::new()))
    }

    fn io_registry_entry_create_cf_properties(&self, _entry: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_service_properties(&self, _service: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_battery_info(&self) -> Result<SafeDictionary> {
        let mut dict = SafeDictionary::new();
        dict.set_bool("present", false);
        dict.set_bool("is_charging", false);
        dict.set_i64("cycle_count", 0);
        dict.set_f64("percentage", 0.0);
        dict.set_f64("temperature", 0.0);
        dict.set_f64("design_capacity", 0.0);
        dict.set_f64("current_capacity", 0.0);
        Ok(dict)
    }

    fn get_cpu_info(&self) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo> {
        Ok(ThermalInfo {
            cpu_temp: 0.0,
            gpu_temp: Some(0.0),
            fan_speed: 0,
            heatsink_temp: Some(0.0),
            ambient_temp: Some(0.0),
            battery_temp: Some(0.0),
            thermal_throttling: false,
            dict: SafeDictionary::new(),
        })
    }

    fn get_all_fans(&self) -> Result<Vec<FanInfo>> {
        Ok(vec![])
    }

    fn check_thermal_throttling(&self, _plane: &str) -> Result<bool> {
        Ok(false)
    }

    fn get_cpu_power(&self) -> Result<f64> {
        Ok(0.0)
    }

    fn get_gpu_stats(&self) -> Result<GpuStats> {
        Ok(GpuStats {
            utilization: 0.0,
            memory_used: 0,
            memory_total: 0,
            perf_cap: 0.0,
            perf_threshold: 0.0,
            name: String::new(),
        })
    }

    fn get_fan_info(&self, _fan_index: u32) -> Result<FanInfo> {
        Ok(FanInfo { speed_rpm: 0, min_speed: 0, max_speed: 0, percentage: 0.0 })
    }

    fn get_battery_temperature(&self) -> Result<Option<f64>> {
        Ok(None)
    }

    fn get_number_property(&self, _dict: &SafeDictionary, _key: &str) -> Result<f64> {
        Ok(0.0)
    }

    fn io_connect_call_method(
        &self,
        _connection: u32,
        _selector: u32,
        _input: &[u64],
        _output: &mut [u64],
    ) -> Result<()> {
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
    }

    fn io_registry_entry_get_parent_entry(&self, _entry: &ThreadSafeAnyObject) -> Result<ThreadSafeAnyObject> {
        Ok(ThreadSafeAnyObject::new(NSObject::new()))
    }

    fn get_cpu_temperature(&self, _plane: &str) -> Result<f64> {
        Ok(0.0)
    }

    fn get_service_matching(&self, _name: &str) -> Result<Option<ThreadSafeAnyObject>> {
        Ok(Some(self.obj.clone()))
    }

    fn get_physical_cores(&self) -> Result<usize> {
        Ok(self.get_physical_cores())
    }

    fn get_logical_cores(&self) -> Result<usize> {
        Ok(self.get_logical_cores())
    }

    fn get_core_usage(&self) -> Result<Vec<f64>> {
        let physical_cores = self.get_physical_cores();
        let mut usage = Vec::with_capacity(physical_cores);
        for i in 0..physical_cores {
            usage.push(self.get_core_usage(i));
        }
        Ok(usage)
    }
}

impl Clone for MockIOKit {
    fn clone(&self) -> Self {
        Self {
            obj: self.obj.clone(),
        }
    }
}

fn create_test_object() -> Retained<NSObject> {
    let obj = NSObject::new();
    obj
}

unsafe impl Send for MockIOKit {}
unsafe impl Sync for MockIOKit {}
