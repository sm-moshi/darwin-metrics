#![allow(unused_imports)]

use crate::hardware::iokit::{FanInfo, GpuStats, IOKitImpl, ThermalInfo};

#[test]
fn test_debug_implementations() {
    let gpu_stats = GpuStats::default();
    let fan_info = FanInfo {
        speed_rpm: 1000,
        min_speed: 500,
        max_speed: 2000,
        percentage: 50.0,
    };
    let thermal_info = ThermalInfo {
        cpu_temp: 42.0,
        gpu_temp: 45.0,
        heatsink_temp: Some(40.0),
        ambient_temp: Some(25.0),
        battery_temp: Some(35.0),
        is_throttling: false,
        cpu_power: Some(15.0),
    };
    
    assert!(!format!("{:?}", gpu_stats).is_empty());
    assert!(!format!("{:?}", fan_info).is_empty());
    assert!(!format!("{:?}", thermal_info).is_empty());
    assert!(!format!("{:?}", IOKitImpl).is_empty());
}

#[test]
fn test_clone_implementations() {
    let gpu_stats = GpuStats {
        utilization: 50.0,
        perf_cap: 75.0,
        perf_threshold: 100.0,
        memory_used: 1024,
        memory_total: 2048,
        name: "Test GPU".to_string(),
    };
    let fan_info = FanInfo {
        speed_rpm: 1000,
        min_speed: 500,
        max_speed: 2000,
        percentage: 50.0,
    };
    let thermal_info = ThermalInfo {
        cpu_temp: 42.0,
        gpu_temp: 45.0,
        heatsink_temp: Some(40.0),
        ambient_temp: Some(25.0),
        battery_temp: Some(35.0),
        is_throttling: false,
        cpu_power: Some(15.0),
    };
    
    let gpu_stats_clone = gpu_stats.clone();
    let fan_info_clone = fan_info.clone();
    let thermal_info_clone = thermal_info.clone();
    
    assert_eq!(format!("{:?}", gpu_stats), format!("{:?}", gpu_stats_clone));
    assert_eq!(format!("{:?}", fan_info), format!("{:?}", fan_info_clone));
    assert_eq!(format!("{:?}", thermal_info), format!("{:?}", thermal_info_clone));
}

#[test]
fn test_gpu_stats_clone() {
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
    let fan = FanInfo { speed_rpm: 2000, min_speed: 500, max_speed: 5000, percentage: 40.0 };
    let fan_clone = fan.clone();

    assert_eq!(fan.speed_rpm, fan_clone.speed_rpm);
    assert_eq!(fan.min_speed, fan_clone.min_speed);
    assert_eq!(fan.max_speed, fan_clone.max_speed);
    assert_eq!(fan.percentage, fan_clone.percentage);
}

#[test]
fn test_thermal_info_clone() {
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

    assert!(debug_str.contains("cpu_temp: 45.0"));
    assert!(debug_str.contains("gpu_temp: 55.0"));
    assert!(debug_str.contains("heatsink_temp: Some(40.0)"));
    assert!(debug_str.contains("ambient_temp: Some(25.0)"));
    assert!(debug_str.contains("battery_temp: Some(35.0)"));
    assert!(debug_str.contains("is_throttling: false"));
    assert!(debug_str.contains("cpu_power: Some(15.0)"));
}

#[test]
fn test_iokit_impl_default() {
    let iokit = IOKitImpl;
    assert!(matches!(iokit, IOKitImpl));
}

#[test]
fn test_iokit_impl_debug() {
    let iokit = IOKitImpl;
    let debug_str = format!("{:?}", iokit);
    assert!(debug_str.contains("IOKitImpl"));
} 