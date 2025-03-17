use super::super::TEST_MUTEX;
use crate::hardware::disk::{Disk, DiskUtilizationMonitor, HardwareMonitor, UtilizationMonitor};
use std::time::Duration;

#[tokio::test]
async fn test_utilization_monitor_creation() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(String::from("/dev/disk0"), String::from("/"), String::from("apfs"), 1000, 200, 800);
    let monitor = DiskUtilizationMonitor::new(disk);

    assert_eq!(monitor.name(), "Disk Utilization Monitor");
    assert_eq!(monitor.hardware_type(), "disk");
    assert_eq!(monitor.device_id(), "/dev/disk0");
}

#[tokio::test]
async fn test_utilization_measurement() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(String::from("/dev/disk0"), String::from("/"), String::from("apfs"), 1000, 200, 800);
    let monitor = DiskUtilizationMonitor::new(disk);

    let utilization = monitor.utilization().await.unwrap();
    assert!(utilization >= 0.0 && utilization <= 100.0, "Utilization should be between 0 and 100%");
}

#[tokio::test]
async fn test_utilization_monitor_with_nonexistent_disk() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(String::from("/dev/nonexistent"), String::from("/invalid"), String::from("invalid"), 0, 0, 0);
    let monitor = DiskUtilizationMonitor::new(disk);

    let utilization_result = monitor.utilization().await;
    assert!(utilization_result.is_ok(), "Should handle invalid disk gracefully");
    assert_eq!(utilization_result.unwrap(), 0.0, "Nonexistent disk should report 0% utilization");
}

#[tokio::test]
async fn test_utilization_monitor_updates() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(String::from("/dev/disk0"), String::from("/"), String::from("apfs"), 1000, 200, 800);
    let monitor = DiskUtilizationMonitor::new(disk);

    // Test multiple consecutive updates
    for _ in 0..3 {
        let utilization = monitor.utilization().await.unwrap();
        assert!(utilization >= 0.0 && utilization <= 100.0, "Utilization should always be between 0 and 100%");

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::test]
async fn test_utilization_monitor_stress() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(String::from("/dev/disk0"), String::from("/"), String::from("apfs"), 1000, 200, 800);
    let monitor = DiskUtilizationMonitor::new(disk);

    // Rapid consecutive measurements
    for _ in 0..10 {
        let utilization = monitor.utilization().await.unwrap();
        assert!(utilization >= 0.0 && utilization <= 100.0, "Utilization should remain valid under stress");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
