use std::time::Duration;

use super::super::TEST_MUTEX;
use crate::gpu::{Gpu, GpuTemperatureMonitor, HardwareMonitor, TemperatureMonitor};

#[tokio::test]
async fn test_temperature_monitor_creation() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuTemperatureMonitor::new(gpu);

    assert_eq!(monitor.name(), "GPU Temperature Monitor");
    assert_eq!(monitor.hardware_type(), "gpu");
    assert!(!monitor.device_id().is_empty());
}

#[tokio::test]
async fn test_temperature_metrics() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuTemperatureMonitor::new(gpu);

    let temp = monitor.temperature().await.unwrap();
    let is_critical = monitor.is_critical().await.unwrap();

    assert!(temp >= 0.0, "Temperature should be non-negative");
    assert!(temp < 150.0, "Temperature should be less than 150°C");

    println!("Temperature: {}°C", temp);
    println!("Critical: {}", is_critical);
}

#[tokio::test]
async fn test_temperature_info() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuTemperatureMonitor::new(gpu);

    let info = monitor.temperature_info().await.unwrap();

    assert!(info.core >= 0.0, "Core temperature should be non-negative");
    assert!(info.core < 150.0, "Core temperature should be less than 150°C");

    if let Some(mem_temp) = info.memory {
        assert!(mem_temp >= 0.0, "Memory temperature should be non-negative");
        assert!(mem_temp < 150.0, "Memory temperature should be less than 150°C");
    }

    if let Some(critical) = info.critical_temp {
        assert!(critical > 0.0, "Critical temperature should be positive");
        assert!(critical < 150.0, "Critical temperature should be less than 150°C");
    }

    println!("Temperature Info: {:?}", info);
}

#[tokio::test]
async fn test_temperature_thresholds() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuTemperatureMonitor::new(gpu);

    let temp = monitor.temperature().await.unwrap();
    let is_critical = monitor.is_critical().await.unwrap();
    let critical_temp = monitor.critical_temperature().await.unwrap();

    if let Some(threshold) = critical_temp {
        assert!(threshold > 0.0, "Critical threshold should be positive");
        assert!(
            is_critical == (temp >= threshold),
            "Critical state should match temperature threshold comparison"
        );
    }

    println!("Temperature: {}°C (Critical Threshold: {:?})", temp, critical_temp);
}

#[tokio::test]
async fn test_temperature_updates() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuTemperatureMonitor::new(gpu);

    let mut prev_temp = monitor.temperature().await.unwrap();

    // Test multiple consecutive updates
    for _ in 0..3 {
        tokio::time::sleep(Duration::from_millis(100)).await;

        let temp = monitor.temperature().await.unwrap();
        let is_critical = monitor.is_critical().await.unwrap();
        let mem_temp = monitor.memory_temperature().await.unwrap();

        assert!(temp >= 0.0, "Temperature should be non-negative");
        assert!(temp < 150.0, "Temperature should be less than 150°C");

        if let Some(mt) = mem_temp {
            assert!(mt >= 0.0, "Memory temperature should be non-negative");
            assert!(mt < 150.0, "Memory temperature should be less than 150°C");
        }

        // Temperature might change slightly between readings
        assert!(
            (temp - prev_temp).abs() < 20.0,
            "Temperature should not change drastically between readings"
        );
        prev_temp = temp;

        println!(
            "Update - Core: {}°C, Memory: {:?}°C, Critical: {}",
            temp, mem_temp, is_critical
        );
    }
}
