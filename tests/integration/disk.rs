use darwin_metrics::disk::{Disk, DiskInfo, DiskMonitor, DiskType};

#[test]
fn test_get_root_disk_info() {
    let disk = Disk::get_info();
    assert!(disk.is_ok(), "Should be able to get root disk info");

    if let Ok(disk) = disk {
        assert_eq!(disk.mount_point, "/", "Root disk should be mounted at /");
        assert!(disk.is_boot_volume, "Root disk should be the boot volume");
        assert!(disk.total > 0, "Total space should be > 0");
        assert!(disk.available > 0, "Available space should be > 0");
        assert!(disk.used > 0, "Used space should be > 0");
        println!("Root disk: {}", disk.summary());
    }
}

#[test]
fn test_get_all_volumes() {
    let disks = Disk::get_all();
    assert!(disks.is_ok(), "Should be able to get all disk volumes");

    if let Ok(disks) = disks {
        assert!(!disks.is_empty(), "There should be at least one volume");

        // Find the root volume
        let root = disks.iter().find(|d| d.mount_point == "/");
        assert!(root.is_some(), "Root volume should be present");

        for disk in disks {
            println!("Volume: {} ({})", disk.mount_point, disk.summary());
        }
    }
}

#[test]
fn test_get_for_path() {
    // Test with current directory
    let result = Disk::get_for_path(".");
    assert!(result.is_ok(), "Should be able to get disk info for current directory");

    if let Ok(disk) = result {
        assert!(!disk.mount_point.is_empty(), "Mount point should not be empty");
        assert!(!disk.device.is_empty(), "Device should not be empty");
        assert!(!disk.fs_type.is_empty(), "Filesystem type should not be empty");
        assert!(disk.total > 0, "Total space should be greater than zero");
        assert!(disk.available <= disk.total, "Available space should not exceed total space");
        assert!(disk.used <= disk.total, "Used space should not exceed total space");
    }

    // Test with system paths
    let system_paths = vec!["/", "/tmp", "/var"];
    for path in system_paths {
        let result = Disk::get_for_path(path);
        assert!(result.is_ok(), "Should be able to get disk info for {}", path);

        if let Ok(disk) = result {
            println!("Path {} is on {} ({})", path, disk.mount_point, disk.device);
            assert!(disk.total > 0, "Total space should be greater than zero for {}", path);
        }
    }
}

#[test]
fn test_disk_info() {
    let info = DiskInfo::new();

    // Basic validation
    assert!(info.total_space > 0);
    assert!(info.free_space <= info.total_space);
    assert!(info.available_space <= info.total_space);

    // Check mount points
    assert!(!info.mount_points.is_empty());

    // Verify each mount point
    for mount in &info.mount_points {
        assert!(!mount.device.is_empty());
        assert!(!mount.path.is_empty());
        assert!(!mount.fs_type.is_empty());
        assert!(mount.total_space > 0);
        assert!(mount.free_space <= mount.total_space);
        assert!(mount.available_space <= mount.total_space);
    }
}

#[test]
fn test_disk_io_stats() {
    let mut monitor = DiskMonitor::new();
    monitor.update().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let stats = monitor.get_performance().unwrap();
    assert!(!stats.is_empty());
}

#[test]
fn test_disk_partitions() {
    let disks = Disk::get_all().unwrap();
    assert!(!disks.is_empty());

    for disk in disks {
        assert!(!disk.device.is_empty());
        assert!(!disk.mount_point.is_empty());
        assert!(!disk.fs_type.is_empty());
    }
}

#[test]
fn test_get_for_path_comprehensive() {
    let paths = vec!["/", "/tmp", "/var", "/Users", "."];
    for path in paths {
        let disk = Disk::get_for_path(path).unwrap();
        assert!(!disk.device.is_empty());
        assert!(!disk.mount_point.is_empty());
        assert!(!disk.fs_type.is_empty());
        assert!(disk.total > 0);
        assert!(disk.available <= disk.total);
        assert!(disk.used <= disk.total);
    }
}
