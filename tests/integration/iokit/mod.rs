#![allow(missing_docs)]

use std::os::raw::c_char;

use darwin_metrics::{
    error::{Error, Result},
    hardware::iokit::{IOKit, IOKitImpl},
    utils::bindings::{
        smc_key_from_chars, SMC_KEY_AMBIENT_TEMP, SMC_KEY_BATTERY_TEMP, SMC_KEY_CPU_TEMP, SMC_KEY_FAN_NUM,
        SMC_KEY_GPU_TEMP,
    },
};

use darwin_metrics::utils::ffi::SmcKey;

#[test]
fn test_iokit_new() {
    let iokit = IOKitImpl::new();
    assert!(iokit.is_ok());
}

#[test]
fn test_iokit_get_thermal_info() {
    let iokit = IOKitImpl::new().unwrap();
    let thermal_info = iokit.get_thermal_info();
    assert!(thermal_info.is_ok());

    let info = thermal_info.unwrap();
    assert!(info.cpu_temp >= 0.0);
}

#[test]
fn test_iokit_get_all_fans() {
    let iokit = IOKitImpl::new().unwrap();
    let fans = iokit.get_all_fans();
    assert!(fans.is_ok());
}

#[test]
fn test_iokit_get_cpu_temperature() {
    let iokit = IOKitImpl::new().unwrap();
    let temp = iokit.get_cpu_temperature("IOService");
    assert!(temp.is_ok());
    assert!(temp.unwrap() >= 0.0);
}

#[test]
fn test_iokit_read_smc_key() {
    let iokit = IOKitImpl::new().unwrap();
    let key = [b'T' as c_char, b'C' as c_char, b'0' as c_char, b'P' as c_char];
    let result = iokit.read_smc_key(key);

    // The test should pass whether the key is available or not
    if let Ok(Some(value)) = result {
        assert!(value >= 0.0);
    }
}

#[test]
#[ignore]
fn test_iokit_get_gpu_stats() {
    let iokit = IOKitImpl::new().unwrap();
    let stats = iokit.get_gpu_stats();

    if let Ok(gpu_stats) = stats {
        assert!(gpu_stats.utilization >= 0.0);
        assert!(gpu_stats.utilization <= 100.0);
    }
}

#[test]
fn test_iokit_get_physical_and_logical_cores() {
    let iokit = IOKitImpl::new().unwrap();

    let physical_cores = iokit.get_physical_cores();
    assert!(physical_cores.is_ok());
    assert!(physical_cores.unwrap() > 0);

    let logical_cores = iokit.get_logical_cores();
    assert!(logical_cores.is_ok());
    assert!(logical_cores.unwrap() > 0);
}

#[test]
fn test_iokit_get_core_usage() {
    let iokit = IOKitImpl::new().unwrap();
    let usage = iokit.get_core_usage();

    assert!(usage.is_ok());
    let core_usage = usage.unwrap();
    assert!(!core_usage.is_empty());

    for usage in core_usage {
        assert!(usage >= 0.0);
        assert!(usage <= 100.0);
    }
}

#[test]
fn test_smc_key_from_chars() {
    let key = SmcKey::from_chars(['T', 'A', '0', 'P']);
    assert_eq!(key.to_string(), "TA0P");
}
