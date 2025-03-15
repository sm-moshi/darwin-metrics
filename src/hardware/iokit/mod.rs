use std::ffi::{c_void, CString};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::convert::AsRef;

use libc::mach_port_t;
use objc2::{
    rc::Retained,
    runtime::{AnyObject, NSObject},
};

use crate::error::{Error, Result};
use crate::utils::{
    bindings::{
        IORegistryEntryCreateCFProperties, IORegistryEntryGetParentEntry, IOServiceGetMatchingService, IOServiceMatching,
        IOConnectCallStructMethod, IOByteCount, K_IOMASTER_PORT_DEFAULT,
    },
    SafeDictionary,
};

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
            "utilization" => Some(self.utilization as f64),
            "memory_used" => Some(self.memory_used as f64),
            "memory_total" => Some(self.memory_total as f64),
            "perf_cap" => Some(self.perf_cap as f64),
            "perf_threshold" => Some(self.perf_threshold as f64),
            _ => None,
        }
    }

    fn get_bool(&self, _key: &str) -> Option<bool> {
        None
    }

    fn get_dictionary(&self, _key: &str) -> Option<SafeDictionary> {
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

impl Default for FanInfo {
    fn default() -> Self {
        Self { speed_rpm: 0, min_speed: 0, max_speed: 0, percentage: 0.0 }
    }
}

/// Thermal information structure
#[derive(Debug)]
pub struct ThermalInfo {
    /// CPU temperature in Celsius
    pub cpu_temp: f64,
    /// GPU temperature in Celsius
    pub gpu_temp: Option<f64>,
    /// Fan speed in RPM
    pub fan_speed: u32,
    /// Heatsink temperature in Celsius
    pub heatsink_temp: Option<f64>,
    /// Ambient temperature in Celsius
    pub ambient_temp: Option<f64>,
    /// Whether thermal throttling is active
    pub thermal_throttling: bool,
    /// Battery temperature in Celsius
    pub battery_temp: Option<f64>,
    /// Dictionary containing raw thermal data
    dict: SafeDictionary,
}

impl Default for ThermalInfo {
    fn default() -> Self {
        Self {
            cpu_temp: 0.0,
            gpu_temp: None,
            fan_speed: 0,
            heatsink_temp: None,
            ambient_temp: None,
            thermal_throttling: false,
            battery_temp: None,
            dict: SafeDictionary::new(),
        }
    }
}

impl ThermalInfo {
    /// Create a new ThermalInfo instance from a dictionary
    pub fn new(dict: SafeDictionary) -> Self {
        Self {
            cpu_temp: dict.get_number("CPU_0_DIE_TEMP").unwrap_or(0.0),
            gpu_temp: dict.get_number("GPU_0_DIE_TEMP"),
            fan_speed: dict.get_number("FAN_0_SPEED").unwrap_or(0.0) as u32,
            heatsink_temp: dict.get_number("HS_0_TEMP"),
            ambient_temp: dict.get_number("AMBIENT_TEMP"),
            thermal_throttling: dict.get_bool("THERMAL_THROTTLING").unwrap_or(false),
            battery_temp: dict.get_number("BATTERY_TEMP"),
            dict,
        }
    }

    /// Get a dictionary from the thermal info with the given key
    pub fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
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

impl crate::utils::dictionary_access::DictionaryAccess for ThermalInfo {
    fn get_string(&self, _key: &str) -> Option<String> {
        None
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        match key {
            "cpu_temp" => Some(self.cpu_temp),
            "heatsink_temp" => self.heatsink_temp,
            "ambient_temp" => self.ambient_temp,
            "battery_temp" => self.battery_temp,
            _ => None,
        }
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        match key {
            "thermal_throttling" => Some(self.thermal_throttling),
            _ => None,
        }
    }

    fn get_dictionary(&self, _key: &str) -> Option<SafeDictionary> {
        None
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

pub trait DictionaryAccess {
    fn get_dictionary(&self, key: &str) -> Option<SafeDictionary>;
}

/// IOKit interface for hardware monitoring
pub trait IOKit: Debug + Send + Sync {
    /// Create a matching dictionary for IOService
    fn io_service_matching(&self, name: &str) -> Result<SafeDictionary>;

    /// Get a service matching the given name
    fn get_service_matching(&self, name: &str) -> Result<Option<ThreadSafeAnyObject>>;

    /// Get service from matching dictionary
    fn io_service_get_matching_service(&self, matching_dict: &SafeDictionary) -> Result<ThreadSafeAnyObject>;

    /// Get properties for a registry entry
    fn io_registry_entry_create_cf_properties(&self, entry: &ThreadSafeAnyObject) -> Result<SafeDictionary>;

    /// Get CPU temperature
    fn get_cpu_temperature(&self, plane: &str) -> Result<f64>;

    /// Get thermal information
    fn get_thermal_info(&self) -> Result<ThermalInfo>;

    /// Get all fans
    fn get_all_fans(&self) -> Result<Vec<FanInfo>>;

    /// Check if thermal throttling is active
    fn check_thermal_throttling(&self, plane: &str) -> Result<bool>;

    /// Get CPU power consumption in watts
    fn get_cpu_power(&self) -> Result<f64>;

    /// Get GPU statistics
    fn get_gpu_stats(&self) -> Result<GpuStats>;

    /// Get information about a specific fan
    fn get_fan_info(&self, fan_index: u32) -> Result<FanInfo>;

    /// Get battery temperature
    fn get_battery_temperature(&self) -> Result<Option<f64>>;

    /// Get battery information
    fn get_battery_info(&self) -> Result<SafeDictionary>;

    /// Get CPU information
    fn get_cpu_info(&self) -> Result<SafeDictionary>;

    /// Get a number property from a dictionary
    fn get_number_property(&self, dict: &SafeDictionary, key: &str) -> Result<f64>;

    /// Call an IOKit method
    fn io_connect_call_method(&self, connection: u32, selector: u32, _input: &[u64], output: &mut [u64]) -> Result<()>;

    /// Clone this IOKit instance into a Box
    fn clone_box(&self) -> Box<dyn IOKit>;

    /// Get the parent entry of a registry entry
    fn io_registry_entry_get_parent_entry(&self, entry: &ThreadSafeAnyObject) -> Result<ThreadSafeAnyObject>;

    /// Get service properties
    fn get_service_properties(&self, service: &ThreadSafeAnyObject) -> Result<SafeDictionary>;

    /// Get physical cores
    fn get_physical_cores(&self) -> Result<usize>;

    /// Get logical cores
    fn get_logical_cores(&self) -> Result<usize>;

    /// Get core usage
    fn get_core_usage(&self) -> Result<Vec<f64>>;

    /// Get dictionary
    fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        None // Default implementation
    }
}

/// Implementation of the IOKit interface
#[derive(Debug, Clone, Default)]
pub struct IOKitImpl {
    // Add fields for caching if needed
}

impl IOKitImpl {
    pub fn new() -> Self {
        Self {}
    }

    fn io_service_matching(&self, name: &str) -> Result<SafeDictionary> {
        let c_str = CString::new(name)?;
        let raw_dict = unsafe { IOServiceMatching(c_str.as_ptr()) };
        if raw_dict.is_null() {
            return Err(Error::IOKitError {
                code: 0,
                message: format!("Failed to create matching dictionary for service {}", name),
            });
        }
        unsafe { Ok(SafeDictionary::from_ptr(raw_dict as *mut NSObject)) }
    }

    fn io_registry_entry_create_cf_properties(&self, entry: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        let mut props: *mut c_void = std::ptr::null_mut();
        unsafe {
            let result = IORegistryEntryCreateCFProperties(
                entry.get_raw_handle() as mach_port_t,
                &mut props,
                std::ptr::null_mut(), // kCFAllocatorDefault
                0,
            );
            if result != 0 || props.is_null() {
                return Err(Error::IOKitError { code: 0, message: "Failed to get properties".into() });
            }
            Ok(SafeDictionary::from_ptr(props as *mut NSObject))
        }
    }

    fn get_service_properties(&self, service: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        self.io_registry_entry_create_cf_properties(service)
    }

    fn io_service_get_matching_service(&self, matching_dict: &SafeDictionary) -> Result<ThreadSafeAnyObject> {
        unsafe {
            let raw_service = IOServiceGetMatchingService(K_IOMASTER_PORT_DEFAULT, matching_dict.as_ptr() as *mut _);
            if raw_service == 0 {
                return Err(Error::IOKitError { code: 0, message: "Failed to get matching service".into() });
            }
            Ok(ThreadSafeAnyObject::from_ptr(raw_service as *mut AnyObject))
        }
    }

    fn get_battery_info(&self) -> Result<SafeDictionary> {
        let matching = self.io_service_matching("AppleSmartBattery")?;
        let service = self.io_service_get_matching_service(&matching)?;
        self.get_service_properties(&service)
    }

    fn get_cpu_info(&self) -> Result<SafeDictionary> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        self.get_service_properties(&service)
    }

    fn get_cpu_temperature(&self, plane: &str) -> Result<f64> {
        let matching = self.io_service_matching(plane)?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        props
            .get_number("TC0P")
            .ok_or_else(|| Error::IOKitError { code: 0, message: format!("Property {} not found", "TC0P") })
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let dict = self.get_service_properties(&service)?;
        let plane = "IOService";

        Ok(ThermalInfo {
            cpu_temp: self.get_cpu_temperature(plane)?,
            gpu_temp: dict.get_number("GPU_0_DIE_TEMP"),
            fan_speed: dict.get_number("FAN_0_SPEED").unwrap_or(0.0) as u32,
            heatsink_temp: dict.get_number("HS_0_TEMP"),
            ambient_temp: dict.get_number("AMBIENT_TEMP"),
            thermal_throttling: self.check_thermal_throttling(plane)?,
            battery_temp: self.get_battery_temperature()?,
            dict,
        })
    }

    fn get_all_fans(&self) -> Result<Vec<FanInfo>> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        let mut fans = Vec::new();
        if let Some(speed) = props.get_number("FAN_0_SPEED") {
            fans.push(FanInfo {
                speed_rpm: speed as u32,
                min_speed: props.get_number("FAN_0_MIN_SPEED").unwrap_or(0.0) as u32,
                max_speed: props.get_number("FAN_0_MAX_SPEED").unwrap_or(0.0) as u32,
                percentage: props.get_number("FAN_0_PERCENTAGE").unwrap_or(0.0),
            });
        }
        Ok(fans)
    }

    fn check_thermal_throttling(&self, plane: &str) -> Result<bool> {
        let matching = self.io_service_matching(plane)?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        Ok(props.get_bool("ThermalThrottling").unwrap_or(false))
    }

    fn get_cpu_power(&self) -> Result<f64> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        props
            .get_number("PC0C")
            .ok_or_else(|| Error::IOKitError { code: 0, message: format!("Property {} not found", "PC0C") })
    }

    fn get_gpu_stats(&self) -> Result<GpuStats> {
        let matching = self.io_service_matching("IOPlatformDevice")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        Ok(GpuStats {
            utilization: props.get_number("GPUUtilization").unwrap_or(0.0),
            memory_used: props.get_number("GPUMemoryUsed").unwrap_or(0.0) as u64,
            memory_total: props.get_number("GPUMemoryTotal").unwrap_or(0.0) as u64,
            perf_cap: props.get_number("GPUPerfCap").unwrap_or(0.0),
            perf_threshold: props.get_number("GPUPerfThreshold").unwrap_or(0.0),
            name: props.get_string("GPUName").unwrap_or_else(|| String::from("Unknown")),
        })
    }

    fn get_fan_info(&self, fan_index: u32) -> Result<FanInfo> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        let prefix = format!("FAN_{}_", fan_index);
        Ok(FanInfo {
            speed_rpm: props.get_number(&format!("{}SPEED", prefix)).unwrap_or(0.0) as u32,
            min_speed: props.get_number(&format!("{}MIN_SPEED", prefix)).unwrap_or(0.0) as u32,
            max_speed: props.get_number(&format!("{}MAX_SPEED", prefix)).unwrap_or(0.0) as u32,
            percentage: props.get_number(&format!("{}PERCENTAGE", prefix)).unwrap_or(0.0),
        })
    }

    fn get_battery_temperature(&self) -> Result<Option<f64>> {
        let matching = self.io_service_matching("AppleSmartBattery")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        Ok(props.get_number("Temperature"))
    }

    fn get_number_property(&self, dict: &SafeDictionary, key: &str) -> Result<f64> {
        dict.get_number(key)
            .ok_or_else(|| Error::IOKitError { code: 0, message: format!("Property {} not found", key) })
    }

    fn io_connect_call_method(&self, connection: u32, selector: u32, _input: &[u64], output: &mut [u64]) -> Result<()> {
        let output_count = output.len() as u32;

        unsafe {
            let result = IOConnectCallStructMethod(
                connection,
                selector,
                std::ptr::null(),
                IOByteCount(0),
                output.as_mut_ptr() as *mut _,
                &mut IOByteCount(output_count as usize),
            );

            if result != 0 {
                return Err(Error::IOKitError { code: result, message: "Failed to call IOKit method".into() });
            }

            Ok(())
        }
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
    }

    fn io_registry_entry_get_parent_entry(&self, entry: &ThreadSafeAnyObject) -> Result<ThreadSafeAnyObject> {
        unsafe {
            let mut parent: mach_port_t = 0;
            let result =
                IORegistryEntryGetParentEntry(entry.get_raw_handle(), b"IOService\0".as_ptr() as *const _, &mut parent);
            if result != 0 || parent == 0 {
                return Err(Error::IOKitError { code: 0, message: "Failed to get parent entry".into() });
            }
            Ok(ThreadSafeAnyObject::from_ptr(parent as *mut AnyObject))
        }
    }

    fn get_physical_cores(&self) -> Result<usize> {
        let info = self.get_cpu_info()?;
        info.get_number("PhysicalCores")
            .map(|n| n as usize)
            .ok_or_else(|| Error::IOKitError { code: 0, message: "Physical core count not found".into() })
    }

    fn get_logical_cores(&self) -> Result<usize> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        Ok(props.get_number("LogicalCores").unwrap_or(1.0) as usize)
    }

    fn get_core_usage(&self) -> Result<Vec<f64>> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        let mut usage = Vec::new();
        let cores = self.get_logical_cores()?;
        for i in 0..cores {
            if let Ok(core_usage) = self.get_number_property(&props, &format!("Core{}_Usage", i)) {
                usage.push(core_usage);
            }
        }
        Ok(usage)
    }
}

impl IOKit for IOKitImpl {
    fn io_service_matching(&self, name: &str) -> Result<SafeDictionary> {
        let c_str = CString::new(name)?;
        let raw_dict = unsafe { IOServiceMatching(c_str.as_ptr()) };
        if raw_dict.is_null() {
            return Err(Error::IOKitError {
                code: 0,
                message: format!("Failed to create matching dictionary for service {}", name),
            });
        }
        unsafe { Ok(SafeDictionary::from_ptr(raw_dict as *mut NSObject)) }
    }

    fn get_service_matching(&self, name: &str) -> Result<Option<ThreadSafeAnyObject>> {
        let matching = self.io_service_matching(name)?;
        match self.io_service_get_matching_service(&matching) {
            Ok(service) => Ok(Some(service)),
            Err(_) => Ok(None),
        }
    }

    fn io_service_get_matching_service(&self, matching_dict: &SafeDictionary) -> Result<ThreadSafeAnyObject> {
        unsafe {
            let raw_service = IOServiceGetMatchingService(K_IOMASTER_PORT_DEFAULT, matching_dict.as_ptr() as *mut _);
            if raw_service == 0 {
                return Err(Error::IOKitError { code: 0, message: "Failed to get matching service".into() });
            }
            Ok(ThreadSafeAnyObject::from_ptr(raw_service as *mut AnyObject))
        }
    }

    fn io_registry_entry_create_cf_properties(&self, entry: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        let mut props: *mut c_void = std::ptr::null_mut();
        unsafe {
            let result = IORegistryEntryCreateCFProperties(
                entry.get_raw_handle() as mach_port_t,
                &mut props,
                std::ptr::null_mut(), // kCFAllocatorDefault
                0,
            );
            if result != 0 || props.is_null() {
                return Err(Error::IOKitError { code: 0, message: "Failed to get properties".into() });
            }
            Ok(SafeDictionary::from_ptr(props as *mut NSObject))
        }
    }

    fn get_service_properties(&self, service: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        self.io_registry_entry_create_cf_properties(service)
    }

    fn get_number_property(&self, dict: &SafeDictionary, key: &str) -> Result<f64> {
        dict.get_number(key)
            .ok_or_else(|| Error::IOKitError { code: 0, message: format!("Property {} not found", key) })
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let dict = self.get_service_properties(&service)?;
        let plane = "IOService";

        Ok(ThermalInfo {
            cpu_temp: self.get_cpu_temperature(plane)?,
            gpu_temp: dict.get_number("GPU_0_DIE_TEMP"),
            fan_speed: dict.get_number("FAN_0_SPEED").unwrap_or(0.0) as u32,
            heatsink_temp: dict.get_number("HS_0_TEMP"),
            ambient_temp: dict.get_number("AMBIENT_TEMP"),
            thermal_throttling: self.check_thermal_throttling(plane)?,
            battery_temp: self.get_battery_temperature()?,
            dict,
        })
    }

    fn get_all_fans(&self) -> Result<Vec<FanInfo>> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        let mut fans = Vec::new();
        if let Ok(speed) = self.get_number_property(&props, "FAN_0_SPEED") {
            fans.push(FanInfo {
                speed_rpm: speed as u32,
                min_speed: self.get_number_property(&props, "FAN_0_MIN_SPEED").unwrap_or(0.0) as u32,
                max_speed: self.get_number_property(&props, "FAN_0_MAX_SPEED").unwrap_or(0.0) as u32,
                percentage: self.get_number_property(&props, "FAN_0_PERCENTAGE").unwrap_or(0.0),
            });
        }
        Ok(fans)
    }

    fn get_fan_info(&self, fan_index: u32) -> Result<FanInfo> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        let prefix = format!("FAN_{}_", fan_index);
        Ok(FanInfo {
            speed_rpm: self.get_number_property(&props, &format!("{}SPEED", prefix)).unwrap_or(0.0) as u32,
            min_speed: self.get_number_property(&props, &format!("{}MIN_SPEED", prefix)).unwrap_or(0.0) as u32,
            max_speed: self.get_number_property(&props, &format!("{}MAX_SPEED", prefix)).unwrap_or(0.0) as u32,
            percentage: self.get_number_property(&props, &format!("{}PERCENTAGE", prefix)).unwrap_or(0.0),
        })
    }

    fn get_gpu_stats(&self) -> Result<GpuStats> {
        let matching = self.io_service_matching("IOPlatformDevice")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        Ok(GpuStats {
            utilization: self.get_number_property(&props, "GPUUtilization").unwrap_or(0.0),
            memory_used: self.get_number_property(&props, "GPUMemoryUsed").unwrap_or(0.0) as u64,
            memory_total: self.get_number_property(&props, "GPUMemoryTotal").unwrap_or(0.0) as u64,
            perf_cap: self.get_number_property(&props, "GPUPerfCap").unwrap_or(0.0),
            perf_threshold: self.get_number_property(&props, "GPUPerfThreshold").unwrap_or(0.0),
            name: props.get_string("GPUName").unwrap_or_else(|| String::from("Unknown")),
        })
    }

    fn get_battery_temperature(&self) -> Result<Option<f64>> {
        let matching = self.io_service_matching("AppleSmartBattery")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        Ok(props.get_number("Temperature"))
    }

    fn get_battery_info(&self) -> Result<SafeDictionary> {
        let matching = self.io_service_matching("AppleSmartBattery")?;
        let service = self.io_service_get_matching_service(&matching)?;
        self.get_service_properties(&service)
    }

    fn get_cpu_info(&self) -> Result<SafeDictionary> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        self.get_service_properties(&service)
    }

    fn get_cpu_temperature(&self, plane: &str) -> Result<f64> {
        let matching = self.io_service_matching(plane)?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        self.get_number_property(&props, "TC0P")
    }

    fn get_cpu_power(&self) -> Result<f64> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        self.get_number_property(&props, "PC0C")
    }

    fn check_thermal_throttling(&self, plane: &str) -> Result<bool> {
        let matching = self.io_service_matching(plane)?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        Ok(props.get_bool("ThermalThrottling").unwrap_or(false))
    }

    fn io_registry_entry_get_parent_entry(&self, entry: &ThreadSafeAnyObject) -> Result<ThreadSafeAnyObject> {
        unsafe {
            let mut parent: mach_port_t = 0;
            let result =
                IORegistryEntryGetParentEntry(entry.get_raw_handle(), b"IOService\0".as_ptr() as *const _, &mut parent);
            if result != 0 || parent == 0 {
                return Err(Error::IOKitError { code: 0, message: "Failed to get parent entry".into() });
            }
            Ok(ThreadSafeAnyObject::from_ptr(parent as *mut AnyObject))
        }
    }

    fn io_connect_call_method(&self, connection: u32, selector: u32, _input: &[u64], output: &mut [u64]) -> Result<()> {
        let output_count = output.len() as u32;

        unsafe {
            let result = IOConnectCallStructMethod(
                connection,
                selector,
                std::ptr::null(),
                IOByteCount(0),
                output.as_mut_ptr() as *mut _,
                &mut IOByteCount(output_count as usize),
            );

            if result != 0 {
                return Err(Error::IOKitError { code: result, message: "Failed to call IOKit method".into() });
            }

            Ok(())
        }
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
    }

    fn get_physical_cores(&self) -> Result<usize> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        Ok(props.get_number("PhysicalCores").unwrap_or(1.0) as usize)
    }

    fn get_logical_cores(&self) -> Result<usize> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        Ok(props.get_number("LogicalCores").unwrap_or(1.0) as usize)
    }

    fn get_core_usage(&self) -> Result<Vec<f64>> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        let mut usage = Vec::new();
        let cores = self.get_logical_cores()?;
        for i in 0..cores {
            if let Ok(core_usage) = self.get_number_property(&props, &format!("Core{}_Usage", i)) {
                usage.push(core_usage);
            }
        }
        Ok(usage)
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

    /// Creates a new thread-safe wrapper from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - The pointer is valid and properly aligned
    /// - The pointer points to a valid NSObject
    /// - The object is not mutated while this ThreadSafeAnyObject exists
    pub unsafe fn from_ptr(ptr: *mut AnyObject) -> Self {
        let obj =
            Retained::from_raw(ptr as *mut NSObject).expect("Failed to create Retained<NSObject> from raw pointer");
        Self::new(obj)
    }

    /// Gets the raw pointer to the underlying NSObject.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - The pointer is not used after the ThreadSafeAnyObject is dropped
    /// - The pointer is not used to mutate the object while other references exist
    pub unsafe fn get_ptr(&self) -> *mut AnyObject {
        let retained = self.obj.lock().expect("Failed to lock mutex");
        Retained::as_ptr(&retained) as *mut AnyObject
    }

    /// Gets the raw handle associated with this object.
    pub fn get_raw_handle(&self) -> mach_port_t {
        self.raw_handle
    }

    /// Gets a reference to the inner AnyObject.
    pub fn inner(&self) -> Retained<NSObject> {
        self.obj.lock().expect("Failed to lock mutex").clone()
    }

    fn get_dictionary(&self, _key: &str) -> Option<SafeDictionary> {
        // Create a new SafeDictionary using default()
        Some(SafeDictionary::default())
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

impl AsRef<NSObject> for ThreadSafeAnyObject {
    fn as_ref(&self) -> &NSObject {
        // SAFETY: We maintain the invariant that obj always contains a valid NSObject
        // and the Arc ensures the object outlives any references. The mutex ensures
        // thread-safety, and we're only creating a reference that lives as long as self.
        unsafe {
            let guard = self.obj.lock().unwrap();
            &*(guard.as_ref() as *const NSObject)
        }
    }
}
