use crate::{
    error::{Error, Result},
    power::{Power, PowerConsumption, PowerError, PowerState},
    tests::common::builders::power::TestPowerBuilder,
};
use std::os::raw::c_char;

// Constants from the power module
const SMC_KEY_CPU_POWER: [c_char; 4] = [b'P' as c_char, b'C' as c_char, b'P' as c_char, b'C' as c_char];
const SMC_KEY_GPU_POWER: [c_char; 4] = [b'P' as c_char, b'G' as c_char, b'P' as c_char, b'G' as c_char];

#[test]
fn test_power_new() -> Result<()> {
    let power = TestPowerBuilder::new().build()?;
    // No assertion needed - test passes if it doesn't panic
    Ok(())
}

#[test]
fn test_power_consumption() -> Result<()> {
    let power = TestPowerBuilder::new()
        .package_power(10.0)
        .cores_power(5.0)
        .gpu_power(Some(2.0))
        .dram_power(Some(1.0))
        .neural_engine_power(Some(0.5))
        .power_state(PowerState::AC)
        .battery_percentage(Some(80.0))
        .power_impact(Some(12.0))
        .build()?;

    let consumption = power.get_power_consumption()?;
    assert!(consumption.package > 0.0, "Package power should be positive");
    assert!(consumption.cores > 0.0, "Core power should be positive");
    assert!(consumption.gpu.is_some(), "GPU power should be present");
    assert!(consumption.dram.is_some(), "DRAM power should be present");
    assert!(consumption.neural_engine.is_some(), "Neural engine power should be present");
    assert_eq!(consumption.power_state, PowerState::AC, "Power state should be AC");

    Ok(())
}

#[test]
fn test_power_throttling() -> Result<()> {
    let power = TestPowerBuilder::new().throttling(false).build()?;

    let is_throttling = power.is_power_throttling()?;
    assert!(!is_throttling, "Mock implementation should not report throttling");

    Ok(())
}

#[test]
fn test_read_smc_power_key() -> Result<()> {
    let power =
        TestPowerBuilder::new().smc_key_value(SMC_KEY_CPU_POWER, 10.0).smc_key_value(SMC_KEY_GPU_POWER, 5.0).build()?;

    // Test valid keys
    let cpu_power = power.read_smc_power_key(SMC_KEY_CPU_POWER)?;
    assert!(cpu_power > 0.0, "CPU power should be positive");

    let gpu_power = power.read_smc_power_key(SMC_KEY_GPU_POWER)?;
    assert!(gpu_power > 0.0, "GPU power should be positive");

    // Test unknown key (should return 0.0)
    let unknown_key = [b'X' as c_char, b'X' as c_char, b'X' as c_char, b'X' as c_char];
    let unknown_power = power.read_smc_power_key(unknown_key)?;
    assert_eq!(unknown_power, 0.0, "Unknown key should return 0.0");

    Ok(())
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
    let error = Error::InvalidData { context: "test".to_string(), value: Some("test".to_string()) };
    let power_error = PowerError::from(error);
    assert!(matches!(power_error, PowerError::InvalidData));

    let error = Error::ServiceNotFound("test".to_string());
    let power_error = PowerError::from(error);
    assert!(matches!(power_error, PowerError::ServiceError(_)));
}
