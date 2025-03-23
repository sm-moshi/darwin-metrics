use super::super::TEST_MUTEX;
use crate::hardware::disk::{Disk, DiskMountMonitor, DiskMountMonitorTrait, HardwareMonitor};

#[tokio::test]
async fn test_mount_monitor_creation() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(
        String::from("/dev/disk0"),
        String::from("/"),
        String::from("apfs"),
        1000,
        200,
        800,
    );
    let monitor = DiskMountMonitor::new(disk);

    assert_eq!(monitor.name(), "Disk Mount Monitor");
    assert_eq!(monitor.hardware_type(), "disk");
    assert_eq!(monitor.device_id(), "/dev/disk0");
}

#[tokio::test]
async fn test_mount_status() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(
        String::from("/dev/disk0"),
        String::from("/"),
        String::from("apfs"),
        1000,
        200,
        800,
    );
    let monitor = DiskMountMonitor::new(disk);

    let is_mounted = monitor.is_mounted().await.unwrap();
    assert!(is_mounted, "Root disk should be mounted");

    let mount_point = monitor.mount_point().await.unwrap();
    assert_eq!(mount_point, "/", "Root disk should be mounted at /");

    let fs_type = monitor.filesystem_type().await.unwrap();
    assert_eq!(fs_type, "apfs", "Root disk should be APFS");

    let options = monitor.mount_options().await.unwrap();
    assert!(!options.is_empty(), "Mount options should not be empty");
}

#[tokio::test]
async fn test_mount_monitor_with_nonexistent_disk() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disk = Disk::new(
        String::from("/dev/nonexistent"),
        String::from("/invalid"),
        String::from("invalid"),
        0,
        0,
        0,
    );
    let monitor = DiskMountMonitor::new(disk);

    // Test error handling for each method
    let mounted_result = monitor.is_mounted().await;
    assert!(mounted_result.is_ok(), "Should handle invalid disk gracefully");
    assert!(!mounted_result.unwrap(), "Nonexistent disk should not be mounted");

    let mount_point_result = monitor.mount_point().await;
    assert!(mount_point_result.is_ok(), "Should handle invalid disk gracefully");
    assert_eq!(mount_point_result.unwrap(), "/invalid");

    let fs_type_result = monitor.filesystem_type().await;
    assert!(fs_type_result.is_ok(), "Should handle invalid disk gracefully");
    assert_eq!(fs_type_result.unwrap(), "invalid");

    let options_result = monitor.mount_options().await;
    assert!(options_result.is_ok(), "Should handle invalid disk gracefully");
}

#[tokio::test]
async fn test_mount_monitor_with_multiple_disks() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let disks = vec![
        Disk::new(
            String::from("/dev/disk0"),
            String::from("/"),
            String::from("apfs"),
            1000,
            200,
            800,
        ),
        Disk::new(
            String::from("/dev/disk1"),
            String::from("/home"),
            String::from("apfs"),
            2000,
            500,
            1500,
        ),
    ];

    for disk in disks {
        let monitor = DiskMountMonitor::new(disk);
        let is_mounted = monitor.is_mounted().await.unwrap();
        let mount_point = monitor.mount_point().await.unwrap();
        let fs_type = monitor.filesystem_type().await.unwrap();
        let options = monitor.mount_options().await.unwrap();

        assert!(is_mounted, "Disk should be mounted");
        assert!(!mount_point.is_empty(), "Mount point should not be empty");
        assert_eq!(fs_type, "apfs", "Filesystem should be APFS");
        assert!(!options.is_empty(), "Mount options should not be empty");
    }
}
