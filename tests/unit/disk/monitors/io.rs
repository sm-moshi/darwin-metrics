use super::super::TEST_MUTEX;
use crate::hardware::disk::{ByteMetricsMonitor, Disk, DiskIOMonitor, HardwareMonitor, RateMonitor};
use std::time::Duration;

#[tokio::test]
async fn test_io_monitor_creation() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(String::from("/dev/disk0"), String::from("/"), String::from("apfs"), 1000, 200, 800);
    let monitor = DiskIOMonitor::new(disk);

    assert_eq!(monitor.name(), "Disk I/O Monitor");
    assert_eq!(monitor.hardware_type(), "disk");
    assert_eq!(monitor.device_id(), "/dev/disk0");
}

#[tokio::test]
async fn test_byte_metrics() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(String::from("/dev/disk0"), String::from("/"), String::from("apfs"), 1000, 200, 800);
    let monitor = DiskIOMonitor::new(disk);

    let bytes_read = monitor.bytes_read().await.unwrap();
    assert!(bytes_read.as_bytes() >= 0, "Bytes read should be non-negative");

    let bytes_written = monitor.bytes_written().await.unwrap();
    assert!(bytes_written.as_bytes() >= 0, "Bytes written should be non-negative");
}

#[tokio::test]
async fn test_io_rates() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(String::from("/dev/disk0"), String::from("/"), String::from("apfs"), 1000, 200, 800);
    let monitor = DiskIOMonitor::new(disk);

    // Initial rates should be zero or very small
    let initial_read_rate = monitor.read_rate().await.unwrap();
    let initial_write_rate = monitor.write_rate().await.unwrap();

    assert!(initial_read_rate >= 0.0, "Read rate should be non-negative");
    assert!(initial_write_rate >= 0.0, "Write rate should be non-negative");

    // Wait a bit and check rates again
    tokio::time::sleep(Duration::from_millis(100)).await;

    let read_rate = monitor.read_rate().await.unwrap();
    let write_rate = monitor.write_rate().await.unwrap();

    assert!(read_rate >= 0.0, "Read rate should be non-negative");
    assert!(write_rate >= 0.0, "Write rate should be non-negative");
}

#[tokio::test]
async fn test_io_monitor_with_nonexistent_disk() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(String::from("/dev/nonexistent"), String::from("/invalid"), String::from("invalid"), 0, 0, 0);
    let monitor = DiskIOMonitor::new(disk);

    // Test error handling for each method
    let bytes_read_result = monitor.bytes_read().await;
    assert!(bytes_read_result.is_ok(), "Should handle invalid disk gracefully");

    let bytes_written_result = monitor.bytes_written().await;
    assert!(bytes_written_result.is_ok(), "Should handle invalid disk gracefully");

    let read_rate_result = monitor.read_rate().await;
    assert!(read_rate_result.is_ok(), "Should handle invalid disk gracefully");

    let write_rate_result = monitor.write_rate().await;
    assert!(write_rate_result.is_ok(), "Should handle invalid disk gracefully");
}

#[tokio::test]
async fn test_io_monitor_updates() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(String::from("/dev/disk0"), String::from("/"), String::from("apfs"), 1000, 200, 800);
    let monitor = DiskIOMonitor::new(disk);

    // Test multiple consecutive updates
    let mut prev_bytes_read = monitor.bytes_read().await.unwrap();
    let mut prev_bytes_written = monitor.bytes_written().await.unwrap();

    for _ in 0..3 {
        tokio::time::sleep(Duration::from_millis(100)).await;

        let bytes_read = monitor.bytes_read().await.unwrap();
        let bytes_written = monitor.bytes_written().await.unwrap();
        let read_rate = monitor.read_rate().await.unwrap();
        let write_rate = monitor.write_rate().await.unwrap();

        assert!(bytes_read >= prev_bytes_read, "Bytes read should not decrease");
        assert!(bytes_written >= prev_bytes_written, "Bytes written should not decrease");
        assert!(read_rate >= 0.0, "Read rate should be non-negative");
        assert!(write_rate >= 0.0, "Write rate should be non-negative");

        prev_bytes_read = bytes_read;
        prev_bytes_written = bytes_written;
    }
}
