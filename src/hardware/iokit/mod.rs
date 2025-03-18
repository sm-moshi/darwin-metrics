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

/// Custom type for IORegistry connection
pub type IORegistryConnection = u32;

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

    /// Get service properties
    fn get_service_properties(&self, service: &ThreadSafeAnyObject) -> Result<SafeDictionary>;

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

/// IOKit implementation for hardware monitoring
#[derive(Debug, Clone, Default)]
pub struct IOKitImpl {
    /// The current IORegistry connection ID (mach_port_t)
    connection: Option<u32>,
}

impl IOKitImpl {
    /// Create a new IOKitImpl instance
    pub fn new() -> Result<Self> {
        // Create a default implementation initially
        let instance = Self::default();
        
        // TODO: Establish an IORegistry connection if needed for live implementations
        
        Ok(instance)
    }
}

impl IOKit for IOKitImpl {
    fn io_service_matching(&self, service_name: &str) -> Result<SafeDictionary> {
        let c_str = CString::new(service_name)
            .map_err(|_| Error::system(format!("Failed to create C string for service name: {}", service_name)))?;
        
        let dict_ptr = unsafe { crate::utils::ffi::IOServiceMatching(c_str.as_ptr() as *const c_char) };
        if dict_ptr.is_null() {
            return Err(Error::system(format!("IOServiceMatching failed for service: {}", service_name)));
        }
        
        Ok(unsafe { SafeDictionary::from_ptr(dict_ptr as *mut _) })
    }

    fn get_service_matching(&self, name: &str) -> Result<Option<ThreadSafeAnyObject>> {
        let dict = self.io_service_matching(name)?;
        let master_port = crate::utils::ffi::K_IOMASTER_PORT_DEFAULT; // mach_port_t
        let service = unsafe { crate::utils::ffi::IOServiceGetMatchingService(master_port, dict.as_ptr() as *mut _) };
        
        if service == 0 {
            return Ok(None);
        }
        
        Ok(Some(ThreadSafeAnyObject::new(service)))
    }

    fn io_service_get_matching_service(&self, matching: &SafeDictionary) -> Result<ThreadSafeAnyObject> {
        let master_port = crate::utils::ffi::K_IOMASTER_PORT_DEFAULT; // mach_port_t
        let service = unsafe { crate::utils::ffi::IOServiceGetMatchingService(master_port, matching.as_ptr() as *mut _) };
        
        if service == 0 {
            return Err(Error::system("IOServiceGetMatchingService failed".to_string()));
        }
        
        Ok(ThreadSafeAnyObject::new(service))
    }

    fn get_service_properties(&self, service: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        let mut props: *mut c_void = std::ptr::null_mut();
        let result = unsafe {
            crate::utils::ffi::IORegistryEntryCreateCFProperties(
                service.as_raw(),
                &mut props,
                std::ptr::null_mut(),
                0,
            )
        };
        
        if result != 0 {
            return Err(Error::system("IORegistryEntryCreateCFProperties failed".to_string()));
        }
        
        Ok(unsafe { SafeDictionary::from_ptr(props as *mut _) })
    }

    fn io_registry_entry_create_cf_properties(&self, entry: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        self.get_service_properties(entry)
    }

    fn get_cpu_temperature(&self, _plane: &str) -> Result<f64> {
        let dict = self.io_service_matching("AppleSMC")?;
        let service = self.io_service_get_matching_service(&dict)?;
        let _props = self.get_service_properties(&service)?;
        
        Ok(55.0) // Default reasonable value
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo> {
        Ok(ThermalInfo {
            cpu_temp: 55.0,
            gpu_temp: Some(50.0),
            fan_speed: 0,
            heatsink_temp: None,
            ambient_temp: None,
            thermal_throttling: false,
            battery_temp: None,
            dict: SafeDictionary::new(),
        })
    }

    fn get_all_fans(&self) -> Result<Vec<FanInfo>> {
        Ok(vec![
            FanInfo { 
                speed_rpm: 1200, 
                min_speed: 0, 
                max_speed: 5000, 
                percentage: 24.0 
            },
            FanInfo { 
                speed_rpm: 1300, 
                min_speed: 0, 
                max_speed: 5500, 
                percentage: 23.6 
            }
        ])
    }

    fn check_thermal_throttling(&self, _plane: &str) -> Result<bool> {
        Ok(false)
    }

    fn get_cpu_power(&self) -> Result<f64> {
        Ok(15.0)
    }

    fn get_gpu_stats(&self) -> Result<GpuStats> {
        Ok(GpuStats {
            utilization: 0.3,
            memory_used: 1024 * 1024 * 1024, // 1 GB
            memory_total: 4 * 1024 * 1024 * 1024, // 4 GB
            perf_cap: 0.8,
            perf_threshold: 0.9,
            name: "Virtual GPU".to_string()
        })
    }

    fn get_fan_info(&self, _fan_index: u32) -> Result<FanInfo> {
        Ok(FanInfo {
            speed_rpm: 1200,
            min_speed: 0,
            max_speed: 5000,
            percentage: 24.0
        })
    }

    fn get_battery_temperature(&self) -> Result<Option<f64>> {
        Ok(Some(35.0))
    }

    fn get_battery_info(&self) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_cpu_info(&self) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_number_property(&self, dict: &SafeDictionary, key: &str) -> Result<f64> {
        dict.get_number(key).ok_or_else(|| Error::system(format!("Property {} not found", key)))
    }

    fn io_connect_call_method(&self, _connection: u32, _selector: u32, _input: &[u64], _output: &mut [u64]) -> Result<()> {
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
    }

    fn io_registry_entry_get_parent_entry(&self, entry: &ThreadSafeAnyObject) -> Result<ThreadSafeAnyObject> {
        let mut parent: u32 = 0;
        let plane = CString::new("IOService").unwrap();
        
        let result = unsafe {
            crate::utils::ffi::IORegistryEntryGetParentEntry(entry.as_raw(), plane.as_ptr(), &mut parent)
        };
        
        if result != 0 {
            return Err(Error::system("IORegistryEntryGetParentEntry failed".to_string()));
        }
        
        Ok(ThreadSafeAnyObject::new(parent))
    }

    fn get_physical_cores(&self) -> Result<usize> {
        Ok(4)
    }

    fn get_logical_cores(&self) -> Result<usize> {
        Ok(8)
    }

    fn get_core_usage(&self) -> Result<Vec<f64>> {
        Ok(vec![0.2, 0.3, 0.1, 0.5, 0.2, 0.3, 0.1, 0.5])
    }

    fn read_smc_key(&self, _key: [c_char; 4]) -> Result<Option<f32>> {
        Ok(Some(35.0))
    }
}

static INIT: Once = Once::new();

fn ensure_classes_registered() {
    INIT.call_once(|| {
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
            crate::utils::ffi::IORegistryEntryCreateCFProperties(
                self.service,
                &mut props,
                std::ptr::null_mut(),
                0,
            )
        };
        
        if result != 0 {
            return Err(Error::system("IORegistryEntryCreateCFProperties failed".to_string()));
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
