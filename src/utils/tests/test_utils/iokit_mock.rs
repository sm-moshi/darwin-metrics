/// Mock implementation of IOKit for testing purposes.
use objc2::{class, runtime::AnyClass};
use objc2_foundation::NSObject;
use std::sync::{Arc, Mutex, Once};

use crate::{
    error::{Error, Result},
    hardware::iokit::{FanInfo, GpuStats, IOKit, ThermalInfo, ThreadSafeAnyObject},
    utils::core::dictionary::SafeDictionary,
};

use std::ffi::{CStr, CString};
use std::fmt::Debug;
use std::os::raw::{c_char, c_void};

static INIT: Once = Once::new();

fn ensure_classes_registered() {
    INIT.call_once(|| {
        let _: &AnyClass = class!(NSObject);
        let _: &AnyClass = class!(NSMutableDictionary);
        let _: &AnyClass = class!(NSNumber);
        let _: &AnyClass = class!(NSString);
    });
}

/// A simple mock implementation of IOKit for testing
#[derive(Debug)]
pub struct MockIOKit {
    physical_cores: usize,
    logical_cores: usize,
    core_usage: Mutex<Vec<f64>>,
    temperature: f64,
    battery_temp: Option<f64>,
    battery_info: BatteryInfo,
    cpu_info: CpuInfo,
    thermal_info: ThermalInfo,
}

#[derive(Debug, Clone)]
struct BatteryInfo {
    present: bool,
    is_charging: bool,
    cycle_count: i64,
    percentage: f64,
    temperature: f64,
    time_remaining: i64,
    design_capacity: f64,
    current_capacity: f64,
}

#[derive(Debug, Clone)]
struct CpuInfo {
    model_name: String,
    frequency: f64,
    min_frequency: Option<f64>,
    max_frequency: Option<f64>,
    available_frequencies: Option<Vec<f64>>,
}

impl Default for BatteryInfo {
    fn default() -> Self {
        Self {
            present: false,
            is_charging: false,
            cycle_count: 0,
            percentage: 0.0,
            temperature: 0.0,
            time_remaining: 0,
            design_capacity: 0.0,
            current_capacity: 0.0,
        }
    }
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self {
            model_name: String::new(),
            frequency: 0.0,
            min_frequency: None,
            max_frequency: None,
            available_frequencies: None,
        }
    }
}

impl MockIOKit {
    pub fn new() -> Result<Self> {
        ensure_classes_registered();
        Ok(Self {
            physical_cores: 0,
            logical_cores: 0,
            core_usage: Mutex::new(Vec::new()),
            temperature: 0.0,
            battery_temp: None,
            battery_info: BatteryInfo::default(),
            cpu_info: CpuInfo::default(),
            thermal_info: ThermalInfo::default(),
        })
    }

    pub fn with_physical_cores(mut self, cores: usize) -> Result<Self> {
        self.physical_cores = cores;
        Ok(self)
    }

    pub fn with_logical_cores(mut self, cores: usize) -> Result<Self> {
        self.logical_cores = cores;
        Ok(self)
    }

    pub fn with_core_usage(self, usage: Vec<f64>) -> Result<Self> {
        *self.core_usage.lock().unwrap() = usage;
        Ok(self)
    }

    pub fn with_temperature(mut self, temp: f64) -> Result<Self> {
        self.temperature = temp;
        self.thermal_info.cpu_temp = temp;
        Ok(self)
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
    ) -> Result<Self> {
        self.battery_info = BatteryInfo {
            present: is_present,
            is_charging,
            cycle_count,
            percentage,
            temperature,
            time_remaining,
            design_capacity,
            current_capacity,
        };
        self.battery_temp = Some(temperature);
        self.thermal_info.battery_temp = Some(temperature);
        Ok(self)
    }

    pub fn with_cpu_info(
        mut self,
        model_name: String,
        frequency: f64,
        min_frequency: Option<f64>,
        max_frequency: Option<f64>,
        available_frequencies: Option<Vec<f64>>,
    ) -> Result<Self> {
        self.cpu_info = CpuInfo { model_name, frequency, min_frequency, max_frequency, available_frequencies };
        Ok(self)
    }

    pub fn set_physical_cores(&mut self, cores: usize) {
        self.physical_cores = cores;
    }

    pub fn set_logical_cores(&mut self, cores: usize) {
        self.logical_cores = cores;
    }

    pub fn set_core_usage(&self, usage: Vec<f64>) -> Result<()> {
        *self.core_usage.lock().unwrap() = usage;
        Ok(())
    }

    pub fn set_temperature(&mut self, temp: f64) {
        self.temperature = temp;
        self.thermal_info.cpu_temp = temp;
    }

    pub fn set_battery_temperature(&mut self, temp: Option<f64>) {
        self.battery_temp = temp;
        self.thermal_info.battery_temp = temp;
    }

    pub fn set_battery_info(&mut self, info: BatteryInfo) {
        self.battery_info = info;
    }

    pub fn set_cpu_info(&mut self, info: CpuInfo) {
        self.cpu_info = info;
    }

    pub fn set_thermal_info(&mut self, info: ThermalInfo) {
        self.thermal_info = info;
    }
}

impl IOKit for MockIOKit {
    fn get_service_matching(&self, _name: &str) -> Result<Option<ThreadSafeAnyObject>> {
        Ok(Some(ThreadSafeAnyObject::new(1)))
    }

    fn io_service_matching(&self, _name: &str) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn io_service_get_matching_service(&self, _matching_dict: &SafeDictionary) -> Result<ThreadSafeAnyObject> {
        Ok(ThreadSafeAnyObject::new(1))
    }

    fn io_registry_entry_create_cf_properties(&self, _entry: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_cpu_temperature(&self, _plane: &str) -> Result<f64> {
        Ok(self.temperature)
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo> {
        Ok(self.thermal_info.clone())
    }

    fn get_all_fans(&self) -> Result<Vec<FanInfo>> {
        Ok(Vec::new())
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
            name: String::from("Mock GPU"),
        })
    }

    fn get_gpu_stats_multiple(&self) -> Result<Vec<GpuStats>> {
        Ok(vec![
            GpuStats {
                utilization: 0.0,
                memory_used: 0,
                memory_total: 0,
                perf_cap: 0.0,
                perf_threshold: 0.0,
                name: String::from("Mock GPU 1"),
            },
            GpuStats {
                utilization: 0.0,
                memory_used: 0,
                memory_total: 0,
                perf_cap: 0.0,
                perf_threshold: 0.0,
                name: String::from("Mock GPU 2"),
            },
        ])
    }

    fn get_fan_info(&self, _fan_index: u32) -> Result<FanInfo> {
        Ok(FanInfo::default())
    }

    fn get_battery_temperature(&self) -> Result<Option<f64>> {
        Ok(self.battery_temp)
    }

    fn get_battery_info(&self) -> Result<SafeDictionary> {
        let mut dict = SafeDictionary::new();
        dict.set_bool("BatteryInstalled", self.battery_info.present);
        dict.set_bool("ExternalConnected", self.battery_info.is_charging);
        dict.set_i64("CycleCount", self.battery_info.cycle_count);
        dict.set_f64("CurrentCapacity", self.battery_info.current_capacity);
        dict.set_f64("MaxCapacity", self.battery_info.design_capacity);
        dict.set_f64("Temperature", self.battery_info.temperature);
        dict.set_i64("TimeRemaining", self.battery_info.time_remaining);
        Ok(dict)
    }

    fn get_cpu_info(&self) -> Result<SafeDictionary> {
        let mut dict = SafeDictionary::new();
        dict.set_f64("frequency", self.cpu_info.frequency);
        if let Some(min_freq) = self.cpu_info.min_frequency {
            dict.set_f64("min_frequency", min_freq);
        }
        if let Some(max_freq) = self.cpu_info.max_frequency {
            dict.set_f64("max_frequency", max_freq);
        }
        Ok(dict)
    }

    fn get_number_property(&self, dict: &SafeDictionary, key: &str) -> Result<f64> {
        dict.get_number(key)
            .ok_or_else(|| Error::NotAvailable { resource: key.to_string(), reason: "Property not found".to_string() })
    }

    fn io_connect_call_method(
        &self,
        _connection: u32,
        _selector: u32,
        _input: &[u64],
        _output: &mut [u64],
    ) -> Result<()> {
        Err(Error::NotAvailable { resource: "IOConnect".to_string(), reason: "Mock implementation".to_string() })
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
    }

    fn io_registry_entry_get_parent_entry(&self, _entry: &ThreadSafeAnyObject) -> Result<ThreadSafeAnyObject> {
        Ok(ThreadSafeAnyObject::new(1))
    }

    fn get_service_properties(&self, _service: &ThreadSafeAnyObject) -> Result<SafeDictionary> {
        Ok(SafeDictionary::new())
    }

    fn get_physical_cores(&self) -> Result<usize> {
        Ok(self.physical_cores)
    }

    fn get_logical_cores(&self) -> Result<usize> {
        Ok(self.logical_cores)
    }

    fn get_core_usage(&self) -> Result<Vec<f64>> {
        Ok(self.core_usage.lock().unwrap().clone())
    }

    fn read_smc_key(&self, _key: [c_char; 4]) -> Result<Option<f32>> {
        // Mock implementation that returns a simulated temperature value
        Ok(Some(45.0))
    }
}

impl Clone for MockIOKit {
    fn clone(&self) -> Self {
        Self {
            physical_cores: self.physical_cores,
            logical_cores: self.logical_cores,
            core_usage: Mutex::new(self.core_usage.lock().unwrap().clone()),
            temperature: self.temperature,
            battery_temp: self.battery_temp,
            battery_info: self.battery_info.clone(),
            cpu_info: self.cpu_info.clone(),
            thermal_info: self.thermal_info.clone(),
        }
    }
}

unsafe impl Send for MockIOKit {}
unsafe impl Sync for MockIOKit {}
