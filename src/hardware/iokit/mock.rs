use std::collections::HashMap;
use std::os::raw::c_char;
use std::sync::Mutex;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSDictionary, NSObject, NSString};

use crate::error::Result;
use crate::hardware::iokit::{FanInfo, GpuStats, IOKit, ThermalInfo};
use crate::utils::test_utils::{create_test_dictionary, create_test_object};

/// A simplified mock implementation of the IOKit trait for testing.
/// This implementation provides safe defaults and avoids thread safety issues.
#[derive(Debug)]
pub struct MockIOKit {
    // Only use primitive types to avoid sync issues
    cpu_temperature_result: Mutex<Result<f64>>,
    gpu_temperature_result: Mutex<Result<f64>>,
    gpu_stats_result: Mutex<Result<GpuStats>>,
    fan_speed_result: Mutex<Result<u32>>,
    fan_count_result: Mutex<Result<u32>>,
    fan_info_results: Mutex<HashMap<u32, Result<FanInfo>>>,
    all_fans_result: Mutex<Result<Vec<FanInfo>>>,
    heatsink_temperature_result: Mutex<Result<f64>>,
    ambient_temperature_result: Mutex<Result<f64>>,
    battery_temperature_result: Mutex<Result<f64>>,
    cpu_power_result: Mutex<Result<f64>>,
    thermal_throttling_result: Mutex<Result<bool>>,
    thermal_info_result: Mutex<Result<ThermalInfo>>,
    smc_key_results: Mutex<HashMap<[c_char; 4], Result<f64>>>,
    string_properties: Mutex<HashMap<String, Option<String>>>,
    number_properties: Mutex<HashMap<String, Option<i64>>>,
    bool_properties: Mutex<HashMap<String, Option<bool>>>,
}

impl Default for MockIOKit {
    fn default() -> Self {
        Self::new()
    }
}

impl MockIOKit {
    /// Creates a new MockIOKit with default expectations.
    pub fn new() -> Self {
        MockIOKit {
            cpu_temperature_result: Mutex::new(Ok(45.0)),
            gpu_temperature_result: Mutex::new(Ok(55.0)),
            gpu_stats_result: Mutex::new(Ok(GpuStats {
                utilization: 50.0,
                perf_cap: 50.0,
                perf_threshold: 100.0,
                memory_used: 1024 * 1024 * 1024,      // 1 GB
                memory_total: 4 * 1024 * 1024 * 1024, // 4 GB
                name: "Test GPU".to_string(),
            })),
            fan_speed_result: Mutex::new(Ok(2000)),
            fan_count_result: Mutex::new(Ok(2)),
            fan_info_results: Mutex::new(HashMap::new()),
            all_fans_result: Mutex::new(Ok(vec![
                FanInfo {
                    speed_rpm: 2000,
                    min_speed: 500,
                    max_speed: 5000,
                    percentage: 40.0,
                },
                FanInfo {
                    speed_rpm: 1800,
                    min_speed: 400,
                    max_speed: 4500,
                    percentage: 35.0,
                },
            ])),
            heatsink_temperature_result: Mutex::new(Ok(40.0)),
            ambient_temperature_result: Mutex::new(Ok(25.0)),
            battery_temperature_result: Mutex::new(Ok(35.0)),
            cpu_power_result: Mutex::new(Ok(15.0)),
            thermal_throttling_result: Mutex::new(Ok(false)),
            thermal_info_result: Mutex::new(Ok(ThermalInfo {
                cpu_temp: 45.0,
                gpu_temp: 55.0,
                heatsink_temp: Some(40.0),
                ambient_temp: Some(25.0),
                battery_temp: Some(35.0),
                is_throttling: false,
                cpu_power: Some(15.0),
            })),
            smc_key_results: Mutex::new(HashMap::new()),
            string_properties: Mutex::new(HashMap::new()),
            number_properties: Mutex::new(HashMap::new()),
            bool_properties: Mutex::new(HashMap::new()),
        }
    }

    // Methods to set expectations
    pub fn expect_get_string_property(&self, key: &str, result: Option<String>) {
        if let Ok(mut map) = self.string_properties.lock() {
            map.insert(key.to_string(), result);
        }
    }

    pub fn expect_get_number_property(&self, key: &str, result: Option<i64>) {
        if let Ok(mut map) = self.number_properties.lock() {
            map.insert(key.to_string(), result);
        }
    }

    pub fn expect_get_bool_property(&self, key: &str, result: Option<bool>) {
        if let Ok(mut map) = self.bool_properties.lock() {
            map.insert(key.to_string(), result);
        }
    }

    pub fn expect_get_cpu_temperature(&self, result: Result<f64>) {
        if let Ok(mut val) = self.cpu_temperature_result.lock() {
            *val = result;
        }
    }

    pub fn expect_get_gpu_temperature(&self, result: Result<f64>) {
        if let Ok(mut val) = self.gpu_temperature_result.lock() {
            *val = result;
        }
    }

    pub fn expect_get_gpu_stats(&self, result: Result<GpuStats>) {
        if let Ok(mut val) = self.gpu_stats_result.lock() {
            *val = result;
        }
    }

    pub fn expect_get_fan_speed(&self, result: Result<u32>) {
        if let Ok(mut val) = self.fan_speed_result.lock() {
            *val = result;
        }
    }

    pub fn expect_get_fan_count(&self, result: Result<u32>) {
        if let Ok(mut val) = self.fan_count_result.lock() {
            *val = result;
        }
    }

    pub fn expect_get_fan_info(&self, fan_index: u32, result: Result<FanInfo>) {
        if let Ok(mut map) = self.fan_info_results.lock() {
            map.insert(fan_index, result);
        }
    }

    pub fn expect_get_all_fans(&self, result: Result<Vec<FanInfo>>) {
        if let Ok(mut val) = self.all_fans_result.lock() {
            *val = result;
        }
    }

    pub fn expect_get_heatsink_temperature(&self, result: Result<f64>) {
        if let Ok(mut val) = self.heatsink_temperature_result.lock() {
            *val = result;
        }
    }

    pub fn expect_get_ambient_temperature(&self, result: Result<f64>) {
        if let Ok(mut val) = self.ambient_temperature_result.lock() {
            *val = result;
        }
    }

    pub fn expect_get_battery_temperature(&self, result: Result<f64>) {
        if let Ok(mut val) = self.battery_temperature_result.lock() {
            *val = result;
        }
    }

    pub fn expect_get_cpu_power(&self, result: Result<f64>) {
        if let Ok(mut val) = self.cpu_power_result.lock() {
            *val = result;
        }
    }

    pub fn expect_check_thermal_throttling(&self, result: Result<bool>) {
        if let Ok(mut val) = self.thermal_throttling_result.lock() {
            *val = result;
        }
    }

    pub fn expect_get_thermal_info(&self, result: Result<ThermalInfo>) {
        if let Ok(mut val) = self.thermal_info_result.lock() {
            *val = result;
        }
    }

    pub fn expect_read_smc_key(&self, key: [c_char; 4], result: Result<f64>) {
        if let Ok(mut map) = self.smc_key_results.lock() {
            map.insert(key, result);
        }
    }
}

unsafe impl Send for MockIOKit {}
unsafe impl Sync for MockIOKit {}

impl IOKit for MockIOKit {
    fn io_service_matching(&self, _service_name: &str) -> Retained<NSDictionary<NSString, NSObject>> {
        create_test_dictionary()
    }

    fn io_service_get_matching_service(
        &self,
        _matching: &NSDictionary<NSString, NSObject>,
    ) -> Option<Retained<AnyObject>> {
        None
    }

    fn io_registry_entry_create_cf_properties(
        &self,
        _entry: &AnyObject,
    ) -> Result<Retained<NSDictionary<NSString, NSObject>>> {
        Ok(create_test_dictionary())
    }

    fn io_object_release(&self, _obj: &AnyObject) {
        // No-op for mock
    }

    fn get_string_property(
        &self,
        _dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<String> {
        if let Ok(map) = self.string_properties.lock() {
            map.get(key).cloned().unwrap_or(None)
        } else {
            None
        }
    }

    fn get_number_property(
        &self,
        _dict: &NSDictionary<NSString, NSObject>,
        key: &str,
    ) -> Option<i64> {
        if let Ok(map) = self.number_properties.lock() {
            map.get(key).cloned().unwrap_or(None)
        } else {
            None
        }
    }

    fn get_bool_property(&self, _dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<bool> {
        if let Ok(map) = self.bool_properties.lock() {
            map.get(key).cloned().unwrap_or(None)
        } else {
            None
        }
    }

    fn get_dict_property(
        &self,
        _dict: &NSDictionary<NSString, NSObject>,
        _key: &str,
    ) -> Option<Retained<NSDictionary<NSString, NSObject>>> {
        None
    }

    fn get_service(&self, _name: &str) -> Result<Retained<AnyObject>> {
        Ok(create_test_object())
    }

    fn io_registry_entry_get_parent(&self, _entry: &AnyObject) -> Option<Retained<AnyObject>> {
        None
    }

    // Temperature related methods
    fn get_cpu_temperature(&self) -> Result<f64> {
        if let Ok(result) = self.cpu_temperature_result.lock() {
            (*result).clone()
        } else {
            Ok(45.0)
        }
    }

    fn get_gpu_temperature(&self) -> Result<f64> {
        if let Ok(result) = self.gpu_temperature_result.lock() {
            (*result).clone()
        } else {
            Ok(55.0)
        }
    }

    fn get_gpu_stats(&self) -> Result<GpuStats> {
        if let Ok(result) = self.gpu_stats_result.lock() {
            (*result).clone()
        } else {
            Ok(GpuStats {
                utilization: 50.0,
                perf_cap: 50.0,
                perf_threshold: 100.0,
                memory_used: 1024 * 1024 * 1024,      // 1 GB
                memory_total: 4 * 1024 * 1024 * 1024, // 4 GB
                name: "Test GPU".to_string(),
            })
        }
    }

    // Fan related methods
    fn get_fan_speed(&self) -> Result<u32> {
        if let Ok(result) = self.fan_speed_result.lock() {
            (*result).clone()
        } else {
            Ok(2000)
        }
    }

    fn get_fan_count(&self) -> Result<u32> {
        if let Ok(result) = self.fan_count_result.lock() {
            (*result).clone()
        } else {
            Ok(2)
        }
    }

    fn get_fan_info(&self, fan_index: u32) -> Result<FanInfo> {
        if let Ok(map) = self.fan_info_results.lock() {
            if let Some(result) = map.get(&fan_index) {
                return (*result).clone();
            }
        }
        
        Ok(FanInfo {
            speed_rpm: 2000,
            min_speed: 500,
            max_speed: 5000,
            percentage: 40.0,
        })
    }

    fn get_all_fans(&self) -> Result<Vec<FanInfo>> {
        if let Ok(result) = self.all_fans_result.lock() {
            (*result).clone()
        } else {
            Ok(vec![
                FanInfo {
                    speed_rpm: 2000,
                    min_speed: 500,
                    max_speed: 5000,
                    percentage: 40.0,
                },
                FanInfo {
                    speed_rpm: 1800,
                    min_speed: 400,
                    max_speed: 4500,
                    percentage: 35.0,
                },
            ])
        }
    }

    // Advanced thermal methods
    fn get_heatsink_temperature(&self) -> Result<f64> {
        if let Ok(result) = self.heatsink_temperature_result.lock() {
            (*result).clone()
        } else {
            Ok(40.0)
        }
    }

    fn get_ambient_temperature(&self) -> Result<f64> {
        if let Ok(result) = self.ambient_temperature_result.lock() {
            (*result).clone()
        } else {
            Ok(25.0)
        }
    }

    fn get_battery_temperature(&self) -> Result<f64> {
        if let Ok(result) = self.battery_temperature_result.lock() {
            (*result).clone()
        } else {
            Ok(35.0)
        }
    }

    fn get_cpu_power(&self) -> Result<f64> {
        if let Ok(result) = self.cpu_power_result.lock() {
            (*result).clone()
        } else {
            Ok(15.0)
        }
    }

    fn check_thermal_throttling(&self) -> Result<bool> {
        if let Ok(result) = self.thermal_throttling_result.lock() {
            (*result).clone()
        } else {
            Ok(false)
        }
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo> {
        if let Ok(result) = self.thermal_info_result.lock() {
            (*result).clone()
        } else {
            Ok(ThermalInfo {
                cpu_temp: 45.0,
                gpu_temp: 55.0,
                heatsink_temp: Some(40.0),
                ambient_temp: Some(25.0),
                battery_temp: Some(35.0),
                is_throttling: false,
                cpu_power: Some(15.0),
            })
        }
    }

    fn read_smc_key(&self, key: [c_char; 4]) -> Result<f64> {
        if let Ok(map) = self.smc_key_results.lock() {
            if let Some(result) = map.get(&key) {
                return (*result).clone();
            }
        }
        
        Ok(42.0)
    }
}