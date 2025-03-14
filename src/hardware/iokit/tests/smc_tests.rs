#![allow(unused_imports)]

use std::os::raw::c_char;

use crate::{
    error::{Error, Result},
    hardware::iokit::{IOKit, IOKitImpl, MockIOKit},
    utils::bindings::{
        smc_key_from_chars,
        SMC_KEY_AMBIENT_TEMP,
        SMC_KEY_BATTERY_TEMP,
        SMC_KEY_CPU_TEMP,
        SMC_KEY_FAN_NUM,
        SMC_KEY_GPU_TEMP,
    },
};

#[test]
fn test_smc_key_from_chars() {
    let iokit = IOKitImpl::default();
    let key = [b'T' as c_char, b'C' as c_char, b'0' as c_char, b'P' as c_char];
    let result = smc_key_from_chars(key);
    let expected = (b'T' as u32) << 24 | (b'C' as u32) << 16 | (b'0' as u32) << 8 | (b'P' as u32);
    assert_eq!(result, expected);
}

#[test]
fn test_smc_read_key_mock_temperatures() {
    let iokit = IOKitImpl;
    
    let cpu_temp = iokit.smc_read_key(SMC_KEY_CPU_TEMP).unwrap();
    assert_eq!(cpu_temp, 42.5);

    let gpu_temp = iokit.smc_read_key(SMC_KEY_GPU_TEMP).unwrap();
    assert_eq!(gpu_temp, 42.5);

    let ambient_temp = iokit.smc_read_key(SMC_KEY_AMBIENT_TEMP).unwrap();
    assert_eq!(ambient_temp, 26.0);

    let battery_temp = iokit.smc_read_key(SMC_KEY_BATTERY_TEMP).unwrap();
    assert_eq!(battery_temp, 35.0);
}

#[test]
fn test_smc_read_key_mock_fans() {
    let iokit = IOKitImpl;
    
    let fan_count = iokit.smc_read_key(SMC_KEY_FAN_NUM).unwrap();
    assert_eq!(fan_count, 2.0);

    let fan_key = [b'F' as c_char, b'0' as c_char, b'A' as c_char, b'c' as c_char];
    let fan_speed = iokit.smc_read_key(fan_key).unwrap();
    assert_eq!(fan_speed, 2000.0);
}

#[test]
fn test_smc_read_key_mock_default() {
    let iokit = IOKitImpl;
    let unknown_key = [b'X' as c_char, b'Y' as c_char, b'Z' as c_char, b'W' as c_char];
    let value = iokit.smc_read_key(unknown_key).unwrap();
    assert_eq!(value, 0.0);
}

#[cfg(feature = "skip-ffi-crashes")]
#[test]
fn test_parse_smc_data_types() {
    let iokit = IOKitImpl;
    
    let result = iokit.parse_smc_data(*b"flt ", [0; 32]).unwrap();
    assert_eq!(result, 42.5);

    let result = iokit.parse_smc_data(*b"uint", [0; 32]).unwrap();
    assert_eq!(result, 100.0);

    let result = iokit.parse_smc_data(*b"si16", [0; 32]).unwrap();
    assert_eq!(result, 50.0);

    let result = iokit.parse_smc_data(*b"SP78", [0; 32]).unwrap();
    assert_eq!(result, 35.5);
}

#[cfg(feature = "skip-ffi-crashes")]
#[test]
fn test_parse_smc_data_invalid_type() {
    let iokit = IOKitImpl;
    let result = iokit.parse_smc_data(*b"invalid", [0; 32]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unsupported SMC data type"));
}

#[test]
fn test_smc_read_key_error_handling() {
    let mut mock = MockIOKit::new();

    mock.expect_read_smc_key().returning(|key| {
        if key == [0, 0, 0, 0] {
            Err(Error::IOKit("Invalid SMC key".to_string()))
        } else {
            Ok(42.5)
        }
    });

    let result = mock.read_smc_key([0, 0, 0, 0]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid SMC key"));

    let result = mock.read_smc_key(*b"TC0P");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42.5);
}

#[cfg(feature = "skip-ffi-crashes")]
mod additional_safe_tests {
    use super::*;

    #[test]
    fn test_smc_read_key_all_temperature_keys() {
        let iokit = IOKitImpl;
        
        let temp_keys = [
            SMC_KEY_CPU_TEMP,
            SMC_KEY_GPU_TEMP,
            SMC_KEY_AMBIENT_TEMP,
            SMC_KEY_BATTERY_TEMP,
        ];
        
        for key in temp_keys.iter() {
            let result = iokit.smc_read_key(*key);
            assert!(result.is_ok());
            assert!(result.unwrap() > 0.0);
        }
    }

    #[test]
    fn test_smc_read_key_fan_keys() {
        let iokit = IOKitImpl;
        
        let result = iokit.smc_read_key(SMC_KEY_FAN_NUM);
        assert!(result.is_ok());
        let fan_count = result.unwrap();
        assert!(fan_count > 0.0);
        
        for i in 0..fan_count as u32 {
            let fan_key = [
                b'F' as c_char,
                (b'0' + i as u8) as c_char,
                b'A' as c_char,
                b'c' as c_char,
            ];
            let result = iokit.smc_read_key(fan_key);
            assert!(result.is_ok());
        }
    }
} 