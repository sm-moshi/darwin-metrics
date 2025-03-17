use super::super::TEST_MUTEX;
use crate::hardware::disk::{Disk, DiskInfo};

#[test]
fn test_disk_usage_percentage() {
    let disk = Disk::new(String::from("/dev/disk1s1"), String::from("/"), String::from("apfs"), 1000, 200, 800);
    assert_eq!(disk.usage_percentage(), 80.0);
}

#[test]
fn test_disk_usage_percentage_cases() {
    // Normal case
    let disk = Disk::new("/dev/test".to_string(), "/test".to_string(), "apfs".to_string(), 1000, 750, 250);
    assert_eq!(disk.usage_percentage(), 25.0);

    // Edge case: empty disk
    let empty_disk = Disk::new("/dev/empty".to_string(), "/empty".to_string(), "apfs".to_string(), 0, 0, 0);
    assert_eq!(empty_disk.usage_percentage(), 0.0);

    // Full disk
    let full_disk = Disk::new("/dev/full".to_string(), "/full".to_string(), "apfs".to_string(), 1000, 0, 1000);
    assert_eq!(full_disk.usage_percentage(), 100.0);
}

#[test]
fn test_format_bytes() {
    assert_eq!(Disk::format_bytes(500), "500 bytes");
    assert_eq!(Disk::format_bytes(1024), "1.0 KB");
    assert_eq!(Disk::format_bytes(1536), "1.5 KB");
    assert_eq!(Disk::format_bytes(1_048_576), "1.0 MB");
    assert_eq!(Disk::format_bytes(1_073_741_824), "1.0 GB");
    assert_eq!(Disk::format_bytes(1_099_511_627_776), "1.0 TB");
}

#[test]
fn test_format_bytes_edge_cases() {
    assert_eq!(Disk::format_bytes(0), "0 bytes");
    assert_eq!(Disk::format_bytes(1023), "1023 bytes");
    assert_eq!(Disk::format_bytes(1024), "1.0 KB");
    assert_eq!(Disk::format_bytes(1024 * 1024 * 1024 * 1024 * 2), "2.0 TB");
}

#[test]
fn test_display_methods() {
    let disk = Disk::new(
        "/dev/test".to_string(),
        "/test".to_string(),
        "apfs".to_string(),
        1_073_741_824, // 1 GB
        536_870_912,   // 512 MB
        536_870_912,   // 512 MB
    );

    assert_eq!(disk.available_display(), "512.0 MB");
    assert_eq!(disk.total_display(), "1.0 GB");
    assert_eq!(disk.used_display(), "512.0 MB");

    let summary = disk.summary();
    assert!(summary.contains("50%")); // Usage percentage
    assert!(summary.contains("512.0 MB")); // Used space
    assert!(summary.contains("1.0 GB")); // Total space
}
