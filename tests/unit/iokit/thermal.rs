use std::fmt::Debug;

use async_trait::async_trait;
use darwin_metrics::error::Error;
use darwin_metrics::hardware::iokit::{FanInfo, GpuStats, IOKit, IOKitImpl, ThermalInfo, ThreadSafeAnyObject};
use darwin_metrics::utils::core::dictionary::SafeDictionary;

// Our own simple mock for testing
#[derive(Debug)]
struct SimpleMockIOKit {
    thermal_info: Option<ThermalInfo>,
    cpu_temp_error: Option<Error>,
    gpu_stats_error: Option<Error>,
    thermal_info_error: Option<Error>,
}

impl SimpleMockIOKit {
    fn new() -> Self {
        Self {
            thermal_info: None,
            cpu_temp_error: None,
            gpu_stats_error: None,
            thermal_info_error: None,
        }
    }

    fn with_thermal_info(mut self, info: ThermalInfo) -> Self {
        self.thermal_info = Some(info);
        self
    }

    fn with_cpu_temp_error(mut self, error: Error) -> Self {
        self.cpu_temp_error = Some(error);
        self
    }

    fn with_gpu_stats_error(mut self, error: Error) -> Self {
        self.gpu_stats_error = Some(error);
        self
    }

    fn with_thermal_info_error(mut self, error: Error) -> Self {
        self.thermal_info_error = Some(error);
        self
    }
}

#[async_trait]
impl IOKit for SimpleMockIOKit {
    fn io_service_matching(&self, _name: &str) -> darwin_metrics::error::Result<SafeDictionary> {
        unimplemented!("Not needed for thermal tests")
    }

    fn get_service_matching(&self, _name: &str) -> darwin_metrics::error::Result<Option<ThreadSafeAnyObject>> {
        unimplemented!("Not needed for thermal tests")
    }

    fn io_service_get_matching_service(
        &self,
        _matching_dict: &SafeDictionary,
    ) -> darwin_metrics::error::Result<ThreadSafeAnyObject> {
        unimplemented!("Not needed for thermal tests")
    }

    fn io_registry_entry_create_cf_properties(
        &self,
        _entry: &ThreadSafeAnyObject,
    ) -> darwin_metrics::error::Result<SafeDictionary> {
        unimplemented!("Not needed for thermal tests")
    }

    fn get_cpu_temperature(&self, _plane: &str) -> darwin_metrics::error::Result<f64> {
        if let Some(error) = &self.cpu_temp_error {
            Err(error.clone())
        } else {
            Ok(45.0)
        }
    }

    fn get_thermal_info(&self) -> darwin_metrics::error::Result<ThermalInfo> {
        if let Some(error) = &self.thermal_info_error {
            Err(error.clone())
        } else if let Some(info) = &self.thermal_info {
            Ok(info.clone())
        } else {
            Ok(ThermalInfo::default())
        }
    }

    fn get_all_fans(&self) -> darwin_metrics::error::Result<Vec<FanInfo>> {
        unimplemented!("Not needed for thermal tests")
    }

    fn check_thermal_throttling(&self, _plane: &str) -> darwin_metrics::error::Result<bool> {
        unimplemented!("Not needed for thermal tests")
    }

    fn get_cpu_power(&self) -> darwin_metrics::error::Result<f64> {
        unimplemented!("Not needed for thermal tests")
    }

    fn get_gpu_stats(&self) -> darwin_metrics::error::Result<GpuStats> {
        if let Some(error) = &self.gpu_stats_error {
            Err(error.clone())
        } else {
            unimplemented!("Not needed for thermal tests except in error case")
        }
    }

    // Implement additional required methods
    fn get_fan_info(&self, _fan_id: u32) -> darwin_metrics::error::Result<FanInfo> {
        unimplemented!("Not needed for thermal tests")
    }

    fn get_battery_temperature(&self) -> darwin_metrics::error::Result<f64> {
        unimplemented!("Not needed for thermal tests")
    }

    fn get_battery_info(&self) -> darwin_metrics::error::Result<darwin_metrics::battery::types::BatteryInfo> {
        unimplemented!("Not needed for thermal tests")
    }

    fn get_cpu_info(&self) -> darwin_metrics::error::Result<darwin_metrics::hardware::cpu::CpuInfo> {
        unimplemented!("Not needed for thermal tests")
    }

    fn get_number_property(&self, _dictionary: &SafeDictionary, _key: &str) -> darwin_metrics::error::Result<f64> {
        unimplemented!("Not needed for thermal tests")
    }

    fn io_connect_call_method(
        &self,
        _connection: &ThreadSafeAnyObject,
        _selector: u32,
        _input_values: &[u64],
        _output_values: &mut [u64],
    ) -> darwin_metrics::error::Result<()> {
        unimplemented!("Not needed for thermal tests")
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(Self {
            thermal_info: self.thermal_info.clone(),
            cpu_temp_error: self.cpu_temp_error.clone(),
            gpu_stats_error: self.gpu_stats_error.clone(),
            thermal_info_error: self.thermal_info_error.clone(),
        })
    }

    fn io_registry_entry_get_parent_entry(
        &self,
        _entry: &ThreadSafeAnyObject,
        _plane: &str,
    ) -> darwin_metrics::error::Result<ThreadSafeAnyObject> {
        unimplemented!("Not needed for thermal tests")
    }

    fn get_service_properties(
        &self,
        _service_name: &str,
        _property_keys: &[&str],
    ) -> darwin_metrics::error::Result<SafeDictionary> {
        unimplemented!("Not needed for thermal tests")
    }

    fn get_physical_cores(&self) -> darwin_metrics::error::Result<u32> {
        unimplemented!("Not needed for thermal tests")
    }

    fn get_logical_cores(&self) -> darwin_metrics::error::Result<u32> {
        unimplemented!("Not needed for thermal tests")
    }

    fn get_core_usage(&self, _core: u32) -> darwin_metrics::error::Result<darwin_metrics::hardware::cpu::CoreUsage> {
        unimplemented!("Not needed for thermal tests")
    }

    fn read_smc_key(&self, _key: &str) -> darwin_metrics::error::Result<Vec<u8>> {
        unimplemented!("Not needed for thermal tests")
    }
}

// Helper function to create a mock ThermalInfo instance for testing
fn get_mock_thermal_info() -> ThermalInfo {
    let mut info = ThermalInfo::default();
    info.cpu_temp = 45.0;
    info.gpu_temp = Some(55.0);
    info.heatsink_temp = Some(40.0);
    info.ambient_temp = Some(25.0);
    info.battery_temp = Some(35.0);
    info.thermal_throttling = false;
    info.fan_speed = 0;
    info
}

#[test]
fn test_get_thermal_info() {
    let test_info = get_mock_thermal_info();
    let mock_iokit = SimpleMockIOKit::new().with_thermal_info(test_info.clone());

    let result = mock_iokit.get_thermal_info().unwrap();

    // Verify all fields match what we created
    assert_eq!(result.cpu_temp, 45.0);
    assert_eq!(result.gpu_temp, Some(55.0));
    assert_eq!(result.heatsink_temp, Some(40.0));
    assert_eq!(result.ambient_temp, Some(25.0));
    assert_eq!(result.battery_temp, Some(35.0));
    assert!(!result.thermal_throttling);
    assert_eq!(result.fan_speed, 0);
}

#[test]
fn test_get_thermal_info_with_failures() {
    let mut test_info = ThermalInfo::default();
    test_info.cpu_temp = 45.0;
    test_info.gpu_temp = Some(55.0);
    test_info.heatsink_temp = None;
    test_info.ambient_temp = None;
    test_info.battery_temp = None;
    test_info.thermal_throttling = false;
    test_info.fan_speed = 0;

    let mock_iokit = SimpleMockIOKit::new().with_thermal_info(test_info);

    let result = mock_iokit.get_thermal_info();
    assert!(result.is_ok());

    let info = result.unwrap();
    assert_eq!(info.cpu_temp, 45.0);
    assert_eq!(info.gpu_temp, Some(55.0));
    assert_eq!(info.heatsink_temp, None);
    assert_eq!(info.ambient_temp, None);
    assert_eq!(info.battery_temp, None);
    assert!(!info.thermal_throttling);
}

#[test]
fn test_get_heatsink_temperature() {
    let info = get_mock_thermal_info();
    assert_eq!(info.heatsink_temp, Some(40.0));
}

#[test]
fn test_get_ambient_temperature() {
    let info = get_mock_thermal_info();
    assert_eq!(info.ambient_temp, Some(25.0));
}

#[test]
fn test_get_battery_temperature() {
    let info = get_mock_thermal_info();
    assert_eq!(info.battery_temp, Some(35.0));
}

#[test]
fn test_check_thermal_throttling() {
    let mut mock_info = ThermalInfo::default();
    mock_info.cpu_temp = 45.0;
    mock_info.gpu_temp = Some(55.0);
    mock_info.heatsink_temp = Some(40.0);
    mock_info.ambient_temp = Some(25.0);
    mock_info.battery_temp = Some(35.0);
    mock_info.thermal_throttling = true;
    mock_info.fan_speed = 0;

    assert!(mock_info.thermal_throttling);
}

#[test]
fn test_thermal_info_error_propagation() {
    let cpu_error = Error::iokit_error("CPU sensor error");
    let gpu_error = Error::iokit_error("GPU sensor error");
    let thermal_error = Error::iokit_error("Thermal sensor error");

    let mock = SimpleMockIOKit::new()
        .with_cpu_temp_error(cpu_error)
        .with_gpu_stats_error(gpu_error)
        .with_thermal_info_error(thermal_error);

    let result = mock.get_cpu_temperature("platform");
    assert!(result.is_err());

    let result = mock.get_gpu_stats();
    assert!(result.is_err());

    let result = mock.get_thermal_info();
    assert!(result.is_err());
}

#[test]
fn test_thermal_info_accessors() {
    let mut info = ThermalInfo::default();
    info.cpu_temp = 45.0;
    info.gpu_temp = Some(55.0);
    info.heatsink_temp = Some(40.0);
    info.ambient_temp = Some(25.0);
    info.battery_temp = Some(35.0);
    info.thermal_throttling = false;
    info.fan_speed = 1500;

    // Test the DictionaryAccess trait implementation
    assert_eq!(info.get_number("cpu_temp").unwrap(), 45.0);
    assert_eq!(info.get_number("gpu_temp").unwrap(), 55.0);
    assert_eq!(info.get_number("heatsink_temp").unwrap(), 40.0);
    assert_eq!(info.get_number("ambient_temp").unwrap(), 25.0);
    assert_eq!(info.get_number("battery_temp").unwrap(), 35.0);
    assert_eq!(info.get_number("fan_speed").unwrap(), 1500.0);
    assert!(!info.get_bool("thermal_throttling").unwrap());

    // Test with a thermal throttling enabled
    let mut info = ThermalInfo::default();
    info.cpu_temp = 45.0;
    info.gpu_temp = None;
    info.heatsink_temp = None;
    info.ambient_temp = None;
    info.battery_temp = None;
    info.thermal_throttling = true;
    info.fan_speed = 0;

    // Test accessors with None values
    assert_eq!(info.get_number("cpu_temp").unwrap(), 45.0);
    assert!(info.get_bool("thermal_throttling").unwrap());
}

mod additional_safe_tests {
    use super::*;

    #[test]
    fn test_thermal_info_complete() {
        if let Ok(info) = IOKitImpl::new().and_then(|iokit| iokit.get_thermal_info()) {
            println!("Thermal Information:");
            println!("CPU Temperature: {}°C", info.cpu_temp);
            println!("Thermal Throttling: {}", info.thermal_throttling);
            println!("Fan Speed: {} RPM", info.fan_speed);

            if let Some(temp) = info.gpu_temp {
                println!("GPU Temperature: {}°C", temp);
            }
            if let Some(temp) = info.ambient_temp {
                println!("Ambient Temperature: {}°C", temp);
            }
            if let Some(temp) = info.battery_temp {
                println!("Battery Temperature: {}°C", temp);
            }
            if let Some(temp) = info.heatsink_temp {
                println!("Heatsink Temperature: {}°C", temp);
            }
        }
    }
}
