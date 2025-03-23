use darwin_metrics::utils::bindings::SmcKey;

#[test]
fn test_smc_key_from_chars() {
    let key = SmcKey::from_chars(['T', 'A', '0', 'P']);
    assert_eq!(key.to_string(), "TA0P");
}

#[test]
fn test_smc_key_edge_cases() {
    // Test with special characters
    let key1 = SmcKey::from_chars(['#', '@', '!', '?']);
    assert_eq!(key1.to_string(), "#@!?");

    // Test with numbers
    let key2 = SmcKey::from_chars(['1', '2', '3', '4']);
    assert_eq!(key2.to_string(), "1234");

    // Test with lowercase letters
    let key3 = SmcKey::from_chars(['a', 'b', 'c', 'd']);
    assert_eq!(key3.to_string(), "abcd");

    // Test with mixed characters
    let key4 = SmcKey::from_chars(['T', '3', 'a', '!']);
    assert_eq!(key4.to_string(), "T3a!");
}

#[test]
fn test_smc_key_common_keys() {
    // Test some common SMC keys
    let cpu_temp = SmcKey::from_chars(['T', 'C', '0', 'P']); // CPU temperature
    assert_eq!(cpu_temp.to_string(), "TC0P");

    let fan_speed = SmcKey::from_chars(['F', '0', 'A', 'c']); // Fan 0 actual speed
    assert_eq!(fan_speed.to_string(), "F0Ac");

    let gpu_temp = SmcKey::from_chars(['T', 'G', '0', 'P']); // GPU temperature
    assert_eq!(gpu_temp.to_string(), "TG0P");
}

#[test]
fn test_smc_key_from_str() {
    // Test valid keys
    let key1 = SmcKey::from_str("TA0P").unwrap();
    assert_eq!(key1.to_string(), "TA0P");

    let key2 = SmcKey::from_str("TC0P").unwrap();
    assert_eq!(key2.to_string(), "TC0P");

    // Test with special characters
    let key3 = SmcKey::from_str("#@!?").unwrap();
    assert_eq!(key3.to_string(), "#@!?");
}

#[test]
fn test_smc_key_from_str_errors() {
    // Test with too short string
    let err1 = SmcKey::from_str("ABC").err().unwrap();
    assert!(err1.contains("must be exactly 4 characters"));

    // Test with too long string
    let err2 = SmcKey::from_str("ABCDE").err().unwrap();
    assert!(err2.contains("must be exactly 4 characters"));

    // Test with empty string
    let err3 = SmcKey::from_str("").err().unwrap();
    assert!(err3.contains("must be exactly 4 characters"));
}

#[test]
fn test_smc_key_to_chars() {
    // Test ASCII conversion
    let key1 = SmcKey::from_str("TC0P").unwrap();
    let chars1 = key1.to_chars();
    assert_eq!(chars1, [84, 67, 48, 80]); // ASCII values for 'T', 'C', '0', 'P'

    // Test with lowercase letters
    let key2 = SmcKey::from_str("abcd").unwrap();
    let chars2 = key2.to_chars();
    assert_eq!(chars2, [97, 98, 99, 100]); // ASCII values for 'a', 'b', 'c', 'd'

    // Test with special characters
    let key3 = SmcKey::from_str("#@!?").unwrap();
    let chars3 = key3.to_chars();
    assert_eq!(chars3, [35, 64, 33, 63]); // ASCII values for '#', '@', '!', '?'

    // Test round-trip conversion
    let original_key = SmcKey::from_chars(['T', 'C', '0', 'P']);
    let chars = original_key.to_chars();
    let new_key = SmcKey {
        key: [chars[0] as u8, chars[1] as u8, chars[2] as u8, chars[3] as u8],
    };
    assert_eq!(original_key.to_string(), new_key.to_string());
}
