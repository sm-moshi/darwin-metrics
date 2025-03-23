// Standard library imports
use std::ffi::{c_char, c_void};
use std::fmt::Debug;
use std::ptr::{NonNull, null_mut};
use std::sync::{Arc, Once};
use std::time::Duration;
use std::time::SystemTime;

use async_trait::async_trait;
// External crate imports
use objc2::class;
use objc2::runtime::AnyClass;
use objc2::rc::Retained;
use objc2_foundation::{NSObject, NSString};
use objc2_core_foundation::{
    CFAllocator,
    CFDictionary,
    CFString,
};
use tokio::sync::Mutex;

// Internal crate imports
use crate::{
    core::metrics::hardware::{
        CpuMonitor, GpuMonitor, PowerConsumptionMonitor, PowerEventMonitor, PowerManagementMonitor, PowerStateMonitor,
        ThermalMonitor,
    },
    error::{Error, Result},
    ffi::{
        CFRelease, IOConnectCallStructMethod, IORegistryEntryCreateCFProperties,
        IORegistryEntryGetParentEntry, IOServiceGetMatchingService, IOServiceMatching,
        SMCKeyData_t, SMC_CMD_READ_BYTES, SMC_CMD_READ_KEYINFO, SMC_KEY_FAN_NUM,
        kIOMasterPortDefault, kIOReturnSuccess, kIOServicePlane,
        smc_key_from_chars, SmcKey, io_service_t, IOServiceOpen, IOServiceClose,
        IOObjectRelease, mach_task_self, KERN_SUCCESS,
    },
    power::PowerState,
    temperature::constants::*,
    temperature::types::{Fan, ThermalLevel, ThermalMetrics},
    utils::{SafeDictionary, core::DictionaryAccess},
};

/// Represents detailed information about a fan
#[derive(Debug, Clone)]
pub struct FanInfo {
    pub current_speed: u32,
    pub target_speed: u32,
    pub min_speed: Option<u32>,
    pub max_speed: Option<u32>,
    pub index: usize,
    pub speed_rpm: u32,
    pub percentage: f64,
}

impl FanInfo {
    pub fn new(index: u32) -> Self {
        Self {
            current_speed: 0,
            target_speed: 0,
            min_speed: Some(0),
            max_speed: Some(0),
            index,
            speed_rpm: 0,
            percentage: 0.0,
        }
    }
}

/// GPU statistics
#[derive(Debug, Clone)]
pub struct GpuStats {
    /// Utilization percentage of the GPU (0-100)
    pub utilization: f64,
    /// Amount of GPU memory currently in use (bytes)
    pub memory_used: u64,
    /// Total GPU memory available (bytes)
    pub memory_total: u64,
    /// Performance capability ratio (0.0-1.0)
    pub perf_cap: f64,
    /// Performance threshold ratio (0.0-1.0)
    pub perf_threshold: f64,
    /// Name of the GPU
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

/// Fan information from IOKit
#[derive(Debug, Clone)]
pub struct IOKitFanInfo {
    /// Current speed in RPM
    pub speed_rpm: u32,
    /// Minimum speed in RPM
    pub min_speed: u32,
    /// Maximum speed in RPM
    pub max_speed: u32,
    /// Current percentage of maximum speed
    pub percentage: f64,
}

impl From<IOKitFanInfo> for Fan {
    fn from(f: IOKitFanInfo) -> Self {
        Self {
            name: "System Fan".to_string(),
            speed_rpm: f.speed_rpm,
            min_speed: f.min_speed,
            max_speed: f.max_speed,
            target_speed: (f.percentage * f.max_speed as f64) as u32,
        }
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
    /// Number of fans in the system
    pub fan_count: Option<u32>,
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
            fan_count: None,
            dict: SafeDictionary::new(),
        }
    }
}

impl ThermalInfo {
    /// Create a new ThermalInfo instance from a dictionary
    pub fn new(dict: SafeDictionary) -> Self {
        let mut info = Self::default();
        info.dict = dict;

        // Initialize fields from dictionary
        info.cpu_temp = info.dict.get_number("CPU_0_DIE_TEMP").unwrap_or(0.0);
        info.gpu_temp = info.dict.get_number("GPU_0_DIE_TEMP");
        info.fan_speed = info.dict.get_number("FAN_0_SPEED").unwrap_or(0.0) as u32;
        info.heatsink_temp = info.dict.get_number("HS_0_TEMP");
        info.ambient_temp = info.dict.get_number("AMBIENT_TEMP");
        info.thermal_throttling = info.dict.get_bool("THERMAL_THROTTLING").unwrap_or(false);
        info.battery_temp = info.dict.get_number("BATTERY_TEMP");

        // Try to get fan count from dictionary
        if let Some(count) = info.dict.get_number("Fan_Count") {
            info.fan_count = Some(count as u32);
        }

        info
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

    /// Get the number of fans in the system
    pub fn get_fan_count(&self) -> u32 {
        self.fan_count.unwrap_or(1)
    }

    /// Get the speed of a specific fan
    pub fn get_fan_speed(&self, fan_index: u32) -> Option<u32> {
        if fan_index >= self.get_fan_count() {
            return None;
        }

        let key = format!("FAN_{}_SPEED", fan_index);
        self.dict.get_number(&key).map(|speed| speed as u32)
    }

    /// Get all fan speeds
    pub fn get_all_fan_speeds(&self) -> Vec<u32> {
        (0..self.get_fan_count())
            .filter_map(|i| self.get_fan_speed(i))
            .collect()
    }

    /// Get fan target speed if available
    pub fn get_fan_target_speed(&self, fan_index: u32) -> Option<u32> {
        if fan_index >= self.get_fan_count() {
            return None;
        }

        let key = format!("FAN_{}_TARGET", fan_index);
        self.dict.get_number(&key).map(|speed| speed as u32)
    }

    /// Get all fan target speeds
    pub fn get_all_fan_target_speeds(&self) -> Vec<u32> {
        (0..self.get_fan_count())
            .filter_map(|i| self.get_fan_target_speed(i))
            .collect()
    }

    /// Get detailed information about a specific fan
    pub fn get_fan_info(&self, fan_index: u32) -> Option<FanInfo> {
        if fan_index >= self.get_fan_count() {
            return None;
        }

        // Get current speed (required)
        let current_speed = self.get_fan_speed(fan_index)?;

        // Get optional values
        let target_speed = self.get_fan_target_speed(fan_index).unwrap_or(0);
        let min_speed = self
            .dict
            .get_number(&format!("FAN_{}_MIN_SPEED", fan_index))
            .map(|speed| speed as u32);
        let max_speed = self
            .dict
            .get_number(&format!("FAN_{}_MAX_SPEED", fan_index))
            .map(|speed| speed as u32);

        // Calculate percentage if max speed is available
        let percentage = if let Some(max) = max_speed {
            if max > 0 {
                (current_speed as f64 / max as f64) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        Some(FanInfo {
            current_speed,
            target_speed,
            min_speed,
            max_speed,
            index: fan_index as usize,
            speed_rpm: current_speed,
            percentage,
        })
    }

    /// Get detailed information about all fans
    pub fn get_all_fan_info(&self) -> Vec<FanInfo> {
        (0..self.get_fan_count()).filter_map(|i| self.get_fan_info(i)).collect()
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
            fan_count: self.fan_count,
            dict: self.dict.clone(),
        }
    }
}

/// Custom type for IORegistry connection
pub type IORegistryConnection = u32;

/// IOKit interface for hardware monitoring
#[async_trait::async_trait]
pub trait IOKit: Debug + Send + Sync {
    /// Create a matching dictionary for IOService
    async fn io_service_matching(&self, name: &str) -> Result<SafeDictionary>;

    /// Get a service matching the given name
    async fn get_service_matching(&self, name: &str) -> Result<Option<SafeDictionary>>;

    /// Get service from matching dictionary
    fn io_service_get_matching_service(&self, matching_dict: &SafeDictionary) -> Result<ThreadSafeAnyObject>;

    /// Get properties for a registry entry
    async fn io_registry_entry_create_cf_properties(&self, entry: u32) -> Result<SafeDictionary>;

    /// Get service properties
    async fn get_service_properties(&self, name: &str) -> Result<Option<SafeDictionary>>;

    /// Get CPU temperature
    fn get_cpu_temperature(&self, plane: &str) -> Result<f64>;

    /// Get thermal information
    fn get_thermal_info(&self) -> Result<ThermalInfo>;

    /// Get all fans
    fn get_all_fans(&self) -> Result<Vec<IOKitFanInfo>>;

    /// Check if thermal throttling is active
    fn check_thermal_throttling(&self, plane: &str) -> Result<bool>;

    /// Get CPU power consumption in watts
    fn get_cpu_power(&self) -> Result<f64>;

    /// Get GPU statistics
    fn get_gpu_stats(&self) -> Result<GpuStats>;

    /// Get information about a specific fan
    fn get_fan_info(&self, fan_index: u32) -> Result<IOKitFanInfo>;

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

    /// Get physical cores
    fn get_physical_cores(&self) -> Result<usize>;

    /// Get logical cores
    fn get_logical_cores(&self) -> Result<usize>;

    /// Get core usage
    fn get_core_usage(&self) -> Result<Vec<f64>>;

    /// Get dictionary
    fn get_dictionary(&self, dict_ptr: *mut c_void) -> Result<SafeDictionary>;

    /// Read a value from an SMC key
    fn read_smc_key(&self, key: [i8; 4]) -> Result<Option<f32>>;

    /// Get statistics for multiple GPUs (for testing)
    fn get_gpu_stats_multiple(&self) -> Result<Vec<GpuStats>> {
        Ok(vec![self.get_gpu_stats()?])
    }

    /// Get properties from pointer
    fn get_properties(&self, props: *mut c_void) -> Result<SafeDictionary>;

    /// Get battery properties from pointer
    fn get_battery_properties(&self, props: *mut c_void) -> Result<SafeDictionary>;

    /// Get battery percentage from properties
    fn get_battery_percentage(&self, dict: &SafeDictionary) -> Result<Option<f32>>;

    /// Get the number of fans in the system
    async fn get_fan_count(&self) -> Result<usize>;

    /// Get the current speed of a fan by index
    async fn get_fan_speed(&self, index: usize) -> Result<u32>;

    /// Get the minimum speed of a fan by index
    async fn get_fan_min_speed(&self, index: usize) -> Result<u32>;

    /// Get the maximum speed of a fan by index
    async fn get_fan_max_speed(&self, index: usize) -> Result<u32>;
}

/// IOKit implementation for hardware monitoring
#[derive(Debug, Clone)]
pub struct IOKitImpl {
    matching_dict: Arc<Mutex<Option<CFDictionary>>>,
}

impl IOKitImpl {
    pub fn new() -> Result<Self> {
        Ok(Self {
            matching_dict: Arc::new(Mutex::new(None)),
        })
    }
}

// Make IOKitImpl Send + Sync by using thread-safe types
unsafe impl Send for IOKitImpl {}
unsafe impl Sync for IOKitImpl {}

impl Drop for IOKitImpl {
    fn drop(&mut self) {
        // Drop implementation
    }
}

impl Clone for Box<dyn IOKit> {
    fn clone(&self) -> Self {
        Box::new((*self).clone())
    }
}

#[async_trait::async_trait]
impl IOKit for IOKitImpl {
    async fn io_service_matching(&self, name: &str) -> Result<SafeDictionary> {
        let cf_string = unsafe {
            let matching_dict = IOServiceMatching(name.as_ptr() as *const i8);
            if matching_dict.is_null() {
                return Err(Error::IOKitError("Failed to create matching dictionary".into()));
            }
            SafeDictionary::from_cf_dictionary_ref(matching_dict as *const CFDictionary)
        };
        Ok(cf_string)
    }

    async fn get_service_matching(&self, name: &str) -> Result<Option<SafeDictionary>> {
        let matching_dict = self.io_service_matching(name).await?;
        let service = unsafe {
            IOServiceGetMatchingService(kIOMasterPortDefault, matching_dict.as_cf_dictionary_ref())
        };
        
        if service == 0 {
            return Ok(None);
        }

        let mut properties = std::ptr::null_mut();
        let result = unsafe {
            IORegistryEntryCreateCFProperties(
                service,
                &mut properties,
                std::ptr::null_mut(),
                0,
            )
        };

        if result != 0 {
            unsafe { CFRelease(service as *mut _) };
            return Err(Error::IOKitError("Failed to get service properties".into()));
        }

        let dict = unsafe { SafeDictionary::from_cf_dictionary_ref(properties as *const CFDictionary) };
        unsafe { CFRelease(service as *mut _) };
        Ok(Some(dict))
    }

    fn io_service_get_matching_service(&self, matching_dict: &SafeDictionary) -> Result<ThreadSafeAnyObject> {
        let master_port = crate::ffi::kIOMasterPortDefault;
        let service = unsafe { crate::utils::ffi::IOServiceGetMatchingService(master_port, matching_dict.as_ptr() as *mut _) };

        if service == 0 {
            return Err(Error::system_error("IOServiceGetMatchingService failed"));
        }

        Ok(ThreadSafeAnyObject::new(service))
    }

    async fn io_registry_entry_create_cf_properties(&self, entry: u32) -> Result<SafeDictionary> {
        let mut properties = std::ptr::null_mut();
        let result = unsafe {
            IORegistryEntryCreateCFProperties(
                entry,
                &mut properties,
                std::ptr::null_mut(),
                0,
            )
        };

        if result != 0 {
            return Err(Error::IOKitError("Failed to create CF properties".into()));
        }

        Ok(unsafe { SafeDictionary::from_cf_dictionary_ref(properties as *const CFDictionary) })
    }

    async fn get_service_properties(&self, name: &str) -> Result<Option<SafeDictionary>> {
        self.get_service_matching(name).await
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
    }

    fn get_cpu_temperature(&self, plane: &str) -> Result<f64> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.io_registry_entry_create_cf_properties(&service)?;
        props.get_number("CPU_TEMP")
            .ok_or_else(|| Error::NotAvailable("CPU temperature not available".into()))
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo> {
        let matching = self.io_service_matching("IOHIDSystem")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.io_registry_entry_create_cf_properties(&service)?;
        Ok(ThermalInfo::new(props))
    }

    fn get_all_fans(&self) -> Result<Vec<IOKitFanInfo>> {
        Ok(vec![
            IOKitFanInfo {
                speed_rpm: 1200,
                min_speed: 0,
                max_speed: 5000,
                percentage: 24.0,
            },
            IOKitFanInfo {
                speed_rpm: 1300,
                min_speed: 0,
                max_speed: 5500,
                percentage: 23.6,
            },
        ])
    }

    fn check_thermal_throttling(&self, plane: &str) -> Result<bool> {
        let matching = self.io_service_matching(plane)?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.io_registry_entry_create_cf_properties(&service)?;
        Ok(props.get_bool("THERMAL_THROTTLING").unwrap_or(false))
    }

    fn get_cpu_power(&self) -> Result<f64> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.io_registry_entry_create_cf_properties(&service)?;
        props.get_number("CPU_POWER")
            .ok_or_else(|| Error::NotAvailable("CPU power not available".into()))
    }

    fn get_gpu_stats(&self) -> Result<GpuStats> {
        let matching = self.io_service_matching("IOAccelerator")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.io_registry_entry_create_cf_properties(&service)?;
        Ok(GpuStats::default())  // Implement proper GPU stats gathering
    }

    fn get_fan_info(&self, _fan_index: u32) -> Result<IOKitFanInfo> {
        Ok(IOKitFanInfo {
            speed_rpm: 1200,
            min_speed: 0,
            max_speed: 5000,
            percentage: 24.0,
        })
    }

    async fn get_fan_speed(&self, index: usize) -> Result<u32> {
        let key = format!("F{}Ac", index);
        let speed = self.read_smc_key(SmcKey::from_str(&key)?.to_chars())?;
        Ok(speed)
    }

    async fn get_fan_min_speed(&self, index: usize) -> Result<u32> {
        let key = format!("F{}Mn", index);
        let speed = self.read_smc_key(SmcKey::from_str(&key)?.to_chars())?;
        Ok(speed)
    }

    async fn get_fan_max_speed(&self, index: usize) -> Result<u32> {
        let key = format!("F{}Mx", index);
        let speed = self.read_smc_key(SmcKey::from_str(&key)?.to_chars())?;
        Ok(speed)
    }

    async fn get_fan_info(&self, index: u32) -> Result<Option<FanInfo>> {
        let service = unsafe {
            let matching = IOServiceMatching(b"AppleSMC\0".as_ptr() as *const i8);
            IOServiceGetMatchingService(kIOMasterPortDefault, matching)
        };

        if service == 0 {
            return Err(Error::hardware_error("Failed to get SMC service"));
        }

        let mut properties = std::ptr::null_mut();
        let result = unsafe {
            IORegistryEntryCreateCFProperties(
                service,
                &mut properties,
                std::ptr::null_mut(),
                0,
            )
        };

        if result != kIOReturnSuccess {
            return Err(Error::hardware_error("Failed to get fan properties"));
        }

        let dict = unsafe { SafeDictionary::from_cf_dictionary_ref(properties as *const _)? };
        
        let current_speed = self.get_fan_speed(index as usize).await?;
        let min_speed = self.get_fan_min_speed(index as usize).await?;
        let max_speed = self.get_fan_max_speed(index as usize).await?;

        // Calculate percentage
        let percentage = if max_speed > min_speed {
            ((current_speed - min_speed) as f64 / (max_speed - min_speed) as f64) * 100.0
        } else {
            0.0
        };

        Ok(Some(FanInfo {
            current_speed,
            target_speed: current_speed, // We don't have target speed info yet
            min_speed: Some(min_speed),
            max_speed: Some(max_speed),
            index: index as usize,
            speed_rpm: current_speed,
            percentage,
        }))
    }

    /// Read a key from the SMC
    fn read_smc_key(&self, key: [i8; 4]) -> Result<Option<f32>> {
        let mut input = SMCKeyData_t::default();
        input.key = smc_key_from_chars(key);
        input.data8 = SMC_CMD_READ_KEYINFO;

        let mut output = SMCKeyData_t::default();

        unsafe {
            let result = IOConnectCallStructMethod(
                self.connection,
                2,
                &input as *const _ as *const c_void,
                std::mem::size_of::<SMCKeyData_t>(),
                &mut output as *mut _ as *mut c_void,
                &mut std::mem::size_of::<SMCKeyData_t>(),
            );

            if result != kIOReturnSuccess {
                return Err(Error::hardware_error("Failed to read SMC key info"));
            }
        }

        input.data8 = SMC_CMD_READ_BYTES;

        unsafe {
            let result = IOConnectCallStructMethod(
                self.connection,
                2,
                &input as *const _ as *const c_void,
                std::mem::size_of::<SMCKeyData_t>(),
                &mut output as *mut _ as *mut c_void,
                &mut std::mem::size_of::<SMCKeyData_t>(),
            );

            if result != kIOReturnSuccess {
                return Err(Error::hardware_error("Failed to read SMC key data"));
            }

            Ok(Some(output.data32 as f32))
        }
    }

    fn get_battery_temperature(&self) -> Result<Option<f64>> {
        let info = self.get_thermal_info()?;
        Ok(info.battery_temp)
    }

    fn get_battery_info(&self) -> Result<SafeDictionary> {
        let matching = self.io_service_matching("AppleSmartBattery")?;
        let service = self.io_service_get_matching_service(&matching)?;
        self.io_registry_entry_create_cf_properties(&service)
    }

    fn get_cpu_info(&self) -> Result<SafeDictionary> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        self.io_registry_entry_create_cf_properties(&service)
    }

    fn get_number_property(&self, dict: &SafeDictionary, key: &str) -> Result<f64> {
        dict.get_number(key)
            .ok_or_else(|| Error::NotAvailable(format!("Property {} not available", key)))
    }

    fn io_connect_call_method(&self, connection: u32, selector: u32, input: &[u64], output: &mut [u64]) -> Result<()> {
        // Implement IOKit method call
        Ok(())
    }

    fn io_registry_entry_get_parent_entry(&self, entry: &ThreadSafeAnyObject) -> Result<ThreadSafeAnyObject> {
        let parent = unsafe {
            let mut parent = 0;
            let kr = IORegistryEntryGetParentEntry(entry.as_raw(), kIOServicePlane.as_ptr(), &mut parent);
            if kr != KERN_SUCCESS {
                return Err(Error::IOKitError("Failed to get parent entry".into()));
            }
            ThreadSafeAnyObject::new(parent)
        };
        Ok(parent)
    }

    fn get_physical_cores(&self) -> Result<usize> {
        let info = self.get_cpu_info()?;
        Ok(info.get_number("CPU_CORES").map(|x| x as usize).unwrap_or(1))
    }

    fn get_logical_cores(&self) -> Result<usize> {
        let info = self.get_cpu_info()?;
        Ok(info.get_number("CPU_THREADS").map(|x| x as usize).unwrap_or(1))
    }

    fn get_core_usage(&self) -> Result<Vec<f64>> {
        let info = self.get_cpu_info()?;
        let mut usage = Vec::new();
        let core_count = self.get_logical_cores()?;
        for i in 0..core_count {
            if let Some(core_usage) = info.get_number(&format!("CPU{}_USAGE", i)) {
                usage.push(core_usage);
            }
        }
        Ok(usage)
    }

    fn get_dictionary(&self, dict_ptr: *mut c_void) -> Result<SafeDictionary> {
        if dict_ptr.is_null() {
            return Err(Error::NullPointer("Dictionary pointer is null".into()));
        }
        unsafe { SafeDictionary::from_ptr(dict_ptr as *mut _) }
    }

    fn get_properties(&self, props: *mut c_void) -> Result<SafeDictionary> {
        self.get_dictionary(props)
    }

    fn get_battery_properties(&self, props: *mut c_void) -> Result<SafeDictionary> {
        self.get_dictionary(props)
    }

    fn get_battery_percentage(&self, dict: &SafeDictionary) -> Result<Option<f32>> {
        Ok(dict.get_number("CurrentCapacity")
            .map(|x| (x as f32 / dict.get_number("MaxCapacity").unwrap_or(100.0) as f32) * 100.0))
    }
}

impl Default for IOKitImpl {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Initialization flag to ensure classes are registered only once
static _INIT: Once = Once::new();

/// Ensures that Objective-C classes are registered for use
///
/// This is necessary for proper interoperability with the Objective-C runtime
fn ensure_classes_registered() {
    _INIT.call_once(|| {
        let _: &AnyClass = class!(NSObject);
        let _: &AnyClass = class!(NSMutableDictionary);
        let _: &AnyClass = class!(NSNumber);
    });
}

/// A registry entry in the IOKit registry
#[derive(Debug, Clone)]
pub struct IORegistryEntry {
    // Raw IOKit service ID
    service: u32,
}

impl IORegistryEntry {
    /// Create a new IORegistryEntry from a service ID
    pub fn new(service: u32) -> Self {
        Self { service }
    }

    /// Get the service ID
    pub fn service(&self) -> u32 {
        self.service
    }

    /// Convert to ThreadSafeAnyObject
    pub fn as_object(&self) -> ThreadSafeAnyObject {
        ThreadSafeAnyObject::new(self.service)
    }

    /// Get the properties of this registry entry
    pub fn get_properties(&self) -> Result<SafeDictionary> {
        let mut props: *mut c_void = std::ptr::null_mut();
        let result = unsafe {
            crate::utils::ffi::IORegistryEntryCreateCFProperties(self.service, &mut props, std::ptr::null_mut(), 0)
        };

        if result != 0 {
            return Err(Error::system_error("IORegistryEntryCreateCFProperties failed"));
        }

        Ok(unsafe { SafeDictionary::from_ptr(props as *mut _) })
    }
}

/// A thread-safe wrapper around an IOKit service
#[derive(Debug)]
pub struct ThreadSafeAnyObject {
    // Raw IOKit service ID
    handle: u32,
}

impl ThreadSafeAnyObject {
    /// Create a new ThreadSafeAnyObject
    pub fn new(handle: u32) -> Self {
        Self { handle }
    }

    /// Get the raw handle
    pub fn as_raw(&self) -> u32 {
        self.handle
    }
}

// Implement Send and Sync for ThreadSafeAnyObject since we're using it across threads
unsafe impl Send for ThreadSafeAnyObject {}
unsafe impl Sync for ThreadSafeAnyObject {}

impl Clone for ThreadSafeAnyObject {
    fn clone(&self) -> Self {
        Self { handle: self.handle }
    }
}

#[async_trait::async_trait]
impl CpuMonitor for IOKitImpl {
    async fn frequency(&self) -> Result<f64> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        props
            .get_number("CPU_Frequency")
            .ok_or_else(|| Error::system_error("CPU frequency not found"))
    }

    async fn min_frequency(&self) -> Result<f64> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        props
            .get_number("CPU_Min_Frequency")
            .ok_or_else(|| Error::system_error("CPU min frequency not found"))
    }

    async fn max_frequency(&self) -> Result<f64> {
        let matching = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties(&service)?;
        props
            .get_number("CPU_Max_Frequency")
            .ok_or_else(|| Error::system_error("CPU max frequency not found"))
    }

    async fn available_frequencies(&self) -> Result<Vec<f64>> {
        let service = self
            .get_service_matching("AppleSMC")?
            .ok_or_else(|| Error::not_found("AppleSMC service not found"))?;
        let props = self.get_service_properties(&service)?;

        let mut frequencies = Vec::new();
        if let Some(freq_array) = props.get_array("CPU_Available_Frequencies") {
            for freq in freq_array.iter() {
                if let Some(freq_num) = freq.as_number() {
                    frequencies.push(freq_num.as_f64());
                }
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
        props
            .get_string("CPU_Model")
            .ok_or_else(|| Error::system_error("CPU model name not found"))
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

#[async_trait]
impl ThermalMonitor for IOKitImpl {
    async fn cpu_temperature(&self) -> Result<Option<f64>> {
        let thermal_info = self.get_thermal_info()?;
        Ok(Some(thermal_info.cpu_temp))
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
        let thermal_info = self.get_thermal_info()?;
        Ok(thermal_info.thermal_throttling)
    }

    async fn get_fans(&self) -> Result<Vec<Fan>> {
        let fans = self.get_all_fans()?;
        Ok(fans.into_iter().map(Fan::from).collect())
    }

    async fn get_thermal_metrics(&self) -> Result<ThermalMetrics> {
        let thermal_info = self.get_thermal_info()?;
        let fans = self.get_all_fans()?;
        let fans_info: Vec<Fan> = fans.into_iter().map(Fan::from).collect();

        let thermal_level = if thermal_info.cpu_temp >= CPU_CRITICAL_TEMPERATURE {
            ThermalLevel::Critical
        } else if thermal_info.cpu_temp >= WARNING_TEMPERATURE_THRESHOLD {
            ThermalLevel::Warning
        } else {
            ThermalLevel::Normal
        };

        Ok(ThermalMetrics {
            fan_speeds: fans_info.iter().map(|f| f.speed_rpm).collect(),
            thermal_level,
            memory_temperature: thermal_info.heatsink_temp,
            is_throttling: thermal_info.thermal_throttling,
            fans: fans_info,
            cpu_temperature: Some(thermal_info.cpu_temp),
            gpu_temperature: thermal_info.gpu_temp,
            battery_temperature: thermal_info.battery_temp,
            ssd_temperature: None,
            ambient_temperature: thermal_info.ambient_temp,
        })
    }
}

#[async_trait::async_trait]
impl PowerConsumptionMonitor for IOKitImpl {
    async fn package_power(&self) -> Result<f32> {
        let matching = self.io_service_matching("IOPMrootDomain").await?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties("IOPMrootDomain").await?;
        Ok(props.and_then(|p| p.get_number("PCPP").map(|n| n as f32)).unwrap_or(0.0))
    }

    async fn cores_power(&self) -> Result<f32> {
        let matching = self.io_service_matching("IOPMrootDomain").await?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties("IOPMrootDomain").await?;
        Ok(props.and_then(|p| p.get_number("PCPC").map(|n| n as f32)).unwrap_or(0.0))
    }

    async fn gpu_power(&self) -> Result<Option<f32>> {
        let matching = self.io_service_matching("IOAccelerator").await?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties("IOAccelerator").await?;
        Ok(props.and_then(|p| p.get_number("GPUPower").map(|p| p as f32)))
    }

    async fn dram_power(&self) -> Result<Option<f32>> {
        let matching = self.io_service_matching("AppleSMC").await?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties("AppleSMC").await?;
        Ok(props.and_then(|p| p.get_number("PDRC").map(|p| p as f32)))
    }

    async fn neural_engine_power(&self) -> Result<Option<f32>> {
        let matching = self.io_service_matching("AppleSMC").await?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties("AppleSMC").await?;
        Ok(props.and_then(|p| p.get_number("PNEC").map(|p| p as f32)))
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

        let is_charging = battery_info.get_bool("ExternalConnected").unwrap_or(false);
        let is_charged = battery_info.get_bool("FullyCharged").unwrap_or(false);
        
        if is_charging && !is_charged {
            Ok(PowerState::Charging)
        } else if is_charging && is_charged {
            Ok(PowerState::AC)
        } else {
            Ok(PowerState::Battery)
        }
    }

    async fn battery_percentage(&self) -> Result<Option<f32>> {
        let battery_info = self.get_battery_info()?;
        if !battery_info.get_bool("BatteryInstalled").unwrap_or(false) {
            return Ok(None);
        }

        let current = battery_info.get_number("CurrentCapacity").unwrap_or(0.0);
        let max = battery_info.get_number("MaxCapacity").unwrap_or(100.0);
        
        if max > 0.0 {
            Ok(Some((current / max * 100.0) as f32))
        } else {
            Ok(None)
        }
    }

    async fn time_remaining(&self) -> Result<Option<u32>> {
        let battery_info = self.get_battery_info()?;
        if !battery_info.get_bool("BatteryInstalled").unwrap_or(false) {
            return Ok(None);
        }

        let is_charging = battery_info.get_bool("ExternalConnected").unwrap_or(false);
        let time_key = if is_charging { "TimeRemaining" } else { "TimeRemaining" };
        
        Ok(battery_info.get_number(time_key).map(|t| t as u32))
    }

    async fn is_on_battery(&self) -> Result<bool> {
        let state = self.power_state().await?;
        Ok(state == PowerState::Battery)
    }

    async fn is_charging(&self) -> Result<bool> {
        let state = self.power_state().await?;
        Ok(state == PowerState::Charging)
    }
}

#[async_trait::async_trait]
impl PowerManagementMonitor for IOKitImpl {
    async fn is_thermal_throttling(&self) -> Result<bool> {
        let matching = self.io_service_matching("IOPMrootDomain").await?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties("IOPMrootDomain").await?;
        Ok(props.and_then(|p| p.get_bool("ThermalStatus")).unwrap_or(false))
    }

    async fn power_impact(&self) -> Result<Option<f32>> {
        let matching = self.io_service_matching("IOPMrootDomain").await?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties("IOPMrootDomain").await?;
        Ok(props.and_then(|p| p.get_number("PowerImpact").map(|n| n as f32)))
    }

    async fn thermal_pressure(&self) -> Result<u32> {
        let matching = self.io_service_matching("IOPMrootDomain").await?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties("IOPMrootDomain").await?;
        Ok(props.and_then(|p| p.get_number("ThermalPressure").map(|n| n as u32)).unwrap_or(0))
    }

    async fn performance_mode(&self) -> Result<String> {
        let matching = self.io_service_matching("IOPMrootDomain").await?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties("IOPMrootDomain").await?;
        Ok(props.and_then(|p| p.get_string("PerformanceMode")).unwrap_or_else(|| "Unknown".to_string()))
    }
}

#[async_trait::async_trait]
impl PowerEventMonitor for IOKitImpl {
    async fn time_since_wake(&self) -> Result<Duration> {
        let matching = self.io_service_matching("IOPMrootDomain").await?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties("IOPMrootDomain").await?;
        let seconds = props.and_then(|p| p.get_number("TimeSinceWake").map(|n| n as u64)).unwrap_or(0);
        Ok(Duration::from_secs(seconds))
    }

    async fn thermal_event_count(&self) -> Result<u32> {
        let matching = self.io_service_matching("IOPMrootDomain").await?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties("IOPMrootDomain").await?;
        Ok(props.and_then(|p| p.get_number("ThermalEventCount").map(|n| n as u32)).unwrap_or(0))
    }

    async fn time_until_sleep(&self) -> Result<Option<Duration>> {
        let matching = self.io_service_matching("IOPMrootDomain").await?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties("IOPMrootDomain").await?;
        Ok(props.and_then(|p| p.get_number("SleepTimer").map(|n| Duration::from_secs(n as u64))))
    }

    async fn is_sleep_prevented(&self) -> Result<bool> {
        let matching = self.io_service_matching("IOPMrootDomain").await?;
        let service = self.io_service_get_matching_service(&matching)?;
        let props = self.get_service_properties("IOPMrootDomain").await?;
        Ok(props.and_then(|p| p.get_bool("PreventSleep")).unwrap_or(false))
    }
}

/// A wrapper around an IOKit service reference
#[derive(Debug)]
pub struct IOService {
    /// Service reference ID from IOKit
    service: u32,
}

/// A wrapper around an IOKit connection handle
#[derive(Debug)]
pub struct IOConnection {
    /// Connection handle reference from IOKit
    handle: u32,
}
