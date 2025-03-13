use std::ffi::{CString, c_void, c_uint};
use std::ptr;
use std::sync::Arc;
use std::fmt::Debug;

use objc2::{
    class,
    msg_send,
    rc::{Retained, autoreleasepool},
    // runtime::{AnyClass, AnyObject},
    // ClassType,
};
use objc2_foundation::{
    NSDictionary, NSObject, NSString,
}; // NSArray, NSNumber
use libc::mach_port_t;
use parking_lot::Mutex as ParkingLotMutex;

use crate::error::{Error, Result};
use crate::utils::bindings::*;
use crate::utils::dictionary_access::DictionaryAccess;

// Define constants that are missing from bindings
pub const kIOMasterPortDefault: mach_port_t = 0;
pub const IOMASTER_PORT_DEFAULT: mach_port_t = kIOMasterPortDefault;
pub const CF_ALLOCATOR_DEFAULT: *const c_void = ptr::null();

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

impl DictionaryAccess for ThermalInfo {
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.dict.get_bool(key)
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        self.dict.get_number(key)
    }

    fn get_string(&self, key: &str) -> Option<String> {
        self.dict.get_string(key)
    }
}

impl ThermalInfo {
    /// Create a new ThermalInfo instance from a dictionary
    pub fn new(dict: ThreadSafeNSDictionary) -> Self {
        Self {
            cpu_temp: dict.get_number("CPU_0_DIE_TEMP").unwrap_or(0.0),
            gpu_temp: dict.get_number("GPU_0_DIE_TEMP").unwrap_or(0.0),
            fan_speed: dict.get_number("FAN_0_SPEED").unwrap_or(0.0) as u32,
            heatsink_temp: dict.get_number("HEAT_SINK_TEMP").unwrap_or(0.0),
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

    /// Call an IOKit method with structured input/output
    fn io_connect_call_method(
        &self,
        connection: u32,
        selector: u32,
        input: &[u8],
        input_cnt: u32,
        output: &mut [u8],
        output_cnt: &mut u32,
    ) -> Result<()>;

    /// Get CPU temperature
    fn get_cpu_temperature(&self) -> Result<f64>;

    /// Get CPU power consumption in watts
    fn get_cpu_power(&self) -> Result<f64>;

    /// Get all system fans information
    fn get_all_fans(&self) -> Result<Vec<FanInfo>>;

    /// Check if thermal throttling is active
    fn check_thermal_throttling(&self) -> Result<bool>;

    /// Get thermal information
    fn get_thermal_info(&self) -> Result<ThermalInfo>;

    /// Get service properties
    fn get_service_properties(&self, service: &ThreadSafeAnyObject) -> Result<ThreadSafeNSDictionary> {
        self.io_registry_entry_create_cf_properties(service)
    }

    /// Get battery temperature
    fn get_battery_temperature(&self) -> Result<Option<f64>> {
        let thermal_info = self.get_thermal_info()?;
        Ok(Some(thermal_info.battery_temp))
    }

    /// Get a number property from a dictionary
    fn get_number_property(&self, dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<f64> {
        let key = NSString::from_str(key);
        unsafe { 
            let obj: *const NSObject = msg_send![dict, objectForKey:&*key];
            if obj.is_null() {
                return None;
            }
            
            let is_number: bool = msg_send![obj, isKindOfClass: class!(NSNumber)];
            if is_number {
                Some(msg_send![obj, doubleValue])
            } else {
                None
            }
        }
    }

    /// Clone the IOKit implementation
    fn clone_box(&self) -> Box<dyn IOKit>;

    /// Get the parent entry of a registry entry
    fn io_registry_entry_get_parent_entry(&self, entry: &ThreadSafeAnyObject, plane: &str) -> Result<ThreadSafeAnyObject> {
        if entry.raw_handle == 0 {
            return Err(Error::iokit_error(0, "Invalid entry handle"));
        }
        
        let plane_cstr = CString::new(plane).map_err(|e| Error::iokit_error(0, format!("Invalid plane name: {}", e)))?;
        let mut parent: c_uint = 0;
        
        let result = unsafe {
            IORegistryEntryGetParentEntry(
                entry.raw_handle as c_uint,
                plane_cstr.as_ptr(),
                &mut parent,
            )
        };
        
        if result != KERN_SUCCESS {
            return Err(Error::iokit_error(result, format!("Failed to get parent entry in plane '{}'", plane)));
        }
        
        if parent == 0 {
            return Err(Error::iokit_error(0, format!("No parent entry found in plane '{}'", plane)));
        }
        
        let parent_obj = autoreleasepool(|_| {
            // Create a placeholder NSObject
            let obj = NSObject::new();
            obj
        });
        
        Ok(ThreadSafeAnyObject::with_raw_handle(parent_obj, parent as mach_port_t))
    }
}

/// Implementation of the IOKit interface
#[derive(Clone, Debug)]
pub struct IOKitImpl;

impl IOKitImpl {
    pub fn new() -> Self {
        Self
    }
}

impl IOKit for IOKitImpl {
    fn io_service_matching(&self, name: &str) -> Result<ThreadSafeNSDictionary> {
        unsafe {
            // Convert Rust string to C string
            let name_cstr = CString::new(name).map_err(|_| Error::iokit_error(0, "Invalid service name"))?;
            
            // Create matching dictionary
            let matching_dict = IOServiceMatching(name_cstr.as_ptr());
            if matching_dict.is_null() {
                return Err(Error::iokit_error(0, "Failed to create matching dictionary"));
            }
            
            // Convert to NSDictionary
            let dict_ptr = matching_dict as *mut NSDictionary<NSString, NSObject>;
            // Check if the pointer is valid
            if dict_ptr.is_null() {
                return Err(Error::iokit_error(0, "Failed to create NSDictionary from pointer"));
            }
            
            // Create a copy of the dictionary to own it
            let dict = unsafe { copy_nsdictionary(&*dict_ptr) };
            
            Ok(ThreadSafeNSDictionary::with_raw_dict(dict, matching_dict))
        }
    }

    fn get_service_matching(&self, name: &str) -> Result<Option<ThreadSafeAnyObject>> {
        let dict = self.io_service_matching(name)?;
        let service = self.io_service_get_matching_service(&dict)?;
        Ok(Some(service))
    }

    fn io_registry_entry_create_cf_properties(&self, entry: &ThreadSafeAnyObject) -> Result<ThreadSafeNSDictionary> {
        let mut props: *mut c_void = std::ptr::null_mut();
        
        let kr = unsafe {
            IORegistryEntryCreateCFProperties(
                entry.raw_handle,
                &mut props as *mut *mut c_void,
                std::ptr::null_mut(),  // Use null instead of kCFAllocatorDefault
                0,
            )
        };
        
        if kr != KERN_SUCCESS {
            return Err(Error::iokit_error(kr, "Failed to get properties"));
        }
        
        if props.is_null() {
            return Err(Error::iokit_error(0, "Properties dictionary is null"));
        }
        
        let dict = autoreleasepool(|_| {
            let dict_ptr = props as *mut NSDictionary<NSString, NSObject>;
            if dict_ptr.is_null() {
                panic!("Failed to create NSDictionary from pointer");
            }
            // Create a new NSDictionary from the pointer
            // We need to copy the dictionary to own it
            let dict = unsafe { copy_nsdictionary(&*dict_ptr) };
            dict
        });
        
        Ok(ThreadSafeNSDictionary::with_raw_dict(dict, props))
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

        *output_cnt = u32::try_from(output_size.0)
            .map_err(|_| Error::iokit_error(0, "Invalid output size conversion"))?;
        Ok(())
    }

    fn get_cpu_temperature(&self) -> Result<f64> {
        let thermal_info = self.get_thermal_info()?;
        thermal_info.get_number("CPU_0_DIE_TEMP")
            .ok_or_else(|| Error::temperature_error("CPU", "Temperature not available"))
    }

    fn get_cpu_power(&self) -> Result<f64> {
        let thermal_info = self.get_thermal_info()?;
        thermal_info.get_number("CPU_Power")
            .ok_or_else(|| Error::temperature_error("CPU", "Power not available"))
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
        Box::new(Self::new())
    }

    fn io_registry_entry_get_parent_entry(&self, entry: &ThreadSafeAnyObject, plane: &str) -> Result<ThreadSafeAnyObject> {
        if entry.raw_handle == 0 {
            return Err(Error::iokit_error(0, "Invalid entry handle"));
        }
        
        let plane_cstr = CString::new(plane).map_err(|e| Error::iokit_error(0, format!("Invalid plane name: {}", e)))?;
        let mut parent: c_uint = 0;
        
        let result = unsafe {
            IORegistryEntryGetParentEntry(
                entry.raw_handle as c_uint,
                plane_cstr.as_ptr(),
                &mut parent,
            )
        };
        
        if result != KERN_SUCCESS {
            return Err(Error::iokit_error(result, format!("Failed to get parent entry in plane '{}'", plane)));
        }
        
        if parent == 0 {
            return Err(Error::iokit_error(0, format!("No parent entry found in plane '{}'", plane)));
        }
        
        let parent_obj = autoreleasepool(|_| {
            // Create a placeholder NSObject
            let obj = NSObject::new();
            obj
        });
        
        Ok(ThreadSafeAnyObject::with_raw_handle(parent_obj, parent as mach_port_t))
    }

    fn io_service_get_matching_service(&self, matching_dict: &ThreadSafeNSDictionary) -> Result<ThreadSafeAnyObject> {
        unsafe {
            // We need to use the raw_dict directly as it's the CFDictionaryRef expected by IOServiceGetMatchingService
            let service = IOServiceGetMatchingService(IOMASTER_PORT_DEFAULT, matching_dict.raw_dict);
            if service == 0 {
                return Err(Error::iokit_error(0, "Failed to get matching service"));
            }
            
            // Create a ThreadSafeAnyObject with the raw handle
            let service_obj = NSObject::new();
            Ok(ThreadSafeAnyObject::with_raw_handle(service_obj, service))
        }
    }
}

/// A thread-safe wrapper for AnyObject that can be shared between threads.
#[derive(Debug)]
pub struct ThreadSafeAnyObject {
    obj: Arc<ParkingLotMutex<Retained<NSObject>>>,
    raw_handle: mach_port_t,
}

impl ThreadSafeAnyObject {
    /// Creates a new thread-safe wrapper for an AnyObject.
    pub fn new(obj: Retained<NSObject>) -> Self {
        Self {
            obj: Arc::new(ParkingLotMutex::new(obj)),
            raw_handle: 0,
        }
    }

    /// Creates a new thread-safe wrapper for an AnyObject with a raw handle.
    pub fn with_raw_handle(obj: Retained<NSObject>, raw_handle: mach_port_t) -> Self {
        Self {
            obj: Arc::new(ParkingLotMutex::new(obj)),
            raw_handle,
        }
    }

    /// Gets a reference to the inner AnyObject.
    pub fn inner(&self) -> Retained<NSObject> {
        self.obj.lock().clone()
    }
}

// Implement Send and Sync for ThreadSafeAnyObject since we're using Arc for thread safety
unsafe impl Send for ThreadSafeAnyObject {}
unsafe impl Sync for ThreadSafeAnyObject {}

impl Clone for ThreadSafeAnyObject {
    fn clone(&self) -> Self {
        Self {
            obj: Arc::clone(&self.obj),
            raw_handle: self.raw_handle,
        }
    }
}

/// A thread-safe wrapper around NSDictionary<NSString, NSObject>
#[derive(Debug)]
pub struct ThreadSafeNSDictionary {
    inner: Arc<ParkingLotMutex<NSDictionary<NSString, NSObject>>>,
    raw_dict: *mut c_void,
}

impl ThreadSafeNSDictionary {
    pub fn empty() -> Self {
        // Create an empty dictionary
        let empty_dict = NSDictionary::new();
        
        // Since NSDictionary::new() returns a Retained<NSDictionary>,
        // we need to extract the NSDictionary from it
        let dict = unsafe { copy_nsdictionary(&*empty_dict) };
        
        Self {
            inner: Arc::new(ParkingLotMutex::new(dict)),
            raw_dict: std::ptr::null_mut(),
        }
    }

    pub fn with_raw_dict(dict: NSDictionary<NSString, NSObject>, raw_dict: *mut c_void) -> Self {
        Self {
            inner: Arc::new(ParkingLotMutex::new(dict)),
            raw_dict,
        }
    }

    pub fn from_ptr(dict_ptr: *mut NSDictionary<NSString, NSObject>) -> Self {
        unsafe {
            // Check if the pointer is valid
            if dict_ptr.is_null() {
                panic!("Failed to create NSDictionary from pointer");
            }
            
            // Create a copy of the dictionary to own it
            let dict = unsafe { copy_nsdictionary(&*dict_ptr) };
            
            Self {
                inner: Arc::new(ParkingLotMutex::new(dict)),
                raw_dict: dict_ptr as *mut c_void,
            }
        }
    }

    pub fn inner(&self) -> NSDictionary<NSString, NSObject> {
        let guard = self.inner.lock();
        unsafe { copy_nsdictionary(&*guard) }
    }

    pub fn get_dictionary(&self, key: &str) -> Option<ThreadSafeNSDictionary> {
        let dict = self.inner.lock();
        let key_str = NSString::from_str(key);
        unsafe {
            let obj: *const NSObject = msg_send![&*dict, objectForKey:&*key_str];
            if obj.is_null() {
                return None;
            }
            
            let is_dict: bool = msg_send![obj, isKindOfClass: class!(NSDictionary)];
            if is_dict {
                let dict_ptr = obj as *const NSDictionary<NSString, NSObject>;
                
                // Create a copy of the dictionary to own it
                let owned_dict = unsafe { copy_nsdictionary(&*dict_ptr) };
                
                Some(ThreadSafeNSDictionary::with_raw_dict(owned_dict, obj as *mut c_void))
            } else {
                None
            }
        }
    }
}

// Implement Send and Sync since we're using Arc for thread safety
unsafe impl Send for ThreadSafeNSDictionary {}
unsafe impl Sync for ThreadSafeNSDictionary {}

impl Clone for ThreadSafeNSDictionary {
    fn clone(&self) -> Self {
        let guard = self.inner.lock();
        let copied_dict = unsafe { copy_nsdictionary(&*guard) };
        
        Self {
            inner: Arc::new(ParkingLotMutex::new(copied_dict)),
            raw_dict: self.raw_dict,
        }
    }
}

impl DictionaryAccess for ThreadSafeNSDictionary {
    fn get_bool(&self, key: &str) -> Option<bool> {
        let dict = self.inner.lock();
        let key_str = NSString::from_str(key);
        unsafe {
            let obj: *const NSObject = msg_send![&*dict, objectForKey:&*key_str];
            if obj.is_null() {
                return None;
            }
            
            let is_number: bool = msg_send![obj, isKindOfClass: class!(NSNumber)];
            if is_number {
                Some(msg_send![obj, boolValue])
            } else {
                None
            }
        }
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        let dict = self.inner.lock();
        let key_str = NSString::from_str(key);
        unsafe {
            let obj: *const NSObject = msg_send![&*dict, objectForKey:&*key_str];
            if obj.is_null() {
                return None;
            }
            
            let is_number: bool = msg_send![obj, isKindOfClass: class!(NSNumber)];
            if is_number {
                Some(msg_send![obj, doubleValue])
            } else {
                None
            }
        }
    }

    fn get_string(&self, key: &str) -> Option<String> {
        let dict = self.inner.lock();
        let key_str = NSString::from_str(key);
        unsafe {
            let obj: *const NSObject = msg_send![&*dict, objectForKey:&*key_str];
            if obj.is_null() {
                return None;
            }
            
            let is_string: bool = msg_send![obj, isKindOfClass: class!(NSString)];
            if is_string {
                let ns_str: *const NSString = msg_send![obj, description];
                Some((*ns_str).to_string())
            } else {
                None
            }
        }
    }
}

// Helper function to copy an NSDictionary
fn copy_nsdictionary(dict: &NSDictionary<NSString, NSObject>) -> NSDictionary<NSString, NSObject> {
    unsafe {
        let copied_ptr: *mut NSObject = msg_send![dict, copy];
        let copied_dict_ptr = copied_ptr as *mut NSDictionary<NSString, NSObject>;
        if copied_dict_ptr.is_null() {
            panic!("Failed to copy dictionary");
        }
        // Use ptr::read to avoid moving out of raw pointer
        std::ptr::read(copied_dict_ptr)
    }
}

impl Clone for Box<dyn IOKit> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
