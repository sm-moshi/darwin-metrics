// Standard library imports
use std::{
    convert::AsRef,
    ffi::{c_char, c_void, CString},
    fmt::Debug,
    sync::{Arc, Mutex, Once},
    time::Duration,
};

// External crate imports
use libc::mach_port_t;
use objc2::{
    class,
    rc::Retained,
    runtime::{AnyClass, AnyObject, NSObject},
};

// Internal crate imports
use crate::{
    core::metrics::hardware::{
        CpuMonitor, GpuMonitor, PowerConsumptionMonitor, PowerEventMonitor, PowerManagementMonitor, PowerStateMonitor,
        ThermalMonitor,
    },
    error::{Error, Result},
    hardware::temperature::{Fan, ThermalMetrics},
    power::PowerState,
    utils::{
        core::DictionaryAccess,
        ffi::{
            IOByteCount, IOConnectCallStructMethod, IORegistryEntryCreateCFProperties, IORegistryEntryGetParentEntry,
            IOServiceGetMatchingService, IOServiceMatching, K_IOMASTER_PORT_DEFAULT,
        },
        SafeDictionary,
    },
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

impl DictionaryAccess for GpuStats {
    fn get_string(&self, key: &str) -> Option<String> {
        match key {
            "name" => Some(self.name.clone()),
            _ => None,
        }
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        match key {
            "utilization" => Some(self.utilization),
            "memory_used" => Some(self.memory_used as f64),
            "memory_total" => Some(self.memory_total as f64),
            "perf_cap" => Some(self.perf_cap),
            "perf_threshold" => Some(self.perf_threshold),
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
#[cfg(any(test, feature = "testing", feature = "mock"))]
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

impl DictionaryAccess for ThermalInfo {
    fn get_string(&self, _key: &str) -> Option<String> {
        None
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        match key {
            "cpu_temp" => Some(self.cpu_temp),
            "gpu_temp" => self.gpu_temp,
            "fan_speed" => Some(self.fan_speed as f64),
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

    fn get_dictionary(&self, key: &str) -> Option<SafeDictionary> {
        self.dict.get_dictionary(key)
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
#[async_trait::async_trait]
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

    /// Read a value from an SMC key
    fn read_smc_key(&self, key: [c_char; 4]) -> Result<Option<f32>>;

    /// Get statistics for multiple GPUs (for testing)
    fn get_gpu_stats_multiple(&self) -> Result<Vec<GpuStats>> {
        Ok(vec![self.get_gpu_stats()?])
    }
}

/// Implementation of the IOKit interface
#[derive(Debug, Clone, Default)]
pub struct IOKitImpl {
    // Add fields for caching if needed
}

impl IOKitImpl {
    /// Creates a new IOKitImpl instance
    pub fn new() -> Result<Self> {
        Ok(Self {
            // Initialize any required fields
        })
    }

    fn io_service_matching(&self, name: &str) -> Result<SafeDictionary> {
        let c_str = CString::new(name)?;
        let raw_dict = unsafe { IOServiceMatching(c_str.as_ptr()) };
        if raw_dict.is_null() {
            return Err(Error::iokit_error(0, "Failed to create matching dictionary for service"));
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
                return Err(Error::iokit_error(0, "Failed to get properties"));
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
                return Err(Error::iokit_error(0, "Failed to get matching service"));
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

        props.get_number("TC0P").ok_or_else(|| Error::iokit_error(0, "CPU frequency not found"))
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

        props.get_number("PC0C").ok_or_else(|| Error::iokit_error(0, "CPU power consumption not found"))
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
        dict.get_number(key).ok_or_else(|| Error::iokit_error(0, format!("Property {} not found", key)))
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
                return Err(Error::iokit_error(result, "Failed to call IOKit method"));
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
                return Err(Error::iokit_error(0, "Failed to get parent entry"));
            }
            Ok(ThreadSafeAnyObject::from_ptr(parent as *mut AnyObject))
        }
    }

    fn get_physical_cores(&self) -> Result<usize> {
        let info = self.get_cpu_info()?;
        info.get_number("PhysicalCores")
            .map(|n| n as usize)
            .ok_or_else(|| Error::iokit_error(0, "Physical core count not found"))
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
            return Err(Error::iokit_error(0, "Failed to create matching dictionary for service"));
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
                return Err(Error::iokit_error(0, "Failed to get matching service"));
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
                return Err(Error::iokit_error(0, "Failed to get properties"));
            }
            Ok(SafeDictionary::from_ptr(props as *mut NSObject))
        }
    }

    fn get_service_properties(&self, service: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        self.io_registry_entry_create_cf_properties(service)
    }

    fn get_number_property(&self, dict: &SafeDictionary, key: &str) -> Result<f64> {
        dict.get_number(key).ok_or_else(|| Error::iokit_error(0, format!("Property {} not found", key)))
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
                return Err(Error::iokit_error(0, "Failed to get parent entry"));
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
                return Err(Error::iokit_error(result, "Failed to call IOKit method"));
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

    fn read_smc_key(&self, key: [c_char; 4]) -> Result<Option<f32>> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        Ok(props.get_number("PC0C").map(|p| p as f32))
    }
}

static INIT: Once = Once::new();

fn ensure_classes_registered() {
    INIT.call_once(|| unsafe {
        let _: &AnyClass = class!(NSObject);
        let _: &AnyClass = class!(NSMutableDictionary);
        let _: &AnyClass = class!(NSNumber);
        let _: &AnyClass = class!(NSString);
    });
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
        ensure_classes_registered();
        Self { obj: Arc::new(Mutex::new(obj)), raw_handle: 0 }
    }

    /// Creates a new thread-safe wrapper for an AnyObject with a raw handle.
    pub fn with_raw_handle(obj: Retained<NSObject>, raw_handle: mach_port_t) -> Self {
        ensure_classes_registered();
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
        ensure_classes_registered();
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

impl DictionaryAccess for ThreadSafeAnyObject {
    fn get_string(&self, _key: &str) -> Option<String> {
        None
    }

    fn get_number(&self, _key: &str) -> Option<f64> {
        None
    }

    fn get_bool(&self, _key: &str) -> Option<bool> {
        None
    }

    fn get_dictionary(&self, _key: &str) -> Option<SafeDictionary> {
        None
    }
}

#[async_trait::async_trait]
impl CpuMonitor for IOKitImpl {
    async fn frequency(&self) -> Result<f64> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        props.get_number("CPU_Frequency").ok_or_else(|| Error::iokit_error(0, "CPU frequency not found"))
    }

    async fn min_frequency(&self) -> Result<f64> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        props.get_number("CPU_Min_Frequency").ok_or_else(|| Error::iokit_error(0, "CPU min frequency not found"))
    }

    async fn max_frequency(&self) -> Result<f64> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        props.get_number("CPU_Max_Frequency").ok_or_else(|| Error::iokit_error(0, "CPU max frequency not found"))
    }

    async fn available_frequencies(&self) -> Result<Vec<f64>> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;

        let mut frequencies = Vec::new();
        if let Some(freq_array) = props.get_array("CPU_Available_Frequencies") {
            for _ in freq_array {
                frequencies.push(0.0);
            }
        }
        Ok(frequencies)
    }

    async fn physical_cores(&self) -> Result<u32> {
        Ok(self.get_physical_cores()? as u32)
    }

    async fn logical_cores(&self) -> Result<u32> {
        Ok(self.get_logical_cores()? as u32)
    }

    async fn model_name(&self) -> Result<String> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        props.get_string("CPU_Model").ok_or_else(|| Error::iokit_error(0, "CPU model name not found"))
    }

    async fn temperature(&self) -> Result<Option<f64>> {
        Ok(Some(self.get_cpu_temperature("IOService")?))
    }

    async fn power_consumption(&self) -> Result<Option<f64>> {
        Ok(Some(self.get_cpu_power()?))
    }

    async fn core_usage(&self) -> Result<Vec<f64>> {
        self.get_core_usage()
    }

    async fn total_usage(&self) -> Result<f64> {
        let core_usage = self.get_core_usage()?;
        Ok(core_usage.iter().sum::<f64>() / core_usage.len() as f64)
    }
}

#[async_trait::async_trait]
impl GpuMonitor for IOKitImpl {
    async fn name(&self) -> Result<String> {
        let stats = self.get_gpu_stats()?;
        Ok(stats.name)
    }

    async fn utilization(&self) -> Result<f64> {
        let stats = self.get_gpu_stats()?;
        Ok(stats.utilization)
    }

    async fn temperature(&self) -> Result<Option<f32>> {
        let thermal_info = self.get_thermal_info()?;
        Ok(thermal_info.gpu_temp.map(|t| t as f32))
    }

    async fn total_memory(&self) -> Result<u64> {
        let stats = self.get_gpu_stats()?;
        Ok(stats.memory_total)
    }

    async fn used_memory(&self) -> Result<u64> {
        let stats = self.get_gpu_stats()?;
        Ok(stats.memory_used)
    }

    async fn free_memory(&self) -> Result<u64> {
        let stats = self.get_gpu_stats()?;
        Ok(stats.memory_total.saturating_sub(stats.memory_used))
    }

    async fn memory_utilization(&self) -> Result<f64> {
        let stats = self.get_gpu_stats()?;
        Ok((stats.memory_used as f64 / stats.memory_total as f64) * 100.0)
    }

    async fn supports_hardware_acceleration(&self) -> Result<bool> {
        let matching = self.io_service_matching("IOAccelerator")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        Ok(props.get_bool("SupportsHardwareAcceleration").unwrap_or(false))
    }

    async fn memory_bandwidth(&self) -> Result<Option<u64>> {
        let matching = self.io_service_matching("IOAccelerator")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        Ok(props.get_number("MemoryBandwidth").map(|b| b as u64))
    }
}

#[async_trait::async_trait]
impl ThermalMonitor for IOKitImpl {
    async fn cpu_temperature(&self) -> Result<Option<f64>> {
        Ok(Some(self.get_cpu_temperature("IOService")?))
    }

    async fn gpu_temperature(&self) -> Result<Option<f64>> {
        let thermal_info = self.get_thermal_info()?;
        Ok(thermal_info.gpu_temp)
    }

    async fn memory_temperature(&self) -> Result<Option<f64>> {
        let thermal_info = self.get_thermal_info()?;
        Ok(thermal_info.heatsink_temp)
    }

    async fn battery_temperature(&self) -> Result<Option<f64>> {
        let thermal_info = self.get_thermal_info()?;
        Ok(thermal_info.battery_temp)
    }

    async fn ambient_temperature(&self) -> Result<Option<f64>> {
        let thermal_info = self.get_thermal_info()?;
        Ok(thermal_info.ambient_temp)
    }

    async fn is_throttling(&self) -> Result<bool> {
        self.check_thermal_throttling("IOService")
    }

    async fn get_fans(&self) -> Result<Vec<Fan>> {
        let fan_info = self.get_all_fans()?;
        Ok(fan_info
            .into_iter()
            .map(|f| Fan {
                name: format!("Fan {}", f.speed_rpm),
                speed_rpm: f.speed_rpm,
                min_speed: f.min_speed,
                max_speed: f.max_speed,
                percentage: f.percentage,
            })
            .collect())
    }

    async fn get_thermal_metrics(&self) -> Result<ThermalMetrics> {
        let thermal_info = self.get_thermal_info()?;
        Ok(ThermalMetrics {
            cpu_temperature: Some(thermal_info.cpu_temp),
            gpu_temperature: thermal_info.gpu_temp,
            heatsink_temperature: thermal_info.heatsink_temp,
            ambient_temperature: thermal_info.ambient_temp,
            battery_temperature: thermal_info.battery_temp,
            is_throttling: thermal_info.thermal_throttling,
            cpu_power: None, // Not implemented yet
            fans: self.get_fans().await?,
            last_refresh: std::time::Instant::now(),
        })
    }
}

#[async_trait::async_trait]
impl PowerConsumptionMonitor for IOKitImpl {
    async fn package_power(&self) -> Result<f32> {
        Ok(self.get_cpu_power()? as f32)
    }

    async fn cores_power(&self) -> Result<f32> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        Ok(props.get_number("PCPC").unwrap_or(0.0) as f32)
    }

    async fn gpu_power(&self) -> Result<Option<f32>> {
        let matching = self.io_service_matching("IOAccelerator")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        Ok(props.get_number("GPUPower").map(|p| p as f32))
    }

    async fn dram_power(&self) -> Result<Option<f32>> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        Ok(props.get_number("PDRC").map(|p| p as f32))
    }

    async fn neural_engine_power(&self) -> Result<Option<f32>> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        Ok(props.get_number("PNEC").map(|p| p as f32))
    }

    async fn total_power(&self) -> Result<f32> {
        let package = self.package_power().await?;
        let cores = self.cores_power().await?;
        let gpu = self.gpu_power().await?.unwrap_or(0.0);
        let dram = self.dram_power().await?.unwrap_or(0.0);
        let neural = self.neural_engine_power().await?.unwrap_or(0.0);
        Ok(package + cores + gpu + dram + neural)
    }
}

#[async_trait::async_trait]
impl PowerStateMonitor for IOKitImpl {
    async fn power_state(&self) -> Result<PowerState> {
        let battery_info = self.get_battery_info()?;
        if !battery_info.get_bool("BatteryInstalled").unwrap_or(false) {
            return Ok(PowerState::AC);
        }
        if battery_info.get_bool("ExternalConnected").unwrap_or(false) {
            Ok(PowerState::Charging)
        } else {
            Ok(PowerState::Battery)
        }
    }

    async fn battery_percentage(&self) -> Result<Option<f32>> {
        let battery_info = self.get_battery_info()?;
        let current = battery_info.get_f64("CurrentCapacity")?;
        let max = battery_info.get_f64("MaxCapacity")?;
        Ok(Some((current / max * 100.0) as f32))
    }

    async fn time_remaining(&self) -> Result<Option<u32>> {
        let battery_info = self.get_battery_info()?;
        Ok(battery_info.get_i64("TimeRemaining").map(|t| t as u32))
    }

    async fn is_on_battery(&self) -> Result<bool> {
        let state = self.power_state().await?;
        Ok(matches!(state, PowerState::Battery))
    }

    async fn is_charging(&self) -> Result<bool> {
        let state = self.power_state().await?;
        Ok(matches!(state, PowerState::Charging))
    }
}

#[async_trait::async_trait]
impl PowerManagementMonitor for IOKitImpl {
    async fn is_thermal_throttling(&self) -> Result<bool> {
        self.check_thermal_throttling("IOService")
    }

    async fn power_impact(&self) -> Result<Option<f32>> {
        let matching = self.io_service_matching("IOPMrootDomain")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        Ok(props.get_number("PowerImpact").map(|p| p as f32))
    }

    async fn thermal_pressure(&self) -> Result<u32> {
        let matching = self.io_service_matching("IOPMrootDomain")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        Ok(props.get_number("ThermalPressure").unwrap_or(0.0) as u32)
    }

    async fn performance_mode(&self) -> Result<String> {
        let matching = self.io_service_matching("IOPMrootDomain")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        Ok(props.get_string("PerformanceMode").unwrap_or_else(|| "Normal".to_string()))
    }
}

#[async_trait::async_trait]
impl PowerEventMonitor for IOKitImpl {
    async fn time_since_wake(&self) -> Result<Duration> {
        let matching = self.io_service_matching("IOPMrootDomain")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        let secs = props.get_number("TimeSinceWake").unwrap_or(0.0);
        Ok(Duration::from_secs_f64(secs))
    }

    async fn thermal_event_count(&self) -> Result<u32> {
        let matching = self.io_service_matching("IOPMrootDomain")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        Ok(props.get_number("ThermalEventCount").unwrap_or(0.0) as u32)
    }

    async fn time_until_sleep(&self) -> Result<Option<Duration>> {
        let matching = self.io_service_matching("IOPMrootDomain")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        Ok(props.get_number("TimeUntilSleep").map(Duration::from_secs_f64))
    }

    async fn is_sleep_prevented(&self) -> Result<bool> {
        let matching = self.io_service_matching("IOPMrootDomain")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        Ok(props.get_bool("PreventSystemSleep").unwrap_or(false))
    }
}
