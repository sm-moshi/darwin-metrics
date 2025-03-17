mod health;
mod io;
mod mount;
mod performance;
mod storage;
mod utilization;

pub use health::*;
pub use io::*;
pub use mount::*;
pub use performance::*;
pub use storage::*;
pub use utilization::*;

use darwin_metrics::disk::{DiskMonitor, DiskStats, DiskType};
use std::time::{Duration, Instant};

#[test]
fn test_disk_monitor_new() {
    let monitor = DiskMonitor::new();
    assert!(monitor.previous_stats.is_empty());
    assert!(monitor.last_update <= Instant::now());
}

#[test]
fn test_disk_stats_default() {
    let stats = DiskStats::default();
    assert_eq!(stats.read_ops, 0);
    assert_eq!(stats.write_ops, 0);
    assert_eq!(stats.bytes_read, 0);
    assert_eq!(stats.bytes_written, 0);
    assert_eq!(stats.read_time_ns, 0);
    assert_eq!(stats.write_time_ns, 0);
    assert!(stats.timestamp <= Instant::now());
}

#[test]
fn test_detect_disk_type() {
    let monitor = DiskMonitor::new();

    // Network mounts
    assert_eq!(monitor.detect_disk_type("//server/share").unwrap(), DiskType::Network);
    assert_eq!(monitor.detect_disk_type("nfs://server/share").unwrap(), DiskType::Network);
    assert_eq!(monitor.detect_disk_type("smb://server/share").unwrap(), DiskType::Network);
    assert_eq!(monitor.detect_disk_type("afp://server/share").unwrap(), DiskType::Network);
}

#[test]
fn test_disk_monitor() {
    let mut monitor = DiskMonitor::new();
    let volumes = monitor.get_volumes().unwrap();
    assert!(!volumes.is_empty());

    for volume in &volumes {
        assert!(!volume.device.is_empty());
        assert!(!volume.mount_point.is_empty());
        assert!(!volume.fs_type.is_empty());
        assert!(volume.total > 0);
        assert!(volume.available <= volume.total);
        assert!(volume.used <= volume.total);
    }

    // Test getting volume by path
    let root = monitor.get_volume_for_path("/").unwrap();
    assert_eq!(root.mount_point, "/");
}

#[test]
fn test_disk_performance() {
    let mut monitor = DiskMonitor::new();
    monitor.update().unwrap();

    // Sleep to allow for stats collection
    std::thread::sleep(Duration::from_millis(200));

    let performance = monitor.get_performance().unwrap();
    assert!(!performance.is_empty());

    for (device, stats) in performance {
        assert!(!device.is_empty());
        assert!(stats.reads_per_second >= 0.0);
        assert!(stats.writes_per_second >= 0.0);
        assert!(stats.utilization >= 0.0 && stats.utilization <= 100.0);
    }
}

#[test]
fn test_get_performance_initial_call() {
    let mut monitor = DiskMonitor::new();
    let performance = monitor.get_performance().unwrap();

    // Should have placeholder data for at least /dev/disk0
    assert!(!performance.is_empty());
    assert!(performance.contains_key("/dev/disk0"));

    // Previous stats should be updated
    assert!(!monitor.previous_stats.is_empty());
}

#[test]
fn test_performance_metrics_calculation() {
    let mut monitor = DiskMonitor::new();
    monitor.update().unwrap();

    std::thread::sleep(Duration::from_millis(100));

    let performance = monitor.get_performance().unwrap();
    for (_, stats) in performance {
        assert!(stats.reads_per_second >= 0.0);
        assert!(stats.writes_per_second >= 0.0);
        assert!(stats.utilization >= 0.0 && stats.utilization <= 100.0);
    }
}

#[test]
fn test_empty_performance_fallback() {
    let mut monitor = DiskMonitor::new();
    monitor.previous_stats.clear();

    let performance = monitor.get_performance().unwrap();
    assert!(!performance.is_empty());

    for (_, stats) in performance {
        assert_eq!(stats.reads_per_second, 0.0);
        assert_eq!(stats.writes_per_second, 0.0);
        assert_eq!(stats.utilization, 0.0);
    }
}

#[test]
fn test_multiple_performance_updates() {
    let mut monitor = DiskMonitor::new();

    for _ in 0..3 {
        monitor.update().unwrap();
        std::thread::sleep(Duration::from_millis(100));

        let performance = monitor.get_performance().unwrap();
        assert!(!performance.is_empty());

        for (_, stats) in performance {
            assert!(stats.reads_per_second >= 0.0);
            assert!(stats.writes_per_second >= 0.0);
            assert!(stats.utilization >= 0.0 && stats.utilization <= 100.0);
        }
    }
}
