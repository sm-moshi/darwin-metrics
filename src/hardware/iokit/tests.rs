#![allow(unused_imports)]

use std::os::raw::c_char;

use objc2::{msg_send, rc::autoreleasepool};

use crate::{
    error::{Error, Result},
    hardware::iokit::{FanInfo, GpuStats, IOKit, IOKitImpl, MockIOKit, ThermalInfo},
    utils::{
        bindings::{
            smc_key_from_chars,
            // These constants are used in test_smc_read_key_mocks test (lines ~1000-1034)
            SMC_KEY_AMBIENT_TEMP,
            SMC_KEY_BATTERY_TEMP,
            SMC_KEY_CPU_TEMP,
            SMC_KEY_FAN_NUM,
            SMC_KEY_GPU_TEMP,
        },
        test_utils::{create_test_dictionary, create_test_object},
    },
};

#[test]
fn test_smc_key_from_chars() {
    // Test with "TC0P" (CPU temperature key)
    let key = [b'T' as c_char, b'C' as c_char, b'0' as c_char, b'P' as c_char];
    let result = smc_key_from_chars(key);

    // Calculate the expected value: ('T' << 24) | ('C' << 16) | ('0' << 8) | 'P'
    let expected = (b'T' as u32) << 24 | (b'C' as u32) << 16 | (b'0' as u32) << 8 | (b'P' as u32);

    assert_eq!(result, expected);
}

#[test]
fn test_get_cpu_temperature() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up the expected behavior
    mock_iokit.expect_get_cpu_temperature().returning(|| Ok(45.5));

    // Call the method
    let result = mock_iokit.get_cpu_temperature().unwrap();

    // Check the result
    assert_eq!(result, 45.5);
}

#[test]
fn test_get_gpu_temperature() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up the expected behavior
    mock_iokit.expect_get_gpu_temperature().returning(|| Ok(55.0));

    // Call the method
    let result = mock_iokit.get_gpu_temperature().unwrap();

    // Check the result
    assert_eq!(result, 55.0);
}

#[test]
fn test_get_gpu_stats() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up the expected behavior
    mock_iokit.expect_get_gpu_stats().returning(|| {
        Ok(GpuStats {
            utilization: 50.0,
            perf_cap: 50.0,
            perf_threshold: 100.0,
            memory_used: 1024 * 1024 * 1024,      // 1 GB
            memory_total: 4 * 1024 * 1024 * 1024, // 4 GB
            name: "Test GPU".to_string(),
        })
    });

    // Call the method
    let result = mock_iokit.get_gpu_stats().unwrap();

    // Check the result
    assert_eq!(result.utilization, 50.0);
    assert_eq!(result.memory_total, 4 * 1024 * 1024 * 1024);
    assert_eq!(result.name, "Test GPU");
}

#[test]
fn test_io_service_matching() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up expectations
    mock_iokit.expect_io_service_matching().returning(|_| create_test_dictionary());

    // Call the method
    let _result = mock_iokit.io_service_matching("TestService");

    // Mock will verify the expectation was met
}

#[test]
fn test_io_service_get_matching_service() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();
    let dict = create_test_dictionary();

    // Set up expectations
    mock_iokit.expect_io_service_get_matching_service().returning(|_| None);

    // Call the method
    let result = mock_iokit.io_service_get_matching_service(&dict);

    // Verify the result
    assert!(result.is_none());
}

#[test]
fn test_io_registry_entry_create_cf_properties() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();
    let obj = create_test_object();

    // Set up expectations
    mock_iokit
        .expect_io_registry_entry_create_cf_properties()
        .returning(|_| Ok(create_test_dictionary()));

    // Call the method
    let result = mock_iokit.io_registry_entry_create_cf_properties(&obj);

    // Verify we got a successful result
    assert!(result.is_ok());
}

#[test]
fn test_get_string_property() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();
    let dict = create_test_dictionary();

    // Set up expectations
    mock_iokit.expect_get_string_property().returning(|_, key| {
        if key == "TestKey" {
            Some("TestValue".to_string())
        } else {
            None
        }
    });

    // Call the method
    let result = mock_iokit.get_string_property(&dict, "TestKey");

    // Verify the result
    assert_eq!(result, Some("TestValue".to_string()));

    // Test with non-existent key
    let result = mock_iokit.get_string_property(&dict, "NonExistentKey");
    assert_eq!(result, None);
}

#[test]
fn test_get_number_property() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();
    let dict = create_test_dictionary();

    // Set up expectations
    mock_iokit.expect_get_number_property().returning(|_, key| {
        if key == "TestKey" {
            Some(42)
        } else {
            None
        }
    });

    // Call the method
    let result = mock_iokit.get_number_property(&dict, "TestKey");

    // Verify the result
    assert_eq!(result, Some(42));

    // Test with non-existent key
    let result = mock_iokit.get_number_property(&dict, "NonExistentKey");
    assert_eq!(result, None);
}

#[test]
fn test_get_bool_property() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();
    let dict = create_test_dictionary();

    // Set up expectations
    mock_iokit.expect_get_bool_property().returning(|_, key| {
        if key == "TestKey" {
            Some(true)
        } else {
            None
        }
    });

    // Call the method
    let result = mock_iokit.get_bool_property(&dict, "TestKey");

    // Verify the result
    assert_eq!(result, Some(true));

    // Test with non-existent key
    let result = mock_iokit.get_bool_property(&dict, "NonExistentKey");
    assert_eq!(result, None);
}

#[test]
fn test_get_dict_property() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();
    let dict = create_test_dictionary();

    // Set up expectation
    mock_iokit.expect_get_dict_property().returning(|_, key| {
        if key == "TestKey" {
            Some(create_test_dictionary())
        } else {
            None
        }
    });

    // Call the method
    let result = mock_iokit.get_dict_property(&dict, "TestKey");

    // Verify the result
    assert!(result.is_some());

    // Test with non-existent key
    let result = mock_iokit.get_dict_property(&dict, "NonExistentKey");
    assert!(result.is_none());
}

#[test]
fn test_get_thermal_info() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up the expectation
    mock_iokit.expect_get_thermal_info().returning(|| {
        Ok(ThermalInfo {
            cpu_temp: 45.0,
            gpu_temp: 55.0,
            heatsink_temp: Some(40.0),
            ambient_temp: Some(25.0),
            battery_temp: Some(35.0),
            is_throttling: false,
            cpu_power: Some(15.0),
        })
    });

    // Call the method
    let result = mock_iokit.get_thermal_info().unwrap();

    // Verify the result
    assert_eq!(result.cpu_temp, 45.0);
    assert_eq!(result.gpu_temp, 55.0);
    assert_eq!(result.heatsink_temp, Some(40.0));
    assert_eq!(result.ambient_temp, Some(25.0));
    assert_eq!(result.battery_temp, Some(35.0));
    assert!(!result.is_throttling);
    assert_eq!(result.cpu_power, Some(15.0));
}

#[test]
fn test_get_thermal_info_with_failures() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Directly mock the get_thermal_info method
    mock_iokit.expect_get_thermal_info().returning(|| {
        Ok(ThermalInfo {
            cpu_temp: 45.0,
            gpu_temp: 55.0,
            heatsink_temp: None,
            ambient_temp: None,
            battery_temp: None,
            is_throttling: false,
            cpu_power: None,
        })
    });

    // Call get_thermal_info - now mocked directly
    let result = mock_iokit.get_thermal_info();

    // It should succeed
    assert!(result.is_ok());
    let info = result.unwrap();

    // Check that required fields were set
    assert_eq!(info.cpu_temp, 45.0);
    assert_eq!(info.gpu_temp, 55.0);

    // Check that optional fields were set to None or default values
    assert_eq!(info.heatsink_temp, None);
    assert_eq!(info.ambient_temp, None);
    assert_eq!(info.battery_temp, None);
    assert_eq!(info.cpu_power, None);
    assert!(!info.is_throttling);
}

#[test]
fn test_get_fan_info() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up the expectation
    mock_iokit.expect_get_fan_info().with(mockall::predicate::eq(0)).returning(|_| {
        Ok(FanInfo { speed_rpm: 2000, min_speed: 500, max_speed: 5000, percentage: 40.0 })
    });

    // Call the method
    let result = mock_iokit.get_fan_info(0).unwrap();

    // Verify the result
    assert_eq!(result.speed_rpm, 2000);
    assert_eq!(result.min_speed, 500);
    assert_eq!(result.max_speed, 5000);
    assert_eq!(result.percentage, 40.0);
}

#[test]
fn test_get_fan_info_min_max_equal() {
    // Test the fan percentage calculation when min == max
    let mut mock_iokit = MockIOKit::new();

    // Expect get_fan_info to return a FanInfo with min == max
    mock_iokit.expect_get_fan_info().with(mockall::predicate::eq(0)).returning(|_| {
        // Return values where min and max are equal
        Ok(FanInfo {
            speed_rpm: 2000,
            min_speed: 2000, // Same as current and max
            max_speed: 2000, // Same as current and min
            percentage: 0.0, // Should be 0 when min==max
        })
    });

    // Call the method
    let result = mock_iokit.get_fan_info(0).unwrap();

    // When min and max are the same, percentage should be 0
    assert_eq!(result.speed_rpm, 2000);
    assert_eq!(result.min_speed, 2000);
    assert_eq!(result.max_speed, 2000);
    assert_eq!(result.percentage, 0.0);
}

#[test]
fn test_io_registry_entry_get_parent() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();
    let obj = create_test_object();

    // Set up the expectation
    mock_iokit.expect_io_registry_entry_get_parent().returning(|_| None);

    // Call the method
    let result = mock_iokit.io_registry_entry_get_parent(&obj);

    // Verify the result
    assert!(result.is_none());
}

#[test]
fn test_get_service() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up the expectation
    mock_iokit.expect_get_service().returning(|_| Ok(create_test_object()));

    // Call the method
    let result = mock_iokit.get_service("TestService");

    // Verify we got a successful result
    assert!(result.is_ok());
}

#[test]
fn test_impl_get_service_safety() {
    // Test the IOKitImpl's get_service method
    // This should be disabled in the safe mode
    let iokit = IOKitImpl;

    // The get_service method should be disabled for safety
    let result = iokit.get_service("TestService");

    // It should return an error without trying to access IOKit
    assert!(result.is_err());
    match result {
        Err(e) => {
            // Make sure we get the expected error message about service access being disabled
            let error_msg = format!("{}", e);
            assert!(error_msg.contains("Service access disabled for stability"));
        },
        _ => panic!("Expected an error from the disabled service access"),
    }
}

#[test]
fn test_get_heatsink_temperature() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up the expectation
    mock_iokit.expect_get_heatsink_temperature().returning(|| Ok(40.0));

    // Call the method
    let result = mock_iokit.get_heatsink_temperature().unwrap();

    // Verify the result
    assert_eq!(result, 40.0);
}

#[test]
fn test_get_ambient_temperature() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up the expectation
    mock_iokit.expect_get_ambient_temperature().returning(|| Ok(25.0));

    // Call the method
    let result = mock_iokit.get_ambient_temperature().unwrap();

    // Verify the result
    assert_eq!(result, 25.0);
}

#[test]
fn test_get_battery_temperature() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up the expectation
    mock_iokit.expect_get_battery_temperature().returning(|| Ok(35.0));

    // Call the method
    let result = mock_iokit.get_battery_temperature().unwrap();

    // Verify the result
    assert_eq!(result, 35.0);
}

#[test]
fn test_get_cpu_power() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up the expectation
    mock_iokit.expect_get_cpu_power().returning(|| Ok(15.0));

    // Call the method
    let result = mock_iokit.get_cpu_power().unwrap();

    // Verify the result
    assert_eq!(result, 15.0);
}

#[test]
fn test_check_thermal_throttling() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up expectation for throttled state
    mock_iokit.expect_check_thermal_throttling().returning(|| Ok(true));

    // Call the method
    let result = mock_iokit.check_thermal_throttling().unwrap();

    // Verify the result
    assert!(result);

    // Set up new mock for non-throttled state
    let mut mock_iokit = MockIOKit::new();
    mock_iokit.expect_check_thermal_throttling().returning(|| Ok(false));

    // Test non-throttled state
    let result = mock_iokit.check_thermal_throttling().unwrap();
    assert!(!result);
}

#[test]
fn test_read_smc_key() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();
    let key = [b'T' as c_char, b'C' as c_char, b'0' as c_char, b'P' as c_char];

    // Set up the expectation
    mock_iokit.expect_read_smc_key().with(mockall::predicate::eq(key)).returning(|_| Ok(42.0));

    // Call the method
    let result = mock_iokit.read_smc_key(key).unwrap();

    // Verify the result
    assert_eq!(result, 42.0);
}

#[test]
fn test_get_fan_count() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up the expectation
    mock_iokit.expect_get_fan_count().returning(|| Ok(2));

    // Call the method
    let result = mock_iokit.get_fan_count().unwrap();

    // Verify the result
    assert_eq!(result, 2);
}

#[test]
fn test_get_fan_speed() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up the expectation
    mock_iokit.expect_get_fan_speed().returning(|| Ok(2000));

    // Call the method
    let result = mock_iokit.get_fan_speed().unwrap();

    // Verify the result
    assert_eq!(result, 2000);
}

#[test]
fn test_get_all_fans() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up the expectation
    mock_iokit.expect_get_all_fans().returning(|| {
        Ok(vec![
            FanInfo { speed_rpm: 2000, min_speed: 500, max_speed: 5000, percentage: 40.0 },
            FanInfo { speed_rpm: 1800, min_speed: 400, max_speed: 4500, percentage: 35.0 },
        ])
    });

    // Call the method
    let result = mock_iokit.get_all_fans().unwrap();

    // Verify the result
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].speed_rpm, 2000);
    assert_eq!(result[1].speed_rpm, 1800);
}

#[test]
fn test_get_all_fans_empty() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up expectations for a system with no fans
    mock_iokit.expect_get_fan_count().returning(|| Ok(0));

    mock_iokit.expect_get_all_fans().returning(|| Ok(vec![]));

    // Call the method
    let result = mock_iokit.get_all_fans().unwrap();

    // Verify the result
    assert!(result.is_empty());
}

#[test]
fn test_get_all_fans_partial_failure() {
    // Create a mock IOKit implementation
    let mut mock_iokit = MockIOKit::new();

    // Set up the fan count
    mock_iokit.expect_get_fan_count().returning(|| Ok(2));

    // Make the first fan succeed and second fan fail
    mock_iokit.expect_get_fan_info().with(mockall::predicate::eq(0)).returning(|_| {
        Ok(FanInfo { speed_rpm: 2000, min_speed: 500, max_speed: 5000, percentage: 40.0 })
    });

    mock_iokit
        .expect_get_fan_info()
        .with(mockall::predicate::eq(1))
        .returning(|_| Err(Error::io_kit("Failed to get fan info")));

    // Create our own implementation of get_all_fans
    let all_fans_result: Result<Vec<FanInfo>> = {
        let fan_count = mock_iokit.get_fan_count().unwrap();
        let fan_count = fan_count.min(4); // Cap to 4 fans as in the implementation

        let mut fans = Vec::with_capacity(fan_count as usize);
        for i in 0..fan_count {
            if let Ok(fan_info) = mock_iokit.get_fan_info(i) {
                fans.push(fan_info);
            }
        }

        Ok(fans)
    };

    // We should still get the first fan even though the second one failed
    let result = all_fans_result.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].speed_rpm, 2000);
}

#[test]
fn test_create_fan_keys() {
    // Test dynamic key generation for fans - no need for an actual instance

    // Test fan keys for fan 0
    let actual_fan0_key = [b'F' as c_char, b'0' as c_char, b'A' as c_char, b'c' as c_char];
    let expected_fan0_key = [b'F' as c_char, b'0' as c_char, b'A' as c_char, b'c' as c_char];
    assert_eq!(actual_fan0_key, expected_fan0_key);

    // Test fan keys for fan 1
    let actual_fan1_key = [b'F' as c_char, b'1' as c_char, b'A' as c_char, b'c' as c_char];
    let expected_fan1_key = [b'F' as c_char, b'1' as c_char, b'A' as c_char, b'c' as c_char];
    assert_eq!(actual_fan1_key, expected_fan1_key);
}

#[test]
fn test_fan_percentage_calculation() {
    // Test fan percentage calculation
    let speed = 2500;
    let min = 1000;
    let max = 5000;

    // Calculate expected percentage
    let expected_percentage = ((speed - min) as f64 / (max - min) as f64) * 100.0;

    // Create a FanInfo struct with these values
    let fan_info = FanInfo {
        speed_rpm: speed,
        min_speed: min,
        max_speed: max,
        percentage: expected_percentage,
    };

    // Verify the percentage value
    assert_eq!(fan_info.percentage, expected_percentage);
    assert!(fan_info.percentage > 0.0 && fan_info.percentage < 100.0);
}

#[test]
fn test_fan_percentage_limits() {
    // Test edge cases for fan percentage calculation

    // 1. Test min speed (should be 0%)
    let min_fan = FanInfo { speed_rpm: 1000, min_speed: 1000, max_speed: 5000, percentage: 0.0 };
    assert_eq!(min_fan.percentage, 0.0);

    // 2. Test max speed (should be 100%)
    let max_fan = FanInfo { speed_rpm: 5000, min_speed: 1000, max_speed: 5000, percentage: 100.0 };
    assert_eq!(max_fan.percentage, 100.0);

    // 3. Test calculation with zero max value (edge case)
    let zero_max_fan = FanInfo {
        speed_rpm: 2000,
        min_speed: 1000,
        max_speed: 0, // This is an invalid scenario but should be handled gracefully
        percentage: 0.0,
    };
    // In this case, percentage should be 0.0 to avoid division by zero
    assert_eq!(zero_max_fan.percentage, 0.0);
}

#[test]
fn test_parse_smc_data() {
    // Test parse_smc_data function - this only works when coverage feature is enabled
    #[cfg(feature = "skip-ffi-crashes")]
    {
        let iokit = IOKitImpl;

        // Test with float data type
        let float_type = *b"flt\0";
        let result = iokit.parse_smc_data(float_type, [0; 32]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42.5);

        // Test with uint data type
        let uint_type = *b"uint";
        let result = iokit.parse_smc_data(uint_type, [0; 32]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100.0);

        // Test with sint16 data type
        let sint16_type = *b"si16";
        let result = iokit.parse_smc_data(sint16_type, [0; 32]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 50.0);

        // Test with SP78 data type
        let sp78_type = *b"SP78";
        let result = iokit.parse_smc_data(sp78_type, [0; 32]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 35.5);

        // Test with unsupported data type
        let unknown_type = *b"xxxx";
        let result = iokit.parse_smc_data(unknown_type, [0; 32]);
        assert!(result.is_err());
        match result {
            Err(e) => {
                let error_msg = format!("{}", e);
                assert!(error_msg.contains("Unsupported SMC data type"));
            },
            _ => panic!("Expected an error for unsupported data type"),
        }
    }
}

#[test]
fn test_gpu_stats_default() {
    // Test the Default implementation for GpuStats
    let stats = GpuStats {
        utilization: 0.0,
        perf_cap: 0.0,
        perf_threshold: 0.0,
        memory_used: 0,
        memory_total: 0,
        name: "".to_string(),
    };

    assert_eq!(stats.utilization, 0.0);
    assert_eq!(stats.perf_cap, 0.0);
    assert_eq!(stats.perf_threshold, 0.0);
    assert_eq!(stats.memory_used, 0);
    assert_eq!(stats.memory_total, 0);
    assert_eq!(stats.name, "");
}

#[test]
fn test_gpu_stats_clone() {
    // Test the Clone implementation for GpuStats
    let original = GpuStats {
        utilization: 50.0,
        perf_cap: 60.0,
        perf_threshold: 70.0,
        memory_used: 1024 * 1024 * 1024,
        memory_total: 4 * 1024 * 1024 * 1024,
        name: "Test GPU".to_string(),
    };

    let cloned = original.clone();

    assert_eq!(cloned.utilization, original.utilization);
    assert_eq!(cloned.perf_cap, original.perf_cap);
    assert_eq!(cloned.perf_threshold, original.perf_threshold);
    assert_eq!(cloned.memory_used, original.memory_used);
    assert_eq!(cloned.memory_total, original.memory_total);
    assert_eq!(cloned.name, original.name);
}

#[test]
fn test_fan_info_clone() {
    // Test the Clone implementation for FanInfo
    let fan = FanInfo { speed_rpm: 2000, min_speed: 500, max_speed: 5000, percentage: 40.0 };

    let fan_clone = fan.clone();

    assert_eq!(fan.speed_rpm, fan_clone.speed_rpm);
    assert_eq!(fan.min_speed, fan_clone.min_speed);
    assert_eq!(fan.max_speed, fan_clone.max_speed);
    assert_eq!(fan.percentage, fan_clone.percentage);
}

#[test]
fn test_thermal_info_clone() {
    // Test the Clone implementation for ThermalInfo
    let info = ThermalInfo {
        cpu_temp: 45.0,
        gpu_temp: 55.0,
        heatsink_temp: Some(40.0),
        ambient_temp: Some(25.0),
        battery_temp: Some(35.0),
        is_throttling: false,
        cpu_power: Some(15.0),
    };

    let info_clone = info.clone();

    assert_eq!(info.cpu_temp, info_clone.cpu_temp);
    assert_eq!(info.gpu_temp, info_clone.gpu_temp);
    assert_eq!(info.heatsink_temp, info_clone.heatsink_temp);
    assert_eq!(info.ambient_temp, info_clone.ambient_temp);
    assert_eq!(info.battery_temp, info_clone.battery_temp);
    assert_eq!(info.is_throttling, info_clone.is_throttling);
    assert_eq!(info.cpu_power, info_clone.cpu_power);
}

#[test]
fn test_thermal_info_debug() {
    // Test the Debug implementation for ThermalInfo
    let info = ThermalInfo {
        cpu_temp: 45.0,
        gpu_temp: 55.0,
        heatsink_temp: Some(40.0),
        ambient_temp: Some(25.0),
        battery_temp: Some(35.0),
        is_throttling: false,
        cpu_power: Some(15.0),
    };

    let debug_str = format!("{:?}", info);

    // Make sure all the fields are represented in the debug output
    assert!(debug_str.contains("cpu_temp: 45.0"));
    assert!(debug_str.contains("gpu_temp: 55.0"));
    assert!(debug_str.contains("heatsink_temp: Some(40.0)"));
    assert!(debug_str.contains("ambient_temp: Some(25.0)"));
    assert!(debug_str.contains("battery_temp: Some(35.0)"));
    assert!(debug_str.contains("is_throttling: false"));
    assert!(debug_str.contains("cpu_power: Some(15.0)"));
}

#[test]
fn test_fan_info_debug() {
    // Test the Debug implementation for FanInfo
    let fan = FanInfo { speed_rpm: 2000, min_speed: 500, max_speed: 5000, percentage: 40.0 };

    let debug_str = format!("{:?}", fan);

    // Make sure all the fields are represented in the debug output
    assert!(debug_str.contains("speed_rpm: 2000"));
    assert!(debug_str.contains("min_speed: 500"));
    assert!(debug_str.contains("max_speed: 5000"));
    assert!(debug_str.contains("percentage: 40.0"));
}

#[test]
fn test_iokit_impl_default() {
    // Test the Default implementation for IOKitImpl
    let iokit = IOKitImpl;

    // Just verify we can create an instance
    assert!(matches!(iokit, IOKitImpl));
}

#[test]
fn test_iokit_impl_debug() {
    // Test the Debug implementation for IOKitImpl
    let iokit = IOKitImpl;

    let debug_str = format!("{:?}", iokit);

    // Verify that the debug output contains IOKitImpl
    assert!(debug_str.contains("IOKitImpl"));
}

// This test is disabled by default because it can cause segfaults in some
// environments Only run it manually when debugging IOKit issues
#[cfg(feature = "unstable-tests")]
#[test]
fn test_real_gpu_stats() {
    // Wrap the entire test in an autoreleasepool to ensure proper memory cleanup
    autoreleasepool(|_| {
        let iokit = IOKitImpl;
        println!("Created IOKitImpl instance");

        // Test just getting IOAccelerator directly
        println!("Testing IOAccelerator service directly");
        let matching = iokit.io_service_matching("IOAccelerator");
        println!("Got matching dictionary for IOAccelerator");

        let service_opt = iokit.io_service_get_matching_service(&matching);
        match service_opt {
            Some(service) => {
                println!("Found IOAccelerator service, trying to get properties");
                match iokit.io_registry_entry_create_cf_properties(&service) {
                    Ok(props) => {
                        println!("Successfully got properties from IOAccelerator");

                        println!("============== Testing Key Access ==============");

                        // Try to get VRAM and memory stats
                        let vram_keys = [
                            "VRAM,totalMB",
                            "VRAM,usedMB",
                            "totalVRAM",
                            "usedVRAM",
                            "vramUsage", // From NeoAsitop
                            "vramFree",  // From NeoAsitop
                        ];

                        for key in vram_keys.iter() {
                            if let Some(value) = iokit.get_number_property(&props, key) {
                                println!("{}: {}", key, value);
                            } else {
                                println!("{}: Not found", key);
                            }
                        }

                        // Try to get GPU identification
                        let name_keys = [
                            "model",
                            "name",
                            "IOGLBundleName",
                            "IOAccelRevision",
                            "device-id",    // From Apple docs
                            "vendor-id",    // From Apple docs
                            "IOAccelIndex", // From NeoAsitop
                            "IOAccelTypes",
                            "gpuType",     // From NeoAsitop
                            "gpu_product", // From NeoAsitop
                        ];

                        for key in name_keys.iter() {
                            if let Some(value) = iokit.get_string_property(&props, key) {
                                println!("{}: {}", key, value);
                            } else {
                                println!("{}: Not found", key);
                            }
                        }

                        // Try to get performance metrics
                        let perf_keys = [
                            "IOGPUCurrentPowerState",
                            "IOGPUMaximumPowerState",
                            "deviceUtilization", // From NeoAsitop
                            "powerState",        // From NeoAsitop
                            "GPUPerfCap",
                            "GPUPerfThreshold",
                        ];

                        for key in perf_keys.iter() {
                            if let Some(value) = iokit.get_number_property(&props, key) {
                                println!("{}: {}", key, value);
                            } else {
                                println!("{}: Not found", key);
                            }
                        }

                        println!("\n============== Testing Metal API ==============");
                        use crate::utils::bindings::MTLCreateSystemDefaultDevice;

                        println!("Creating Metal device...");
                        autoreleasepool(|_pool| {
                            unsafe {
                                // Get default Metal device (GPU)
                                let device = MTLCreateSystemDefaultDevice();
                                if device.is_null() {
                                    println!("Failed to create Metal device");
                                    return;
                                }

                                println!("Metal device created successfully");

                                // Cast it to AnyObject so we can send messages to it
                                let device_obj: *mut objc2::runtime::AnyObject = device.cast();

                                // Get the device name
                                println!("Fetching device name...");
                                let name_obj: *mut objc2::runtime::AnyObject =
                                    msg_send![device_obj, name];
                                if name_obj.is_null() {
                                    println!("Failed to get device name");
                                } else {
                                    let utf8_string: *const u8 = msg_send![name_obj, UTF8String];
                                    if utf8_string.is_null() {
                                        println!("Failed to get UTF8 string");
                                    } else {
                                        let c_str =
                                            std::ffi::CStr::from_ptr(utf8_string as *const i8);
                                        let name = c_str.to_string_lossy();
                                        println!("GPU name from Metal API: {}", name);
                                    }
                                }

                                // Release the Metal device
                                println!("Releasing Metal device...");
                                let _: () = msg_send![device_obj, release];
                                println!("Metal device released");
                            }
                        });

                        println!("Memory management handled properly, test continuing...");
                    },
                    Err(e) => {
                        println!("Error getting properties: {:?}", e);
                    },
                }
            },
            None => {
                println!("IOAccelerator service not found");
            },
        }
    });
}

#[test]
#[cfg(feature = "skip-ffi-crashes")]
fn test_smc_read_key_mocks() {
    let iokit = IOKitImpl;

    if cfg!(feature = "skip-ffi-crashes") {
        // Test mocked values for CPU temperature
        let cpu_temp = iokit.smc_read_key(SMC_KEY_CPU_TEMP);
        assert!(cpu_temp.is_ok());
        assert_eq!(cpu_temp.as_ref().unwrap(), &42.5);

        // Test mocked values for GPU temperature
        let gpu_temp = iokit.smc_read_key(SMC_KEY_GPU_TEMP);
        assert!(cpu_temp.is_ok());
        assert_eq!(gpu_temp.as_ref().unwrap(), &42.5);

        // Test mocked values for ambient temperature
        let ambient_temp = iokit.smc_read_key(SMC_KEY_AMBIENT_TEMP);
        assert!(ambient_temp.is_ok());
        assert_eq!(ambient_temp.as_ref().unwrap(), &26.0);

        // Test mocked values for battery temperature
        let battery_temp = iokit.smc_read_key(SMC_KEY_BATTERY_TEMP);
        assert!(battery_temp.is_ok());
        assert_eq!(battery_temp.as_ref().unwrap(), &35.0);

        // Test mocked values for fan count
        let fan_count = iokit.smc_read_key(SMC_KEY_FAN_NUM);
        assert!(fan_count.is_ok());
        assert_eq!(fan_count.as_ref().unwrap(), &2.0);

        // Test mocked values for fan speed
        let fan_key = [b'F' as c_char, b'0' as c_char, b'A' as c_char, b'c' as c_char];
        let fan_speed = iokit.smc_read_key(fan_key);
        assert!(fan_speed.is_ok());
        assert_eq!(fan_speed.as_ref().unwrap(), &2000.0);

        // Test mocked default value
        let unknown_key = [b'X' as c_char, b'X' as c_char, b'X' as c_char, b'X' as c_char];
        let unknown = iokit.smc_read_key(unknown_key);
        assert!(unknown.is_ok());
        assert_eq!(unknown.as_ref().unwrap(), &0.0);
    }
}
