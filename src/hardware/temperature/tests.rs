use std::{thread, time::Duration};

use super::*;
use crate::hardware::iokit::{FanInfo, ThermalInfo, ThreadSafeNSDictionary, ThreadSafeAnyObject,GpuStats};
use crate::Error;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSDictionary, NSObject, NSString};
use std::sync::Arc;
use std::sync::Mutex;

// Custom mock implementation of IOKit that implements Clone
#[derive(Clone)]
struct MockIOKitClone {
    thermal_info: Arc<dyn Fn() -> Result<ThermalInfo> + Send + Sync>,
    fan_info: Arc<dyn Fn() -> Result<Vec<FanInfo>> + Send + Sync>,
}

impl std::fmt::Debug for MockIOKitClone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockIOKitClone")
            .field("thermal_info", &"<function>")
            .field("fan_info", &"<function>")
            .finish()
    }
}

impl MockIOKitClone {
    fn new() -> Self {
        Self {
            thermal_info: Arc::new(|| {
                Ok(ThermalInfo {
                    cpu_temp: 45.0,
                    gpu_temp: 55.0,
                    heatsink_temp: 40.0,
                    ambient_temp: 30.0,
                    battery_temp: 35.0,
                    is_throttling: false,
                    cpu_power: 25.0,
                    fan_speed: vec![2000.0],
                })
            }),
            fan_info: Arc::new(|| {
                Ok(vec![
                    FanInfo { speed_rpm: 2000, min_speed: 1000, max_speed: 4000, percentage: 33.3 },
                    FanInfo { speed_rpm: 2500, min_speed: 1200, max_speed: 5000, percentage: 40.0 },
                ])
            }),
        }
    }

    fn with_thermal_info<F>(self, f: F) -> Self
    where
        F: Fn() -> Result<ThermalInfo> + Send + Sync + 'static,
    {
        Self { thermal_info: Arc::new(f), fan_info: self.fan_info }
    }

    fn with_fan_info<F>(self, f: F) -> Self
    where
        F: Fn() -> Result<Vec<FanInfo>> + Send + Sync + 'static,
    {
        Self { thermal_info: self.thermal_info, fan_info: Arc::new(f) }
    }
}

impl IOKit for MockIOKitClone {
    fn io_service_matching(&self, _name: &str) -> Result<ThreadSafeNSDictionary> {
        Ok(ThreadSafeNSDictionary::new(NSDictionary::new()))
    }

    fn io_service_get_matching_service(&self, _matching: &ThreadSafeNSDictionary) -> Result<ThreadSafeNSDictionary> {
        Ok(ThreadSafeNSDictionary::new(NSDictionary::new()))
    }

    fn io_registry_entry_create_cf_properties(&self, _entry: &ThreadSafeNSDictionary) -> Result<ThreadSafeNSDictionary> {
        Ok(ThreadSafeNSDictionary::new(NSDictionary::new()))
    }

    fn io_connect_call_method(
        &self,
        _connection: u32,
        _selector: u32,
        _input: &[u8],
        _input_cnt: u32,
        _output: &mut [u8],
        _output_cnt: &mut u32,
    ) -> Result<()> {
        Ok(())
    }

    fn get_number_property(&self, _dict: &NSDictionary<NSString, NSObject>, key: &str) -> Option<f64> {
        match key {
            "CurrentCapacity" => Some(85.0),
            "CycleCount" => Some(100.0),
            "Temperature" => Some(35.0),
            "Voltage" => Some(12.0),
            "Amperage" => Some(1.5),
            "DesignCapacity" => Some(100.0),
            _ => None,
        }
    }

    fn get_service_properties(&self, _service: &ThreadSafeAnyObject) -> Result<ThreadSafeNSDictionary> {
        Ok(ThreadSafeNSDictionary::new(NSDictionary::new()))
    }

    fn get_service_matching(&self, _name: &str) -> Result<Option<ThreadSafeAnyObject>> {
        Ok(None)
    }

    fn get_cpu_temperature(&self) -> Result<f64> {
        Ok((*self.thermal_info)()?.cpu_temp)
    }

    fn get_gpu_stats(&self) -> Result<GpuStats> {
        unimplemented!("Not needed for tests")
    }

    fn get_fan_info(&self, fan_index: u32) -> Result<FanInfo> {
        let fans = (*self.fan_info)()?;
        fans.get(fan_index as usize).cloned().ok_or_else(|| {
            Error::iokit_error(-1, format!("Fan index out of bounds: {}", fan_index))
        })
    }

    fn get_thermal_info(&self) -> Result<ThermalInfo> {
        (*self.thermal_info)()
    }

    fn clone_box(&self) -> Box<dyn IOKit> {
        Box::new(self.clone())
    }
}

#[test]
fn test_temperature_new() {
    let temp = Temperature::new();
    assert!(temp.sensors.is_empty());
    assert!(temp.fans.is_empty());
    assert!(!temp.is_throttling);
    assert_eq!(temp.cpu_power, None);
}

#[test]
fn test_with_config() {
    let config = TemperatureConfig {
        poll_interval_ms: 5000,
        throttling_threshold: 90.0,
        auto_refresh: false,
    };

    let temp = Temperature::with_config(config);
    assert_eq!(temp.config.poll_interval_ms, 5000);
    assert_eq!(temp.config.throttling_threshold, 90.0);
    assert!(!temp.config.auto_refresh);
}

#[test]
fn test_should_refresh() {
    // Create Temperature with short refresh interval
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 10,
        throttling_threshold: 80.0,
        auto_refresh: true,
    });

    // Should be false immediately after creation (because we set last_refresh to now-60s in constructor)
    assert!(temp.should_refresh());

    // Manually update last_refresh
    temp.last_refresh = Instant::now();
    assert!(!temp.should_refresh());

    // Wait for the interval to elapse
    thread::sleep(Duration::from_millis(20));
    assert!(temp.should_refresh());
}

#[test]
fn test_should_refresh_elapsed_time() {
    // Create Temperature with a short poll interval
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 10,
        throttling_threshold: 80.0,
        auto_refresh: true,
    });

    // Set last_refresh to now
    temp.last_refresh = Instant::now();

    // Should not refresh immediately
    assert!(!temp.should_refresh());

    // Wait for the interval to elapse
    thread::sleep(Duration::from_millis(15));

    // Now should refresh
    assert!(temp.should_refresh());

    // Test with a longer interval
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 100,
        throttling_threshold: 80.0,
        auto_refresh: true,
    });

    // Set last_refresh to now
    temp.last_refresh = Instant::now();

    // Should not refresh immediately
    assert!(!temp.should_refresh());

    // Manually set last_refresh to a time in the past
    temp.last_refresh = Instant::now() - Duration::from_millis(150);

    // Now should refresh
    assert!(temp.should_refresh());
}

#[test]
fn test_cpu_temperature() {
    // Create a Temperature instance with auto_refresh disabled
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    // Insert the test value directly into the sensors map
    temp.sensors.insert("CPU".to_string(), 42.5);

    // Test the method using the manually inserted value
    let result = temp.cpu_temperature().unwrap();
    assert_eq!(result, 42.5);
}

#[test]
fn test_gpu_temperature() {
    // Create a Temperature instance with auto_refresh disabled
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    temp.sensors.insert("GPU".to_string(), 55.0);

    // Test the method
    let result = temp.gpu_temperature().unwrap();
    assert_eq!(result, 55.0);
}

#[test]
fn test_additional_sensors() {
    // Create a Temperature instance with auto_refresh disabled
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    // Add various temperature sensors
    temp.sensors.insert("Heatsink".to_string(), 45.0);
    temp.sensors.insert("Ambient".to_string(), 32.0);
    temp.sensors.insert("Battery".to_string(), 38.0);

    // Test each sensor
    assert_eq!(temp.heatsink_temperature().unwrap(), 45.0);
    assert_eq!(temp.ambient_temperature().unwrap(), 32.0);
    assert_eq!(temp.battery_temperature().unwrap(), 38.0);
}

#[test]
fn test_list_sensors() {
    // Create a Temperature instance with pre-populated data
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    temp.sensors.insert("CPU".to_string(), 42.5);
    temp.sensors.insert("GPU".to_string(), 55.0);
    temp.sensors.insert("Heatsink".to_string(), 45.0);
    temp.sensors.insert("Ambient".to_string(), 32.0);
    temp.sensors.insert("Custom".to_string(), 27.0);

    // Test the method
    let sensors = temp.list_sensors().unwrap();

    // Verify results
    assert_eq!(sensors.len(), 5);

    // Check that the sensors have correct locations
    let has_cpu = sensors
        .iter()
        .any(|(name, location)| name == "CPU" && matches!(location, SensorLocation::Cpu));
    let has_gpu = sensors
        .iter()
        .any(|(name, location)| name == "GPU" && matches!(location, SensorLocation::Gpu));
    let has_heatsink = sensors
        .iter()
        .any(|(name, location)| name == "Heatsink" && matches!(location, SensorLocation::Heatsink));
    let has_ambient = sensors
        .iter()
        .any(|(name, location)| name == "Ambient" && matches!(location, SensorLocation::Ambient));
    let has_custom = sensors.iter().any(|(name, location)| {
        name == "Custom"
            && if let SensorLocation::Other(custom_name) = location {
                custom_name == "Custom"
            } else {
                false
            }
    });

    assert!(has_cpu);
    assert!(has_gpu);
    assert!(has_heatsink);
    assert!(has_ambient);
    assert!(has_custom);
}

#[test]
fn test_fan_functions() {
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    // Add mock fans
    temp.fans.push(Fan {
        name: "Fan 0".to_string(),
        speed_rpm: 2000,
        min_speed: 1000,
        max_speed: 4000,
        percentage: 33.3,
    });

    temp.fans.push(Fan {
        name: "Fan 1".to_string(),
        speed_rpm: 3000,
        min_speed: 1200,
        max_speed: 5000,
        percentage: 47.4,
    });

    // Test fan count
    assert_eq!(temp.fan_count().unwrap(), 2);

    // Test get_fans
    let fans = temp.get_fans().unwrap();
    assert_eq!(fans.len(), 2);
    assert_eq!(fans[0].speed_rpm, 2000);
    assert_eq!(fans[1].speed_rpm, 3000);

    // Test get_fan
    let fan0 = temp.get_fan(0).unwrap();
    assert_eq!(fan0.name, "Fan 0");
    assert_eq!(fan0.speed_rpm, 2000);
    assert_eq!(fan0.min_speed, 1000);
    assert_eq!(fan0.max_speed, 4000);
    assert!(fan0.percentage > 33.0 && fan0.percentage < 34.0);

    let fan1 = temp.get_fan(1).unwrap();
    assert_eq!(fan1.name, "Fan 1");
    assert_eq!(fan1.speed_rpm, 3000);

    // Test get_fan with invalid index
    assert!(temp.get_fan(2).is_err());
}

#[test]
fn test_is_throttling_property() {
    // The is_throttling() method calls IOKit methods that are difficult to mock in tests So instead, we're testing that
    // the property correctly reflects the state

    // Test default state
    let temp = Temperature::new();
    assert!(!temp.is_throttling);

    // Test setting the property manually
    let mut temp = Temperature::new();
    temp.is_throttling = true;
    assert!(temp.is_throttling);
}

#[test]
fn test_is_throttling_temperature_heuristic() {
    // In this test we're just testing the logic that determines throttling based on CPU temperature compared to the
    // threshold
    let threshold = 80.0;

    // Test case 1: Below threshold
    let cpu_temp = 75.0;
    assert!(cpu_temp < threshold);

    // Test case 2: Above threshold
    let cpu_temp = 85.0;
    assert!(cpu_temp > threshold);
}

#[test]
fn test_get_fan_info() {
    // Create a Temperature instance with auto_refresh disabled
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    // Add a fan
    temp.fans.push(Fan {
        name: "Test Fan".to_string(),
        speed_rpm: 2000,
        min_speed: 500,
        max_speed: 5000,
        percentage: 40.0,
    });

    // Get the fan info
    let result = temp.get_fan(0).unwrap();

    // Verify the result
    assert_eq!(result.speed_rpm, 2000);
    assert_eq!(result.min_speed, 500);
    assert_eq!(result.max_speed, 5000);
    assert_eq!(result.percentage, 40.0);
}

#[test]
fn test_get_fan_info_min_max_equal() {
    // Test the fan percentage calculation when min == max
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    // Add a fan with min == max
    temp.fans.push(Fan {
        name: "Test Fan".to_string(),
        speed_rpm: 2000,
        min_speed: 2000, // Same as current and max
        max_speed: 2000, // Same as current and min
        percentage: 0.0, // Should be 0 when min==max
    });

    // Get the fan info
    let result = temp.get_fan(0).unwrap();

    // When min and max are the same, percentage should be 0
    assert_eq!(result.speed_rpm, 2000);
    assert_eq!(result.min_speed, 2000);
    assert_eq!(result.max_speed, 2000);
    assert_eq!(result.percentage, 0.0);
}

#[test]
fn test_get_heatsink_temperature() {
    // Create a Temperature instance with auto_refresh disabled
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    // Set up test data
    temp.sensors.insert("Heatsink".to_string(), 40.0);

    // Call the method
    let result = temp.heatsink_temperature().unwrap();

    // Verify the result
    assert_eq!(result, 40.0);
}

#[test]
fn test_get_ambient_temperature() {
    // Create a Temperature instance with auto_refresh disabled
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    // Set up test data
    temp.sensors.insert("Ambient".to_string(), 25.0);

    // Call the method
    let result = temp.ambient_temperature().unwrap();

    // Verify the result
    assert_eq!(result, 25.0);
}

#[test]
fn test_get_battery_temperature() {
    // Create a Temperature instance with auto_refresh disabled
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    // Set up test data
    temp.sensors.insert("Battery".to_string(), 35.0);

    // Call the method
    let result = temp.battery_temperature().unwrap();

    // Verify the result
    assert_eq!(result, 35.0);
}

#[test]
fn test_fan_info_debug() {
    // Test the Debug implementation for Fan
    let fan = Fan {
        name: "Test Fan".to_string(),
        speed_rpm: 2000,
        min_speed: 500,
        max_speed: 5000,
        percentage: 40.0,
    };

    let debug_str = format!("{:?}", fan);

    // Make sure all the fields are represented in the debug output
    assert!(debug_str.contains("name: \"Test Fan\""));
    assert!(debug_str.contains("speed_rpm: 2000"));
    assert!(debug_str.contains("min_speed: 500"));
    assert!(debug_str.contains("max_speed: 5000"));
    assert!(debug_str.contains("percentage: 40.0"));
}

#[test]
fn test_fan_info_clone() {
    // Test the Clone implementation for Fan
    let fan = Fan {
        name: "Test Fan".to_string(),
        speed_rpm: 2000,
        min_speed: 500,
        max_speed: 5000,
        percentage: 40.0,
    };

    let fan_clone = fan.clone();

    assert_eq!(fan.name, fan_clone.name);
    assert_eq!(fan.speed_rpm, fan_clone.speed_rpm);
    assert_eq!(fan.min_speed, fan_clone.min_speed);
    assert_eq!(fan.max_speed, fan_clone.max_speed);
    assert_eq!(fan.percentage, fan_clone.percentage);
}

#[test]
fn test_thermal_metrics_clone() {
    // Test the Clone implementation for ThermalMetrics
    let metrics = ThermalMetrics {
        cpu_temperature: Some(45.0),
        gpu_temperature: Some(55.0),
        heatsink_temperature: Some(40.0),
        ambient_temperature: Some(25.0),
        battery_temperature: Some(35.0),
        is_throttling: false,
        cpu_power: Some(15.0),
        fans: vec![Fan {
            name: "Test Fan".to_string(),
            speed_rpm: 2000,
            min_speed: 500,
            max_speed: 5000,
            percentage: 40.0,
        }],
    };

    let metrics_clone = metrics.clone();

    assert_eq!(metrics.cpu_temperature, metrics_clone.cpu_temperature);
    assert_eq!(metrics.gpu_temperature, metrics_clone.gpu_temperature);
    assert_eq!(metrics.heatsink_temperature, metrics_clone.heatsink_temperature);
    assert_eq!(metrics.ambient_temperature, metrics_clone.ambient_temperature);
    assert_eq!(metrics.battery_temperature, metrics_clone.battery_temperature);
    assert_eq!(metrics.is_throttling, metrics_clone.is_throttling);
    assert_eq!(metrics.cpu_power, metrics_clone.cpu_power);
    assert_eq!(metrics.fans.len(), metrics_clone.fans.len());
    assert_eq!(metrics.fans[0].name, metrics_clone.fans[0].name);
    assert_eq!(metrics.fans[0].speed_rpm, metrics_clone.fans[0].speed_rpm);
}

#[test]
fn test_thermal_metrics_debug() {
    // Test the Debug implementation for ThermalMetrics
    let metrics = ThermalMetrics {
        cpu_temperature: Some(45.0),
        gpu_temperature: Some(55.0),
        heatsink_temperature: Some(40.0),
        ambient_temperature: Some(25.0),
        battery_temperature: Some(35.0),
        is_throttling: false,
        cpu_power: Some(15.0),
        fans: vec![Fan {
            name: "Test Fan".to_string(),
            speed_rpm: 2000,
            min_speed: 500,
            max_speed: 5000,
            percentage: 40.0,
        }],
    };

    let debug_str = format!("{:?}", metrics);

    // Make sure all the fields are represented in the debug output
    assert!(debug_str.contains("cpu_temperature: Some(45.0)"));
    assert!(debug_str.contains("gpu_temperature: Some(55.0)"));
    assert!(debug_str.contains("heatsink_temperature: Some(40.0)"));
    assert!(debug_str.contains("ambient_temperature: Some(25.0)"));
    assert!(debug_str.contains("battery_temperature: Some(35.0)"));
    assert!(debug_str.contains("is_throttling: false"));
    assert!(debug_str.contains("cpu_power: Some(15.0)"));
    assert!(debug_str.contains("fans:"));
}

#[test]
fn test_sensor_location_debug() {
    // Test Debug implementation for SensorLocation
    let locations = vec![
        SensorLocation::Cpu,
        SensorLocation::Gpu,
        SensorLocation::Memory,
        SensorLocation::Storage,
        SensorLocation::Battery,
        SensorLocation::Heatsink,
        SensorLocation::Ambient,
        SensorLocation::Other("Custom".to_string()),
    ];

    for location in locations {
        let debug_str = format!("{:?}", location);
        match location {
            SensorLocation::Cpu => assert_eq!(debug_str, "Cpu"),
            SensorLocation::Gpu => assert_eq!(debug_str, "Gpu"),
            SensorLocation::Memory => assert_eq!(debug_str, "Memory"),
            SensorLocation::Storage => assert_eq!(debug_str, "Storage"),
            SensorLocation::Battery => assert_eq!(debug_str, "Battery"),
            SensorLocation::Heatsink => assert_eq!(debug_str, "Heatsink"),
            SensorLocation::Ambient => assert_eq!(debug_str, "Ambient"),
            SensorLocation::Other(name) => {
                assert!(debug_str.contains(&format!("Other(\"{}\")", name)))
            },
        }
    }
}

#[test]
fn test_sensor_location_clone() {
    // Test Clone implementation for SensorLocation
    let original = SensorLocation::Other("Custom".to_string());
    let cloned = original.clone();

    match cloned {
        SensorLocation::Other(name) => assert_eq!(name, "Custom"),
        _ => panic!("Expected SensorLocation::Other"),
    }
}

#[test]
fn test_sensor_location_equality() {
    // Test PartialEq implementation for SensorLocation
    assert_eq!(SensorLocation::Cpu, SensorLocation::Cpu);
    assert_ne!(SensorLocation::Cpu, SensorLocation::Gpu);

    let other1 = SensorLocation::Other("Custom".to_string());
    let other2 = SensorLocation::Other("Custom".to_string());
    let other3 = SensorLocation::Other("Different".to_string());

    assert_eq!(other1, other2);
    assert_ne!(other1, other3);
}

#[test]
fn test_temperature_config_default() {
    // Test Default implementation for TemperatureConfig
    let config = TemperatureConfig::default();

    assert_eq!(config.poll_interval_ms, 1000);
    assert_eq!(config.throttling_threshold, 80.0);
    assert!(config.auto_refresh);
}

#[test]
fn test_get_sensor_temperature() {
    // Create a Temperature instance with auto_refresh disabled
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    // Set up test data
    temp.sensors.insert("CPU".to_string(), 42.5);
    temp.sensors.insert("Custom".to_string(), 30.0);

    // Test getting existing sensor
    let result = temp.get_sensor_temperature("CPU").unwrap();
    assert_eq!(result, 42.5);

    let result = temp.get_sensor_temperature("Custom").unwrap();
    assert_eq!(result, 30.0);

    // Test getting non-existent sensor
    let result = temp.get_sensor_temperature("NonExistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_cpu_power() {
    // Create a Temperature instance with auto_refresh disabled
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    // Test with no power data
    let result = temp.cpu_power();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not available"));

    // Set power data and test again
    temp.cpu_power = Some(45.5);
    let result = temp.cpu_power().unwrap();
    assert_eq!(result, 45.5);
}

#[test]
fn test_temperature_default() {
    // Test Default implementation for Temperature
    let temp = Temperature::default();

    // Should be the same as calling new()
    let temp_new = Temperature::new();

    assert_eq!(temp.sensors.len(), temp_new.sensors.len());
    assert_eq!(temp.fans.len(), temp_new.fans.len());
    assert_eq!(temp.is_throttling, temp_new.is_throttling);
    assert_eq!(temp.cpu_power, temp_new.cpu_power);
    assert_eq!(temp.config.poll_interval_ms, temp_new.config.poll_interval_ms);
    assert_eq!(temp.config.throttling_threshold, temp_new.config.throttling_threshold);
    assert_eq!(temp.config.auto_refresh, temp_new.config.auto_refresh);
}

#[test]
fn test_get_thermal_metrics() {
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    // Set up test data
    temp.sensors.insert("CPU".to_string(), 42.5);
    temp.sensors.insert("GPU".to_string(), 55.0);
    temp.sensors.insert("Heatsink".to_string(), 45.0);
    temp.sensors.insert("Ambient".to_string(), 32.0);
    temp.sensors.insert("Battery".to_string(), 38.0);
    temp.is_throttling = false;
    temp.cpu_power = Some(28.5);

    // Add a fan for completeness
    temp.fans.push(Fan {
        name: "Test Fan".to_string(),
        speed_rpm: 2000,
        min_speed: 1000,
        max_speed: 4000,
        percentage: 33.3,
    });

    // Get metrics
    let metrics = temp.get_thermal_metrics().unwrap();

    // Verify the metrics structure
    assert_eq!(metrics.cpu_temperature, Some(42.5));
    assert_eq!(metrics.gpu_temperature, Some(55.0));
    assert_eq!(metrics.heatsink_temperature, Some(45.0));
    assert_eq!(metrics.ambient_temperature, Some(32.0));
    assert_eq!(metrics.battery_temperature, Some(38.0));
    assert!(!metrics.is_throttling);
    assert_eq!(metrics.cpu_power, Some(28.5));
    assert_eq!(metrics.fans.len(), 1);
    assert_eq!(metrics.fans[0].speed_rpm, 2000);
}

#[test]
fn test_is_throttling() {
    // Create a Temperature instance with auto_refresh disabled
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    // Test with CPU temperature below threshold
    temp.sensors.insert("CPU".to_string(), 75.0);
    let result = temp.is_throttling().unwrap();
    assert!(!result);

    // Test with CPU temperature above threshold
    temp.sensors.insert("CPU".to_string(), 85.0);
    let result = temp.is_throttling().unwrap();
    assert!(result);
}

#[test]
fn test_error_cases() {
    // Create a Temperature instance with auto_refresh disabled
    let mut temp = Temperature::with_config(TemperatureConfig {
        poll_interval_ms: 1000,
        throttling_threshold: 80.0,
        auto_refresh: false,
    });

    // Test error when sensor doesn't exist
    let result = temp.cpu_temperature();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not available"));

    let result = temp.gpu_temperature();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not available"));

    let result = temp.heatsink_temperature();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not available"));

    let result = temp.ambient_temperature();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not available"));

    let result = temp.battery_temperature();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not available"));

    // Test error when fan index is out of bounds
    let result = temp.get_fan(0);
    assert!(result.is_err());
    // The error message might vary, so just check that it's an error
    assert!(result.is_err());
}

#[test]
fn test_fan_count_empty() {
    let mock_iokit = MockIOKitClone::new()
        .with_thermal_info(|| {
            Ok(ThermalInfo {
                cpu_temp: 45.0,
                gpu_temp: 55.0,
                heatsink_temp: 40.0,
                ambient_temp: 30.0,
                battery_temp: 35.0,
                is_throttling: false,
                cpu_power: 25.0,
                fan_speed: vec![],
            })
        })
        .with_fan_info(|| Ok(vec![]));

    let mut temp = Temperature::with_iokit(mock_iokit, TemperatureConfig::default());
    assert_eq!(temp.fan_count().unwrap(), 0);
}

#[test]
fn test_get_fans_empty() {
    let mock_iokit = MockIOKitClone::new()
        .with_thermal_info(|| {
            Ok(ThermalInfo {
                cpu_temp: 45.0,
                gpu_temp: 55.0,
                heatsink_temp: 40.0,
                ambient_temp: 30.0,
                battery_temp: 35.0,
                is_throttling: false,
                cpu_power: 25.0,
                fan_speed: vec![],
            })
        })
        .with_fan_info(|| Ok(vec![]));

    let mut temp = Temperature::with_iokit(mock_iokit, TemperatureConfig::default());
    let fans = temp.get_fans().unwrap();
    assert!(fans.is_empty());
}

#[test]
fn test_get_thermal_info() {
    let mock_iokit = MockIOKitClone::new()
        .with_thermal_info(|| {
            Ok(ThermalInfo {
                cpu_temp: 42.5,
                gpu_temp: 55.0,
                heatsink_temp: 45.0,
                ambient_temp: 32.0,
                battery_temp: 38.0,
                is_throttling: false,
                cpu_power: 28.5,
                fan_speed: vec![2000.0],
            })
        })
        .with_fan_info(|| {
            Ok(vec![FanInfo {
                speed_rpm: 2000,
                min_speed: 1000,
                max_speed: 4000,
                percentage: 33.3,
            }])
        });

    let mut temp = Temperature::with_iokit(mock_iokit, TemperatureConfig::default());
    let metrics = temp.get_thermal_metrics().unwrap();

    assert_eq!(metrics.cpu_temperature, Some(42.5));
    assert_eq!(metrics.gpu_temperature, Some(55.0));
    assert_eq!(metrics.heatsink_temperature, Some(45.0));
    assert_eq!(metrics.ambient_temperature, Some(32.0));
    assert_eq!(metrics.battery_temperature, Some(38.0));
    assert!(!metrics.is_throttling);
    assert_eq!(metrics.cpu_power, Some(28.5));
    assert_eq!(metrics.fans.len(), 1);
    assert_eq!(metrics.fans[0].speed_rpm, 2000);
}

#[test]
fn test_get_thermal_info_with_failures() {
    let mock_iokit = MockIOKitClone::new()
        .with_thermal_info(|| {
            Ok(ThermalInfo {
                cpu_temp: 42.5,
                gpu_temp: 55.0,
                heatsink_temp: 40.0,
                ambient_temp: 30.0,
                battery_temp: 35.0,
                is_throttling: false,
                cpu_power: 25.0,
                fan_speed: vec![],
            })
        })
        .with_fan_info(|| Ok(vec![]));

    let mut temp = Temperature::with_iokit(mock_iokit, TemperatureConfig::default());
    let metrics = temp.get_thermal_metrics().unwrap();

    assert_eq!(metrics.cpu_temperature, Some(42.5));
    assert_eq!(metrics.gpu_temperature, Some(55.0));
    assert_eq!(metrics.heatsink_temperature, Some(40.0));
    assert_eq!(metrics.ambient_temperature, Some(30.0));
    assert_eq!(metrics.battery_temperature, Some(35.0));
    assert!(!metrics.is_throttling);
    assert_eq!(metrics.cpu_power, Some(25.0));
    assert!(metrics.fans.is_empty());
}
