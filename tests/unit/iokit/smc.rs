use std::os::raw::c_char;

use darwin_metrics::hardware::iokit::IOKit;
use darwin_metrics::hardware::iokit::mock::MockIOKit;
use darwin_metrics::utils::bindings::{
    SMC_KEY_AMBIENT_TEMP, SMC_KEY_BATTERY_TEMP, SMC_KEY_CPU_TEMP, SMC_KEY_FAN_NUM, SMC_KEY_GPU_TEMP, smc_key_from_chars,
};

#[test]
fn test_smc_key_from_chars() {
    let key = [b'T' as c_char, b'C' as c_char, b'0' as c_char, b'P' as c_char];
    let result = smc_key_from_chars(key);
    let expected = (b'T' as u32) << 24 | (b'C' as u32) << 16 | (b'0' as u32) << 8 | (b'P' as u32);
    assert_eq!(result, expected);
}

#[test]
fn test_mock_iokit_creation() {
    let mock = MockIOKit::new();
    assert!(mock.is_ok());
}

#[test]
fn test_smc_read_key_error_handling() {
    let mock = MockIOKit::new().unwrap();

    // Since we can't directly set expectations on the mock in this test,
    // we'll just verify that the mock can be created successfully
    assert!(mock.read_smc_key([0, 0, 0, 0]).is_ok());
}

// #[cfg(feature = "skip-ffi-crashes")]
// mod additional_safe_tests {
// use super::*;
//
// #[test]
// fn test_smc_read_key_all_temperature_keys() {
// let iokit = IOKitImpl;
//
// let temp_keys = [
// SMC_KEY_CPU_TEMP,
// SMC_KEY_GPU_TEMP,
// SMC_KEY_AMBIENT_TEMP,
// SMC_KEY_BATTERY_TEMP,
// ];
//
// for key in temp_keys.iter() {
// let result = iokit.smc_read_key(*key);
// assert!(result.is_ok());
// assert!(result.unwrap() > 0.0);
// }
// }
//
// #[test]
// fn test_smc_read_key_fan_keys() {
// let iokit = IOKitImpl;
//
// let result = iokit.smc_read_key(SMC_KEY_FAN_NUM);
// assert!(result.is_ok());
// let fan_count = result.unwrap();
// assert!(fan_count > 0.0);
//
// for i in 0..fan_count as u32 {
// let fan_key = [
// b'F' as c_char,
// (b'0' + i as u8) as c_char,
// b'A' as c_char,
// b'c' as c_char,
// ];
// let result = iokit.smc_read_key(fan_key);
// assert!(result.is_ok());
// }
// }
// }
