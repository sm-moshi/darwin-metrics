use std::time::Duration;

use super::super::TEST_MUTEX;
use crate::hardware::disk::{Disk, DiskHealthMonitor, DiskHealthMonitorTrait, HardwareMonitor};

#[tokio::test]
async fn test_health_monitor_creation() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(
        String::from("/dev/disk0"),
        String::from("/"),
        String::from("apfs"),
        1000,
        200,
        800,
    );
    let monitor = DiskHealthMonitor::new(disk);

    assert_eq!(monitor.name(), "Disk Health Monitor");
    assert_eq!(monitor.hardware_type(), "disk");
    assert_eq!(monitor.device_id(), "/dev/disk0");
}

#[tokio::test]
async fn test_health_status() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(
        String::from("/dev/disk0"),
        String::from("/"),
        String::from("apfs"),
        1000,
        200,
        800,
    );
    let monitor = DiskHealthMonitor::new(disk);

    let is_healthy = monitor.is_healthy().await.unwrap();
    assert!(is_healthy, "Disk should be healthy by default");

    let smart_status = monitor.smart_status().await.unwrap();
    assert!(!smart_status.is_empty(), "SMART status should not be empty");
}

#[tokio::test]
async fn test_temperature_monitoring() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(
        String::from("/dev/disk0"),
        String::from("/"),
        String::from("apfs"),
        1000,
        200,
        800,
    );
    let monitor = DiskHealthMonitor::new(disk);

    let temp = monitor.temperature().await.unwrap();
    assert!(temp >= 0.0, "Temperature should be non-negative");
    assert!(temp < 100.0, "Temperature should be less than 100°C");
}

#[tokio::test]
async fn test_health_monitor_error_handling() {
    let _guard = TEST_MUTEX.lock().unwrap();
    // Create a monitor with an invalid disk path
    let disk = Disk::new(
        String::from("/dev/nonexistent"),
        String::from("/invalid"),
        String::from("invalid"),
        0,
        0,
        0,
    );
    let monitor = DiskHealthMonitor::new(disk);

    // Test error handling for each method
    let health_result = monitor.is_healthy().await;
    assert!(health_result.is_ok(), "Should handle invalid disk gracefully");

    let smart_result = monitor.smart_status().await;
    assert!(smart_result.is_ok(), "Should handle invalid disk gracefully");

    let temp_result = monitor.temperature().await;
    assert!(temp_result.is_ok(), "Should handle invalid disk gracefully");
}

#[tokio::test]
async fn test_health_monitor_updates() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(
        String::from("/dev/disk0"),
        String::from("/"),
        String::from("apfs"),
        1000,
        200,
        800,
    );
    let monitor = DiskHealthMonitor::new(disk);

    // Test multiple consecutive updates
    for _ in 0..3 {
        let health = monitor.is_healthy().await.unwrap();
        let smart = monitor.smart_status().await.unwrap();
        let temp = monitor.temperature().await.unwrap();

        assert!(health, "Disk should remain healthy");
        assert!(!smart.is_empty(), "SMART status should never be empty");
        assert!(temp >= 0.0 && temp < 100.0, "Temperature should be in valid range");

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
