use darwin_metrics::hardware::iokit::{FanInfo, GpuStats, IOKitImpl, ThermalInfo};
use darwin_metrics::utils::SafeDictionary;

#[test]
fn test_debug_implementations() {
    let gpu_stats = GpuStats::default();
    let fan_info = FanInfo {
        speed_rpm: 1000,
        min_speed: 500,
        max_speed: 2000,
        percentage: 50.0,
    };

    let mut thermal_info = ThermalInfo::default();
    thermal_info.cpu_temp = 42.0;
    thermal_info.gpu_temp = Some(45.0);
    thermal_info.fan_speed = 1200;
    thermal_info.heatsink_temp = Some(40.0);
    thermal_info.ambient_temp = Some(25.0);
    thermal_info.battery_temp = Some(35.0);
    thermal_info.thermal_throttling = false;

    assert!(!format!("{:?}", gpu_stats).is_empty());
    assert!(!format!("{:?}", fan_info).is_empty());
    assert!(!format!("{:?}", thermal_info).is_empty());
    assert!(!format!("{:?}", IOKitImpl).is_empty());
}

#[test]
fn test_clone_implementations() {
    let gpu_stats = GpuStats {
        name: "Test GPU".to_string(),
        utilization: 50.0,
        memory_used: 1024,
        memory_total: 2048,
        perf_cap: 80.0,
        perf_threshold: 90.0,
    };
    let fan_info = FanInfo {
        speed_rpm: 1000,
        min_speed: 500,
        max_speed: 2000,
        percentage: 50.0,
    };

    let mut thermal_info = ThermalInfo::default();
    thermal_info.cpu_temp = 42.0;
    thermal_info.gpu_temp = Some(45.0);
    thermal_info.fan_speed = 1200;
    thermal_info.heatsink_temp = Some(40.0);
    thermal_info.ambient_temp = Some(25.0);
    thermal_info.battery_temp = Some(35.0);
    thermal_info.thermal_throttling = false;

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
        name: "Test GPU".to_string(),
        utilization: 50.0,
        memory_used: 1024 * 1024 * 1024,
        memory_total: 4 * 1024 * 1024 * 1024,
        perf_cap: 80.0,
        perf_threshold: 90.0,
    };

    let cloned = original.clone();

    assert_eq!(cloned.name, original.name);
    assert_eq!(cloned.utilization, original.utilization);
    assert_eq!(cloned.memory_used, original.memory_used);
    assert_eq!(cloned.memory_total, original.memory_total);
    assert_eq!(cloned.perf_cap, original.perf_cap);
    assert_eq!(cloned.perf_threshold, original.perf_threshold);
}

#[test]
fn test_fan_info_clone() {
    let fan = FanInfo {
        speed_rpm: 2000,
        min_speed: 500,
        max_speed: 5000,
        percentage: 40.0,
    };
    let fan_clone = fan.clone();

    assert_eq!(fan.speed_rpm, fan_clone.speed_rpm);
    assert_eq!(fan.min_speed, fan_clone.min_speed);
    assert_eq!(fan.max_speed, fan_clone.max_speed);
    assert_eq!(fan.percentage, fan_clone.percentage);
}

#[test]
fn test_thermal_info_clone() {
    let mut info = ThermalInfo::default();
    info.cpu_temp = 45.0;
    info.gpu_temp = Some(55.0);
    info.fan_speed = 1200;
    info.heatsink_temp = Some(40.0);
    info.ambient_temp = Some(25.0);
    info.battery_temp = Some(35.0);
    info.thermal_throttling = false;

    let info_clone = info.clone();

    assert_eq!(info.cpu_temp, info_clone.cpu_temp);
    assert_eq!(info.gpu_temp, info_clone.gpu_temp);
    assert_eq!(info.fan_speed, info_clone.fan_speed);
    assert_eq!(info.heatsink_temp, info_clone.heatsink_temp);
    assert_eq!(info.ambient_temp, info_clone.ambient_temp);
    assert_eq!(info.battery_temp, info_clone.battery_temp);
    assert_eq!(info.thermal_throttling, info_clone.thermal_throttling);
}

#[test]
fn test_thermal_info_debug() {
    let mut info = ThermalInfo::default();
    info.cpu_temp = 45.0;
    info.gpu_temp = Some(55.0);
    info.fan_speed = 1200;
    info.heatsink_temp = Some(40.0);
    info.ambient_temp = Some(25.0);
    info.battery_temp = Some(35.0);
    info.thermal_throttling = false;

    let debug_str = format!("{:?}", info);

    assert!(debug_str.contains("cpu_temp: 45.0"));
    assert!(debug_str.contains("gpu_temp: Some(55.0)"));
    assert!(debug_str.contains("fan_speed: 1200"));
    assert!(debug_str.contains("heatsink_temp: Some(40.0)"));
    assert!(debug_str.contains("ambient_temp: Some(25.0)"));
    assert!(debug_str.contains("battery_temp: Some(35.0)"));
    assert!(debug_str.contains("thermal_throttling: false"));
}

#[test]
fn test_iokit_debug() {
    let iokit = IOKitImpl::default();
    assert!(!format!("{:?}", iokit).is_empty());
}

#[test]
fn test_iokit_clone() {
    let iokit = IOKitImpl::default();
    let iokit_clone = iokit.clone();
    assert!(matches!(iokit_clone, IOKitImpl));
}

#[test]
fn test_iokit_debug_format() {
    let iokit = IOKitImpl::default();
    let debug_str = format!("{:?}", iokit);
    assert!(debug_str.contains("IOKitImpl"));
}
