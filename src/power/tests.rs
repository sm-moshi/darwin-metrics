use super::*;
use crate::error::Error;
use crate::hardware::iokit::IOKitImpl;
use std::os::raw::c_char;

#[test]
fn test_power_new() {
    let _power = Power::new();
    // No assertion needed - test passes if it doesn't panic
}

#[test]
fn test_power_consumption() {
    let power = Power::new();
    let result = power.get_power_consumption();
    assert!(result.is_ok(), "Should return Ok result");

    let consumption = result.unwrap();
    assert!(consumption.package > 0.0, "Package power should be positive");
    assert!(consumption.cores > 0.0, "Core power should be positive");
    assert!(consumption.gpu.is_some(), "GPU power should be present");
    assert!(consumption.dram.is_some(), "DRAM power should be present");
    assert!(consumption.neural_engine.is_some(), "Neural engine power should be present");
    assert_eq!(consumption.power_state, PowerState::AC, "Power state should be AC");
}

#[test]
fn test_power_throttling() {
    let power = Power::new();
    let result = power.is_power_throttling();
    assert!(result.is_ok(), "Should return Ok result");

    // Our mock implementation always returns false for throttling
    let is_throttling = result.unwrap();
    assert!(!is_throttling, "Mock implementation should not report throttling");
}

#[test]
fn test_read_smc_power_key() {
    let power = Power::new();

    // Test valid keys
    let cpu_power = power.read_smc_power_key(SMC_KEY_CPU_POWER);
    assert!(cpu_power.is_ok(), "CPU power key should return Ok result");
    assert!(cpu_power.unwrap() > 0.0, "CPU power should be positive");

    let gpu_power = power.read_smc_power_key(SMC_KEY_GPU_POWER);
    assert!(gpu_power.is_ok(), "GPU power key should return Ok result");
    assert!(gpu_power.unwrap() > 0.0, "GPU power should be positive");

    // Test unknown key (should return 0.0)
    let unknown_key = [b'X' as c_char, b'X' as c_char, b'X' as c_char, b'X' as c_char];
    let unknown_power = power.read_smc_power_key(unknown_key);
    assert!(unknown_power.is_ok(), "Unknown power key should return Ok result");
    assert_eq!(unknown_power.unwrap(), 0.0, "Unknown key should return 0.0");
}

#[test]
fn test_convenience_functions() {
    let result = get_power_consumption();
    assert!(result.is_ok(), "get_power_consumption should return Ok result");

    let consumption = result.unwrap();
    assert!(consumption.package > 0.0, "Package power should be positive");
}

#[test]
fn test_power_state_enum() {
    let state = PowerState::Battery;
    assert!(matches!(state, PowerState::Battery));

    let state = PowerState::AC;
    assert!(matches!(state, PowerState::AC));

    let state = PowerState::Charging;
    assert!(matches!(state, PowerState::Charging));
}

#[test]
fn test_power_consumption_struct() {
    let consumption = PowerConsumption {
        package: 10.0,
        cores: 5.0,
        gpu: Some(2.0),
        dram: Some(1.0),
        neural_engine: Some(0.5),
        power_state: PowerState::AC,
        battery_percentage: Some(80.0),
        power_impact: Some(12.0),
    };

    assert_eq!(consumption.package, 10.0);
    assert_eq!(consumption.cores, 5.0);
    assert_eq!(consumption.gpu, Some(2.0));
    assert_eq!(consumption.dram, Some(1.0));
    assert_eq!(consumption.neural_engine, Some(0.5));
    assert_eq!(consumption.power_state, PowerState::AC);
    assert_eq!(consumption.battery_percentage, Some(80.0));
    assert_eq!(consumption.power_impact, Some(12.0));
}

#[test]
fn test_power_error_conversion() {
    let error = Error::InvalidData { context: "test".to_string(), value: "test".to_string() };
    let power_error = PowerError::from(error);
    assert!(matches!(power_error, PowerError::InvalidData));

    let error = Error::ServiceNotFound("test".to_string());
    let power_error = PowerError::from(error);
    assert!(matches!(power_error, PowerError::ServiceError(_)));
}
