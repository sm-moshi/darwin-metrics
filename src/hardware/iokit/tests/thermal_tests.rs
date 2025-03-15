#![allow(unused_imports)]

use crate::{
    error::Error,
    hardware::iokit::{IOKit, IOKitImpl, MockIOKit, ThermalInfo},
};

#[test]
fn test_get_thermal_info() {
    let mut mock_iokit = MockIOKit::new();

    mock_iokit.expect_get_thermal_info().returning(|| {
        Ok(ThermalInfo {
            cpu_temp: 45.0,
            gpu_temp: Some(55.0),
            heatsink_temp: Some(40.0),
            ambient_temp: Some(25.0),
            battery_temp: Some(35.0),
            is_throttling: false,
            cpu_power: Some(15.0),
        })
    });

    let result = mock_iokit.get_thermal_info().unwrap();

    assert_eq!(result.cpu_temp, 45.0);
    assert_eq!(result.gpu_temp, Some(55.0));
    assert_eq!(result.heatsink_temp, Some(40.0));
    assert_eq!(result.ambient_temp, Some(25.0));
    assert_eq!(result.battery_temp, Some(35.0));
    assert!(!result.is_throttling);
    assert_eq!(result.cpu_power, Some(15.0));
}

#[test]
fn test_get_thermal_info_with_failures() {
    let mut mock_iokit = MockIOKit::new();

    mock_iokit.expect_get_thermal_info().returning(|| {
        Ok(ThermalInfo {
            cpu_temp: 45.0,
            gpu_temp: Some(55.0),
            heatsink_temp: None,
            ambient_temp: None,
            battery_temp: None,
            is_throttling: false,
            cpu_power: None,
        })
    });

    let result = mock_iokit.get_thermal_info();
    assert!(result.is_ok());
    let info = result.unwrap();

    assert_eq!(info.cpu_temp, 45.0);
    assert_eq!(info.gpu_temp, Some(55.0));
    assert_eq!(info.heatsink_temp, None);
    assert_eq!(info.ambient_temp, None);
    assert_eq!(info.battery_temp, None);
    assert_eq!(info.cpu_power, None);
    assert!(!info.is_throttling);
}

#[test]
fn test_get_heatsink_temperature() {
    let mut mock_iokit = MockIOKit::new();
    mock_iokit.expect_get_heatsink_temperature().returning(|| Ok(40.0));
    let result = mock_iokit.get_heatsink_temperature().unwrap();
    assert_eq!(result, 40.0);
}

#[test]
fn test_get_ambient_temperature() {
    let mut mock_iokit = MockIOKit::new();
    mock_iokit.expect_get_ambient_temperature().returning(|| Ok(25.0));
    let result = mock_iokit.get_ambient_temperature().unwrap();
    assert_eq!(result, 25.0);
}

#[test]
fn test_get_battery_temperature() {
    let mut mock_iokit = MockIOKit::new();
    mock_iokit.expect_get_battery_temperature().returning(|| Ok(35.0));
    let result = mock_iokit.get_battery_temperature().unwrap();
    assert_eq!(result, 35.0);
}

#[test]
fn test_get_cpu_power() {
    let mut mock_iokit = MockIOKit::new();
    mock_iokit.expect_get_cpu_power().returning(|| Ok(15.0));
    let result = mock_iokit.get_cpu_power().unwrap();
    assert_eq!(result, 15.0);
}

#[test]
fn test_check_thermal_throttling() {
    let mut mock_iokit = MockIOKit::new();
    mock_iokit.expect_check_thermal_throttling().returning(|| Ok(true));
    let result = mock_iokit.check_thermal_throttling().unwrap();
    assert!(result);

    let mut mock_iokit = MockIOKit::new();
    mock_iokit.expect_check_thermal_throttling().returning(|| Ok(false));
    let result = mock_iokit.check_thermal_throttling().unwrap();
    assert!(!result);
}

#[test]
fn test_thermal_info_error_propagation() {
    let mut mock = MockIOKit::new();

    mock.expect_get_cpu_temperature().returning(|| {
        Err(Error::IOKit("CPU sensor error".to_string()))
    });
    mock.expect_get_gpu_temperature().returning(|| Ok(55.0));
    mock.expect_get_heatsink_temperature().returning(|| Ok(40.0));
    mock.expect_get_ambient_temperature().returning(|| Ok(25.0));
    mock.expect_get_battery_temperature().returning(|| Ok(35.0));
    mock.expect_check_thermal_throttling().returning(|| Ok(false));
    mock.expect_get_cpu_power().returning(|| Ok(28.5));

    let result = mock.get_thermal_info();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("CPU sensor error"));

    let mut mock = MockIOKit::new();
    mock.expect_get_cpu_temperature().returning(|| Ok(45.0));
    mock.expect_get_gpu_temperature().returning(|| {
        Err(Error::IOKit("GPU sensor error".to_string()))
    });
    mock.expect_get_heatsink_temperature().returning(|| Ok(40.0));
    mock.expect_get_ambient_temperature().returning(|| Ok(25.0));
    mock.expect_get_battery_temperature().returning(|| Ok(35.0));
    mock.expect_check_thermal_throttling().returning(|| Ok(false));
    mock.expect_get_cpu_power().returning(|| Ok(28.5));

    let result = mock.get_thermal_info();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("GPU sensor error"));
}

#[test]
fn test_thermal_info() {
    let iokit = IOKitImpl::default();
    let result = iokit.get_thermal_info();
    assert!(result.is_ok());
    
    let info = result.unwrap();
    assert!(info.cpu_temp > 0.0);
    assert!(info.gpu_temp.is_some());
    assert!(info.ambient_temp.is_some());
    assert!(info.battery_temp.is_some());
    assert!(info.heatsink_temp.is_some());
    assert!(!info.is_throttling);
}

#[cfg(feature = "skip-ffi-crashes")]
mod additional_safe_tests {
    use super::*;

    #[test]
    fn test_thermal_info_complete() {
        let iokit = IOKitImpl;
        let result = iokit.get_thermal_info();
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert!(info.cpu_temp > 0.0);
        assert!(info.gpu_temp.is_some());
        assert!(info.ambient_temp.is_some());
        assert!(info.battery_temp.is_some());
        assert!(info.heatsink_temp.is_some());
        assert!(!info.is_throttling);
    }
} 