use std::ffi::{c_uint, c_void, CString};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use libc::mach_port_t;
use objc2::{
    class,
    msg_send,
    rc::Retained,
    Message,
}; // Import necessary macros and types
use objc2_foundation::{NSDictionary, NSObject, NSString};

use crate::error::{Error, Result};
use crate::utils::bindings::*;

/// GPU statistics
#[derive(Debug, Clone)]
pub struct GpuStats {
    pub utilization: f64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub perf_cap: f64,
    pub perf_threshold: f64,
    pub name: String,
}

impl Default for GpuStats {
    fn default() -> Self {
        GpuStats {
            utilization: 0.0,
            memory_used: 0,
            memory_total: 0,
            perf_cap: 0.0,
            perf_threshold: 0.0,
            name: String::new(),
        }
    }
}

impl crate::utils::dictionary_access::DictionaryAccess for GpuStats {
    fn get_string(&self, key: &str) -> Option<String> {
        match key {
            "name" => Some(self.name.clone()),
            _ => None,
        }
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        match key {
            "utilization" => Some(self.utilization),
            "perf_cap" => Some(self.perf_cap),
            "perf_threshold" => Some(self.perf_threshold),
            "memory_used" => Some(self.memory_used as f64),
            "memory_total" => Some(self.memory_total as f64),
            _ => None,
        }
    }

    fn get_bool(&self, _key: &str) -> Option<bool> {
        None
    }
}

/// Mock implementation for testing
#[cfg(any(test, feature = "mock"))]
pub mod mock;

/// Fan information structure containing speed and range information
#[derive(Debug, Clone)]
pub struct FanInfo {
    /// Current fan speed in RPM
    pub speed_rpm: u32,
    /// Minimum fan speed in RPM
    pub min_speed: u32,
    /// Maximum fan speed in RPM
    pub max_speed: u32,
    /// Current fan speed as a percentage between min and max speed
    pub percentage: f64,
}

/// Thermal information structure
#[derive(Debug)]
pub struct ThermalInfo {
    /// CPU temperature in Celsius
    pub cpu_temp: f64,
    /// GPU temperature in Celsius
    pub gpu_temp: f64,
    /// Fan speed in RPM
    pub fan_speed: u32,
    /// Heatsink temperature in Celsius
    pub heatsink_temp: f64,
    /// Ambient temperature in Celsius
    pub ambient_temp: f64,
    /// Whether thermal throttling is active
    pub thermal_throttling: bool,
    /// Battery temperature in Celsius
    pub battery_temp: f64,
    /// Dictionary containing raw thermal data
    dict: ThreadSafeNSDictionary,
}

impl crate::utils::dictionary_access::DictionaryAccess for ThermalInfo {
    fn get_string(&self, key: &str) -> Option<String> {
        self.dict.get_string(key)
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        self.dict.get_number(key)
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        self.dict.get_bool(key)
    }
}

impl ThermalInfo {
    /// Create a new ThermalInfo instance from a dictionary
    pub fn new(dict: ThreadSafeNSDictionary) -> Self {
        Self {
            cpu_temp: dict.get_number("CPU_0_DIE_TEMP").unwrap_or(0.0),
            gpu_temp: dict.get_number("GPU_0_DIE_TEMP").unwrap_or(0.0),
            fan_speed: dict.get_number("FAN_0_SPEED").unwrap_or(0.0) as u32,
            heatsink_temp: dict.get_number("HS_0_TEMP").unwrap_or(0.0),
            ambient_temp: dict.get_number("AMBIENT_TEMP").unwrap_or(0.0),
            thermal_throttling: dict.get_bool("THERMAL_THROTTLING").unwrap_or(false),
            battery_temp: dict.get_number("BATTERY_TEMP").unwrap_or(0.0),
            dict,
        }
    }

    /// Get a dictionary from the thermal info with the given key
    pub fn get_dictionary(&self, key: &str) -> Option<ThreadSafeNSDictionary> {
        self.dict.get_dictionary(key)
    }

    /// Get a number from the thermal info with the given key
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.dict.get_number(key)
    }

    /// Get a string from the thermal info with the given key
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.dict.get_string(key)
    }

    /// Get a boolean from the thermal info with the given key
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.dict.get_bool(key)
    }
}

impl Clone for ThermalInfo {
    fn clone(&self) -> Self {
        Self {
            cpu_temp: self.cpu_temp,
            gpu_temp: self.gpu_temp,
            fan_speed: self.fan_speed,
            heatsink_temp: self.heatsink_temp,
            ambient_temp: self.ambient_temp,
            thermal_throttling: self.thermal_throttling,
            battery_temp: self.battery_temp,
            dict: self.dict.clone(),
        }
    }
}

/// IOKit interface for hardware monitoring
pub trait IOKit: Debug + Send + Sync {
    /// Create a matching dictionary for IOService
    fn io_service_matching(&self, name: &str) -> Result<ThreadSafeNSDictionary>;

    /// Get a service matching the given name
    fn get_service_matching(&self, name: &str) -> Result<Option<ThreadSafeAnyObject>>;

    /// Get service from matching dictionary
    fn io_service_get_matching_service(&self, matching_dict: &ThreadSafeNSDictionary) -> Result<ThreadSafeAnyObject>;

    /// Get properties for a registry entry
    fn io_registry_entry_create_cf_properties(&self, entry: &ThreadSafeAnyObject) -> Result<ThreadSafeNSDictionary>;

    /// Get CPU temperature
    fn get_cpu_temperature(&self) -> Result<f64>;

    /// Get thermal information
    fn get_thermal_info(&self) -> Result<ThermalInfo>;

    /// Get all fans
    fn get_all_fans(&self) -> Result<Vec<FanInfo>>;

    /// Check if thermal throttling is active
    fn check_thermal_throttling(&self) -> Result<bool>;

    /// Get CPU power consumption in watts
    fn get_cpu_power(&self) -> Result<f64>;

    /// Get GPU statistics
    fn get_gpu_stats(&self) -> Result<GpuStats>;

    /// Get information about a specific fan
    fn get_fan_info(&self, fan_index: u32) -> Result<FanInfo>;

    /// Get battery temperature
    fn get_battery_temperature(&self) -> Result<Option<f64>>;

    /// Get battery information
    fn get_battery_info(&self) -> Result<ThreadSafeNSDictionary>;

    /// Get CPU information
    fn get_cpu_info(&self) -> Result<ThreadSafeNSDictionary>;

    /// Get a number property from a dictionary
    fn get_number_property(&self, dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<f64>;

    /// Call an IOKit method
    fn io_connect_call_method(
        &self,
        connection: u32,
        selector: u32,
        input: &[u8],
        input_cnt: u32,
        output: &mut [u8],
        output_cnt: &mut u32,
    ) -> Result<()>;

    /// Clone this IOKit instance into a Box
    fn clone_box(&self) -> Box<dyn IOKit>;

    /// Get the parent entry of a registry entry
    fn io_registry_entry_get_parent_entry(
        &self,
        entry: &ThreadSafeAnyObject,
        plane: &str,
    ) -> Result<ThreadSafeAnyObject>;

    /// Get service properties
    fn get_service_properties(&self, service: &ThreadSafeAnyObject) -> Result<ThreadSafeNSDictionary>;
}

/// Implementation of the IOKit interface
#[derive(Clone, Debug)]
pub struct IOKitImpl;

impl Default for IOKitImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl IOKitImpl {
    pub fn new() -> Self {
        Self
    }
}

impl IOKit for IOKitImpl {
    fn io_service_matching(&self, name: &str) -> Result<ThreadSafeNSDictionary> {
        let c_str = CString::new(name).unwrap();
        let matching_dict = unsafe { IOServiceMatching(c_str.as_ptr()) };

        if matching_dict.is_null() {
            return Err(Error::iokit_error(0, "Failed to create matching dictionary"));
        }

        // Convert the raw pointer to a ThreadSafeNSDictionary
        unsafe { Ok(ThreadSafeNSDictionary::from_ptr(matching_dict as *mut _)) }
    }

    fn get_service_matching(&self, name: &str) -> Result<Option<ThreadSafeAnyObject>> {
        let dict = self.io_service_matching(name)?;
        let service = self.io_service_get_matching_service(&dict)?;
        Ok(Some(service))
    }

    fn io_registry_entry_create_cf_properties(&self, entry: &ThreadSafeAnyObject) -> Result<ThreadSafeNSDictionary> {
        let mut props: *mut c_void = std::ptr::null_mut();

        let kr = unsafe {
            IORegistryEntryCreateCFProperties(entry.raw_handle, &mut props as *mut *mut c_void, std::ptr::null_mut(), 0)
        };

        if kr != KERN_SUCCESS {
            return Err(Error::iokit_error(kr, "Failed to get properties"));
        }

        if props.is_null() {
            return Err(Error::iokit_error(0, "Properties dictionary is null"));
        }

        let dict_ptr = props as *mut NSDictionary<NSString, NSObject>;
        let dict = unsafe {
            let dict = &*dict_ptr;
            std::ptr::read(dict)
        };
        Ok(unsafe { ThreadSafeNSDictionary::with_raw_dict(dict, props) })
    }

    fn io_connect_call_method(
        &self,
        connection: u32,
        selector: u32,
        input: &[u8],
        input_cnt: u32,
        output: &mut [u8],
        output_cnt: &mut u32,
    ) -> Result<()> {
        let mut output_size = IOByteCount(*output_cnt as usize);
        let input_size = IOByteCount(input_cnt as usize);

        let result = unsafe {
            IOConnectCallStructMethod(
                connection,
                selector,
                input.as_ptr() as *const SMCKeyData_t,
                input_size,
                output.as_mut_ptr() as *mut SMCKeyData_t,
                &mut output_size,
            )
        };

        if result != KERN_SUCCESS {
            return Err(Error::iokit_error(result, "Failed to call IOKit method"));
        }

        *output_cnt =
            u32::try_from(output_size.0).map_err(|_| Error::iokit_error(0, "Invalid output size conversion"))?;
        Ok(())
    }

    fn get_cpu_temperature(&self) -> Result<f64> {
        let thermal_info = self.get_thermal_info()?;
        thermal_info
            .get_number("CPU_0_DIE_TEMP")
            .ok_or_else(|| Error::temperature_error("CPU", "Temperature not available"))
    }

    fn get_cpu_power(&self) -> Result<f64> {
        let thermal_info = self.get_thermal_info()?;
        thermal_info.get_number("CPU_Power").ok_or_else(|| Error::temperature_error("CPU", "Power not available"))
    }

    fn get_all_fans(&self) -> Result<Vec<FanInfo>> {
        let thermal_info = self.get_thermal_info()?;
        let mut fans = Vec::new();

        if let Some(fan_dict) = thermal_info.get_dictionary("Fans") {
            let fan_count = fan_dict.get_number("Count").unwrap_or(0.0) as u32;
            for i in 0..fan_count {
                if let Some(fan) = fan_dict.get_dictionary(&format!("Fan_{}", i)) {
                    fans.push(FanInfo {
                        speed_rpm: fan.get_number("CurrentSpeed").unwrap_or(0.0) as u32,
                        min_speed: fan.get_number("MinSpeed").unwrap_or(0.0) as u32,
                        max_speed: fan.get_number("MaxSpeed").unwrap_or(0.0) as u32,
                        percentage: fan.get_number("Percentage").unwrap_or(0.0),
                    });
                }
            }
        }

        Ok(fans)
    }

    fn check_thermal_throttling(&self) -> Result<bool> {
        let thermal_info = self.get_thermal_info()?;
        Ok(thermal_info.get_bool("THERMAL_THROTTLING").unwrap_or(false))
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo> {
        let matching = self.io_service_matching("IOPlatformExpertDevice")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.io_registry_entry_create_cf_properties(&service)?;

        Ok(ThermalInfo::new(props))
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
    }

    fn io_registry_entry_get_parent_entry(
        &self,
        entry: &ThreadSafeAnyObject,
        plane: &str,
    ) -> Result<ThreadSafeAnyObject> {
        if entry.raw_handle == 0 {
            return Err(Error::iokit_error(0, "Invalid entry handle"));
        }

        let plane_cstr =
            CString::new(plane).map_err(|e| Error::iokit_error(0, format!("Invalid plane name: {}", e)))?;
        let mut parent: c_uint = 0;

        let result =
            unsafe { IORegistryEntryGetParentEntry(entry.raw_handle as c_uint, plane_cstr.as_ptr(), &mut parent) };

        if result != KERN_SUCCESS {
            return Err(Error::iokit_error(result, format!("Failed to get parent entry in plane '{}'", plane)));
        }

        if parent == 0 {
            return Err(Error::iokit_error(0, format!("No parent entry found in plane '{}'", plane)));
        }

        let parent_obj = NSObject::new();

        Ok(ThreadSafeAnyObject::with_raw_handle(parent_obj, parent as mach_port_t))
    }

    fn get_service_properties(&self, service: &ThreadSafeAnyObject) -> Result<ThreadSafeNSDictionary> {
        let mut props: *mut c_void = std::ptr::null_mut();

        let result = unsafe {
            IORegistryEntryCreateCFProperties(
                service.raw_handle,
                &mut props as *mut *mut c_void,
                std::ptr::null_mut(),
                0,
            )
        };

        if result != KERN_SUCCESS {
            return Err(Error::iokit_error(result, "Failed to get service properties"));
        }

        let dict_ptr = props as *mut NSDictionary<NSString, NSObject>;
        let dict = unsafe {
            let dict = &*dict_ptr;
            std::ptr::read(dict)
        };
        Ok(unsafe { ThreadSafeNSDictionary::with_raw_dict(dict, props) })
    }

    fn io_service_get_matching_service(&self, matching_dict: &ThreadSafeNSDictionary) -> Result<ThreadSafeAnyObject> {
        let service = unsafe { IOServiceGetMatchingService(IOMASTER_PORT_DEFAULT, matching_dict.raw_dict) };
        if service == 0 {
            return Err(Error::iokit_error(0, "Failed to get matching service"));
        }

        // Create a ThreadSafeAnyObject with the raw handle
        let service_obj = NSObject::new();
        Ok(ThreadSafeAnyObject::with_raw_handle(service_obj, service))
    }

    fn get_gpu_stats(&self) -> Result<GpuStats> {
        let stats = GpuStats::default();

        let result = KERN_SUCCESS;

        if result != KERN_SUCCESS {
            return Err(Error::iokit_error(result, "Failed to get GPU stats"));
        }

        Ok(stats)
    }

    fn get_fan_info(&self, fan_index: u32) -> Result<FanInfo> {
        // Get all fans and return the one at the specified index
        let fans = self.get_all_fans()?;
        fans.get(fan_index as usize)
            .cloned()
            .ok_or_else(|| Error::iokit_error(-1, format!("Fan index out of bounds: {}", fan_index)))
    }

    fn get_battery_temperature(&self) -> Result<Option<f64>> {
        let thermal_info = self.get_thermal_info()?;
        Ok(Some(thermal_info.battery_temp))
    }

    fn get_battery_info(&self) -> Result<ThreadSafeNSDictionary> {
        let result = KERN_SUCCESS;

        if result != KERN_SUCCESS {
            return Err(Error::iokit_error(result, "Failed to get battery info"));
        }

        Ok(ThreadSafeNSDictionary::empty()) // Placeholder for actual implementation
    }

    fn get_cpu_info(&self) -> Result<ThreadSafeNSDictionary> {
        // Implementation for real IOKit
        // This would need to query CPU metrics from the system
        Err(Error::iokit_error(0, "CPU info not implemented for this platform"))
    }

    fn get_number_property(&self, dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<f64> {
        let key = NSString::from_str(key);
        let obj: *const NSObject = unsafe { msg_send![dict, objectForKey:std::convert::AsRef::<NSString>::as_ref(&key)] };
        if obj.is_null() {
            return None;
        }

            unsafe {
            let obj: *const NSObject = msg_send![dict, objectForKey:&*key];
            if obj.is_null() {
                return None;
            }

            let is_number: bool = msg_send![obj, isKindOfClass: class!(NSNumber)];
            if is_number {
                let value: f64 = msg_send![obj, doubleValue];
                Some(value)
            } else {
                None
            }
        }
    }
}

/// A thread-safe wrapper for AnyObject that can be shared between threads.
#[derive(Debug)]
pub struct ThreadSafeAnyObject {
    obj: Arc<Mutex<Retained<NSObject>>>,
    raw_handle: mach_port_t,
}

impl ThreadSafeAnyObject {
    /// Creates a new thread-safe wrapper for an AnyObject.
    pub fn new(obj: Retained<NSObject>) -> Self {
        Self { obj: Arc::new(Mutex::new(obj)), raw_handle: 0 }
    }

    /// Creates a new thread-safe wrapper for an AnyObject with a raw handle.
    pub fn with_raw_handle(obj: Retained<NSObject>, raw_handle: mach_port_t) -> Self {
        Self { obj: Arc::new(Mutex::new(obj)), raw_handle }
    }

    /// Gets a reference to the inner AnyObject.
    pub fn inner(&self) -> Retained<NSObject> {
        self.obj.lock().expect("Failed to lock mutex").clone()
    }
}

// Implement Send and Sync for ThreadSafeAnyObject since we're using Arc for thread safety
unsafe impl Send for ThreadSafeAnyObject {}
unsafe impl Sync for ThreadSafeAnyObject {}

impl Clone for ThreadSafeAnyObject {
    fn clone(&self) -> Self {
        Self { obj: Arc::clone(&self.obj), raw_handle: self.raw_handle }
    }
}

/// A thread-safe wrapper around NSDictionary<NSString, NSObject>
#[derive(Debug)]
pub struct ThreadSafeNSDictionary {
    inner: Arc<Mutex<NSDictionary<NSString, NSObject>>>,
    raw_dict: *mut c_void,
}

impl ThreadSafeNSDictionary {
    pub fn empty() -> Self {
        // Create an empty dictionary
        let empty_dict = NSDictionary::new();

        // Since NSDictionary::new() returns a Retained<NSDictionary>,
        // we need to extract the NSDictionary from it
        let dict = copy_nsdictionary(&empty_dict);
        Self { inner: Arc::new(Mutex::new(dict)), raw_dict: std::ptr::null_mut() }
    }

    /// Creates a new ThreadSafeNSDictionary from a raw pointer.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that:
    /// - The pointer is valid and properly aligned
    /// - The pointer points to a valid NSDictionary<NSString, NSObject>
    /// - The dictionary is not mutated while this ThreadSafeNSDictionary exists
    pub unsafe fn from_ptr(dict_ptr: *mut NSDictionary<NSString, NSObject>) -> Self {
        let dict = if dict_ptr.is_null() {
            // Since NSDictionary::new() returns a Retained<NSDictionary>,
            // we need to extract the NSDictionary from it
            let empty_dict = NSDictionary::new();
            copy_nsdictionary(&empty_dict)
        } else {
            unsafe {
                let dict = &*dict_ptr;
                std::ptr::read(dict)
            }
        };
        Self { inner: Arc::new(Mutex::new(dict)), raw_dict: dict_ptr as *mut c_void }
    }

    /// Creates a new ThreadSafeNSDictionary from a dictionary and its raw pointer.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that:
    /// - The dictionary is valid and matches the pointer
    /// - The pointer is either null or points to the same dictionary
    /// - The dictionary is not mutated while this ThreadSafeNSDictionary exists
    pub unsafe fn with_raw_dict(dict: NSDictionary<NSString, NSObject>, dict_ptr: *mut c_void) -> Self {
        if dict_ptr.is_null() {
            // Create an empty dictionary using from_retained_objects with empty arrays
            let retained = NSDictionary::from_retained_objects::<NSString>(&[], &[]);
            let empty_dict = copy_nsdictionary(&retained);
            ThreadSafeNSDictionary {
                inner: Arc::new(Mutex::new(empty_dict)),
                raw_dict: std::ptr::null_mut()
            }
        } else {
            ThreadSafeNSDictionary {
                inner: Arc::new(Mutex::new(dict)),
                raw_dict: dict_ptr
            }
        }
    }

    pub fn get_ref(&self) -> NSDictionary<NSString, NSObject> {
        let retained = self.inner.lock().expect("Failed to lock mutex");
        copy_nsdictionary(&retained)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        let dict = self.get_ref();
        <crate::utils::property_utils::PropertyAccessor as crate::utils::property_utils::PropertyUtils>::get_string_property(&dict, key)
    }

    pub fn get_number(&self, key: &str) -> Option<f64> {
        let dict = self.get_ref();
        <crate::utils::property_utils::PropertyAccessor as crate::utils::property_utils::PropertyUtils>::get_number_property(&dict, key)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        let dict = self.get_ref();
        <crate::utils::property_utils::PropertyAccessor as crate::utils::property_utils::PropertyUtils>::get_bool_property(&dict, key)
    }

    pub fn get_number_as_f32(&self, key: &str) -> Option<f32> {
        self.get_number(key).map(|n| n as f32)
    }

    pub fn get_dictionary(&self, key: &str) -> Option<ThreadSafeNSDictionary> {
        let dict = self.get_ref();
        let key_str = NSString::from_str(key);
        unsafe {
            let dict_ptr: *const NSObject = msg_send![&dict, objectForKey:std::convert::AsRef::<NSString>::as_ref(&key_str)];
            if dict_ptr.is_null() {
                None
            } else {
                let dict = copy_nsdictionary(&*(dict_ptr as *const NSDictionary<NSString, NSObject>));
                Some(ThreadSafeNSDictionary::with_raw_dict(dict, dict_ptr as *mut c_void))
            }
        }
    }
}

// Implement Send and Sync since we're using Arc for thread safety
unsafe impl Send for ThreadSafeNSDictionary {}
unsafe impl Sync for ThreadSafeNSDictionary {}

impl Clone for ThreadSafeNSDictionary {
    fn clone(&self) -> Self {
        let dict = self.get_ref();
        Self { inner: Arc::new(Mutex::new(dict)), raw_dict: self.raw_dict }
    }
}

impl crate::utils::dictionary_access::DictionaryAccess for ThreadSafeNSDictionary {
    fn get_string(&self, key: &str) -> Option<String> {
        self.get_string(key)
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        self.get_number(key)
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_bool(key)
    }
}

/// Helper function to copy an NSDictionary
fn copy_nsdictionary(dict: &NSDictionary<NSString, NSObject>) -> NSDictionary<NSString, NSObject> {
    let copied_ptr: *mut NSObject = unsafe { msg_send![dict, copy] };
    let copied_dict_ptr = copied_ptr as *mut NSDictionary<NSString, NSObject>;
    if copied_dict_ptr.is_null() {
        panic!("Failed to copy dictionary");
    }
    let dict = unsafe {
        let dict = &*copied_dict_ptr;
        std::ptr::read(dict)
    };
    dict
}

// Helper function for safe numeric conversions
fn convert_number<T, U>(value: T) -> Result<U>
where
    T: TryInto<U>,
    T::Error: std::fmt::Display,
{
    value.try_into().map_err(|e| Error::invalid_argument("numeric conversion failed", Some(e.to_string())))
}

impl Clone for Box<dyn IOKit> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// Removed manual implementation of Debug for dyn IOKit

pub const K_IOMASTER_PORT_DEFAULT: mach_port_t = 0; // Define this constant if not already defined
pub const IOMASTER_PORT_DEFAULT: mach_port_t = K_IOMASTER_PORT_DEFAULT;
