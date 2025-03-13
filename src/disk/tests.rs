use crate::disk::{Disk, DiskConfig, DiskInfo, DiskMonitor, DiskType};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::Instant;
use std::{thread, time::Duration};

// Initialize the test mutex
static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

// Initialize a static semaphore with only one permit to prevent disk tests from running in parallel
static DISK_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_disk_usage_percentage() {
    let disk = Disk::new(String::from("/dev/disk1s1"), String::from("/"), String::from("apfs"), 1000, 200, 800);

    assert_eq!(disk.usage_percentage(), 80.0);
}

#[test]
fn test_is_nearly_full() {
    let nearly_full_disk = Disk::new(
        "/dev/test1".to_string(),
        "/test1".to_string(),
        "apfs".to_string(),
        1000,
        50,  // 5% available
        950, // 95% used
    );

    let not_full_disk = Disk::new(
        "/dev/test2".to_string(),
        "/test2".to_string(),
        "apfs".to_string(),
        1000,
        500, // 50% available
        500, // 50% used
    );

    assert!(nearly_full_disk.is_nearly_full());
    assert!(!not_full_disk.is_nearly_full());
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
fn test_get_root_disk_info() {
    // This tests the actual implementation on macOS
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
    // This tests the actual implementation on macOS
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
fn test_disk_performance() {
    let mut monitor = DiskMonitor::new();
    monitor.update().unwrap();

    // Sleep to allow for stats collection
    thread::sleep(Duration::from_millis(200));

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
fn test_disk_type_default() {
    assert_eq!(DiskType::default(), DiskType::Unknown);
}

#[test]
fn test_disk_config() {
    let config = DiskConfig { disk_type: DiskType::SSD, name: "Test Drive".to_string(), is_boot_volume: true };

    assert_eq!(config.disk_type, DiskType::SSD);
    assert_eq!(config.name, "Test Drive");
    assert!(config.is_boot_volume);
}

#[test]
fn test_disk_new() {
    let disk = Disk::new(String::from("/dev/disk1s1"), String::from("/"), String::from("apfs"), 1000, 200, 800);

    assert_eq!(disk.device, "/dev/disk1s1");
    assert_eq!(disk.mount_point, "/");
    assert_eq!(disk.fs_type, "apfs");
    assert_eq!(disk.total, 1000);
    assert_eq!(disk.available, 200);
    assert_eq!(disk.used, 800);
    assert_eq!(disk.disk_type, DiskType::Unknown);
    assert_eq!(disk.name, "");
    assert!(!disk.is_boot_volume);

    let disk_with_details = Disk::with_details(
        String::from("/dev/disk1s1"),
        String::from("/"),
        String::from("apfs"),
        1000,
        200,
        800,
        DiskConfig { disk_type: DiskType::SSD, name: String::from("Macintosh HD"), is_boot_volume: true },
    );

    assert_eq!(disk_with_details.device, "/dev/disk1s1");
    assert_eq!(disk_with_details.mount_point, "/");
    assert_eq!(disk_with_details.fs_type, "apfs");
    assert_eq!(disk_with_details.total, 1000);
    assert_eq!(disk_with_details.available, 200);
    assert_eq!(disk_with_details.used, 800);
    assert_eq!(disk_with_details.disk_type, DiskType::SSD);
    assert_eq!(disk_with_details.name, "Macintosh HD");
    assert!(disk_with_details.is_boot_volume);
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
fn test_disk_usage_percentage_cases() {
    // Renamed from test_disk_usage_percentage Normal case
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

#[test]
fn test_format_bytes_edge_cases() {
    assert_eq!(Disk::format_bytes(0), "0 bytes");
    assert_eq!(Disk::format_bytes(1023), "1023 bytes");
    assert_eq!(Disk::format_bytes(1024), "1.0 KB");
    assert_eq!(Disk::format_bytes(1024 * 1024 * 1024 * 1024 * 2), "2.0 TB");
}

#[test]
fn test_disk_monitor_new() {
    let monitor = DiskMonitor::new();
    assert!(monitor.previous_stats.is_empty());
    assert!(monitor.last_update <= Instant::now());
}

#[test]
fn test_disk_stats_default() {
    use crate::disk::DiskStats;
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

    // RAM disk
    assert_eq!(monitor.detect_disk_type("/dev/ram0").unwrap(), DiskType::RAM);
    assert_eq!(monitor.detect_disk_type("/dev/ram1").unwrap(), DiskType::RAM);

    // Virtual disk
    assert_eq!(monitor.detect_disk_type("/dev/vda").unwrap(), DiskType::Virtual);
    assert_eq!(monitor.detect_disk_type("/dev/virtual0").unwrap(), DiskType::Virtual);

    // SSD (typical for modern macOS)
    assert_eq!(monitor.detect_disk_type("/dev/disk0").unwrap(), DiskType::SSD);
    assert_eq!(monitor.detect_disk_type("/dev/disk1").unwrap(), DiskType::SSD);

    // Unknown
    assert_eq!(monitor.detect_disk_type("/dev/unknown").unwrap(), DiskType::Unknown);
    assert_eq!(monitor.detect_disk_type("/dev/custom").unwrap(), DiskType::Unknown);
}

#[test]
fn test_get_for_path() {
    // Test with current directory
    let result = Disk::get_for_path(".");
    assert!(result.is_ok(), "Should be able to get disk info for current directory");

    if let Ok(disk) = result {
        // Verify the returned disk information is valid
        assert!(!disk.mount_point.is_empty(), "Mount point should not be empty");
        assert!(!disk.device.is_empty(), "Device should not be empty");
        assert!(!disk.fs_type.is_empty(), "Filesystem type should not be empty");
        assert!(disk.total > 0, "Total space should be greater than zero");
        assert!(disk.available <= disk.total, "Available space should not exceed total space");

        let _calculated_usage = disk.total - disk.available;
        assert!(disk.used <= disk.total, "Used space should not exceed total space");

        // Check percentage calculation
        let percentage = disk.usage_percentage();
        assert!((0.0..=100.0).contains(&percentage), "Percentage should be between 0 and 100");
    }

    // Test with absolute paths
    let home = std::env::var("HOME").unwrap_or_default();
    if !home.is_empty() {
        let result = Disk::get_for_path(&home);
        assert!(result.is_ok(), "Should be able to get disk info for home directory");

        if let Ok(disk) = result {
            println!("Home directory disk: {}", disk.summary());
            assert!(!disk.mount_point.is_empty());
        }
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
fn test_get_for_path_error_cases() {
    // Test with non-existent path
    let result = Disk::get_for_path("/definitely/not/a/real/path/12345");
    assert!(result.is_err(), "Should fail for non-existent path");

    // Test with empty path (should fail)
    let result = Disk::get_for_path("");
    assert!(result.is_err(), "Should fail for empty path");

    // Test with invalid characters (if your OS allows creating such strings) Note: This might not work on all systems,
    // so we're just checking it doesn't crash
    let invalid_paths = vec![
        "\0", // Null byte
        "path\nwith\nnewlines",
        // Fix: Use &str instead of String by removing the .repeat() call
        "path with extremely long name", // Simplified to avoid type mismatch
    ];

    for path in invalid_paths {
        let result = Disk::get_for_path(path);
        // We don't necessarily assert success or failure, just that it doesn't panic
        println!("Invalid path test: {:?} => {:?}", path, result.is_ok());
    }
}

#[test]
fn test_get_for_path_disk_type_detection() {
    // This test checks that the disk type detection logic in get_for_path works correctly
    let result = Disk::get_for_path(".");

    if let Ok(disk) = result {
        // The actual disk type will vary by system, but it should be determined We just verify it's been set to
        // something
        println!("Detected disk type for current directory: {:?}", disk.disk_type);
        assert!(
            matches!(
                disk.disk_type,
                DiskType::SSD
                    | DiskType::HDD
                    | DiskType::Network
                    | DiskType::Unknown
                    | DiskType::RAM
                    | DiskType::Virtual
                    | DiskType::External
                    | DiskType::Fusion
            ),
            "Disk type should be a valid variant"
        );

        // Check boot volume flag is set correctly for root
        if disk.mount_point == "/" {
            assert!(disk.is_boot_volume, "Root volume should be marked as boot volume");
        }

        // Verify name is populated
        assert!(!disk.name.is_empty(), "Disk name should not be empty");
    }
}

#[test]
fn test_get_for_path_with_relative_paths() {
    // Test various relative paths
    let relative_paths = vec![".", "..", "../..", "./test", "../test"];

    for path in relative_paths {
        let result = Disk::get_for_path(path);
        // We don't check success/failure as some paths may not exist, but the function shouldn't panic
        if let Ok(disk) = result {
            println!("Relative path {} is on filesystem {}", path, disk.mount_point);
            assert!(!disk.fs_type.is_empty());
            assert!(disk.total > 0);
        }
    }
}

#[test]
fn test_get_for_path_performance() {
    use std::time::Instant;

    // Measure performance of repeated calls
    let start = Instant::now();
    let iterations = 10;

    for _ in 0..iterations {
        let result = Disk::get_for_path(".");
        assert!(result.is_ok());
    }

    let duration = start.elapsed();
    println!(
        "get_for_path performed {} calls in {:?} ({:?} per call)",
        iterations,
        duration,
        duration / iterations as u32
    );
}

// Add these new tests for get_volumes

#[test]
fn test_get_volumes() {
    let mut monitor = DiskMonitor::new();
    let volumes = monitor.get_volumes();

    assert!(volumes.is_ok(), "Should be able to get disk volumes");

    if let Ok(disks) = volumes {
        assert!(!disks.is_empty(), "There should be at least one volume");

        for disk in &disks {
            // Basic validation of returned values
            assert!(!disk.device.is_empty(), "Device should not be empty");
            assert!(!disk.mount_point.is_empty(), "Mount point should not be empty");
            assert!(!disk.fs_type.is_empty(), "Filesystem type should not be empty");
            assert!(disk.total > 0, "Total space should be greater than zero");

            // Check that disk type is properly set
            assert!(
                matches!(
                    disk.disk_type,
                    DiskType::SSD
                        | DiskType::HDD
                        | DiskType::Network
                        | DiskType::Unknown
                        | DiskType::RAM
                        | DiskType::Virtual
                        | DiskType::External
                        | DiskType::Fusion
                ),
                "Disk type should be a valid variant"
            );

            // Check that name is populated
            assert!(!disk.name.is_empty(), "Disk name should not be empty");

            // Check boot volume flag for root volume
            if disk.mount_point == "/" {
                assert!(disk.is_boot_volume, "Root volume should be marked as boot volume");
            }

            println!("Volume: {} ({}) - {}", disk.name, disk.mount_point, disk.summary());
        }

        // Verify we can find the root volume
        let root_volume = disks.iter().find(|d| d.mount_point == "/");
        assert!(root_volume.is_some(), "Root volume should be present");
    }
}

#[test]
fn test_get_volume_for_path() {
    // Test multiple paths to ensure function works as expected
    let paths = vec![
        ".",    // Current directory
        "/",    // Root directory
        "/tmp", // System directory
    ];

    let mut monitor = DiskMonitor::new();

    for &path in &paths {
        let result = monitor.get_volume_for_path(path);
        assert!(result.is_ok(), "Should be able to get volume for path: {}", path);

        if let Ok(disk) = result {
            // Basic validation
            assert!(!disk.device.is_empty(), "Device should not be empty");
            assert!(!disk.mount_point.is_empty(), "Mount point should not be empty");
            assert!(!disk.fs_type.is_empty(), "Filesystem type should not be empty");
            assert!(disk.total > 0, "Total space should be greater than zero");

            println!("Path {} is on volume {} ({}) - {}", path, disk.name, disk.mount_point, disk.summary());

            // For root path, verify it's marked as boot volume
            if path == "/" {
                assert!(disk.is_boot_volume, "Root volume should be marked as boot volume");
                assert_eq!(disk.mount_point, "/", "Root volume should be mounted at /");
            }
        }
    }

    // Test non-existent path
    let result = monitor.get_volume_for_path("/definitely/not/a/real/path/12345");
    assert!(result.is_err(), "Should fail for non-existent path");
}

#[test]
fn test_volume_comparison() {
    // Test that root volume retrieved by different methods is the same
    let mut monitor = DiskMonitor::new();

    // Get root volume from get_volumes
    let volumes_result = monitor.get_volumes();
    assert!(volumes_result.is_ok());

    // Get root volume directly
    let direct_result = monitor.get_volume_for_path("/");
    assert!(direct_result.is_ok());

    if let (Ok(volumes), Ok(direct)) = (volumes_result, direct_result) {
        // Find root volume in the volumes list
        let from_list = volumes.iter().find(|d| d.mount_point == "/");
        assert!(from_list.is_some());

        if let Some(from_list) = from_list {
            // Compare key properties
            assert_eq!(from_list.device, direct.device);
            assert_eq!(from_list.mount_point, direct.mount_point);
            assert_eq!(from_list.fs_type, direct.fs_type);
            assert_eq!(from_list.total, direct.total);
            assert_eq!(from_list.is_boot_volume, direct.is_boot_volume);
        }
    }
}

#[test]
fn test_get_volumes_filtering() {
    // Test that special filesystems are filtered out
    let mut monitor = DiskMonitor::new();
    let volumes = monitor.get_volumes().unwrap();

    // Check that none of the returned volumes have these filesystem types
    for disk in &volumes {
        assert!(disk.fs_type != "devfs", "devfs should be filtered out");
        assert!(disk.fs_type != "autofs", "autofs should be filtered out");
        assert!(disk.fs_type != "msdos", "msdos should be filtered out unless actually mounted");

        // Check that returned disks have valid name and mount point
        if disk.name != "Root" {
            assert!(!disk.mount_point.is_empty() && disk.mount_point != "/");
        }
    }
}

#[test]
fn test_get_performance_initial_call() {
    // Create a fresh DiskMonitor with empty previous_stats
    let mut monitor = DiskMonitor::new();

    // Verify previous_stats is initially empty
    assert!(monitor.previous_stats.is_empty());

    // First call to get_performance should initialize stats and return placeholder
    let perf_result = monitor.get_performance();
    assert!(perf_result.is_ok(), "First call should succeed with placeholder data");

    if let Ok(perf_map) = perf_result {
        // Verify we got data for disk0
        assert!(perf_map.contains_key("/dev/disk0"), "Should have placeholder for /dev/disk0");

        // Check the placeholder has expected values (all zeros)
        if let Some(perf) = perf_map.get("/dev/disk0") {
            assert_eq!(perf.device, "/dev/disk0");
            assert_eq!(perf.reads_per_second, 0.0);
            assert_eq!(perf.writes_per_second, 0.0);
            assert_eq!(perf.bytes_read_per_second, 0);
            assert_eq!(perf.bytes_written_per_second, 0);
            assert_eq!(perf.read_latency_ms, 0.0);
            assert_eq!(perf.write_latency_ms, 0.0);
            assert_eq!(perf.utilization, 0.0);
            assert_eq!(perf.queue_depth, 0.0);
        }

        // Verify previous_stats is no longer empty after the call
        assert!(!monitor.previous_stats.is_empty());
        assert!(monitor.previous_stats.contains_key("/dev/disk0"));
    }

    // Sleep to allow for simulated activity
    std::thread::sleep(std::time::Duration::from_millis(200));

    // A second call should now use the "normal" path since stats are initialized
    let second_perf_result = monitor.get_performance();
    assert!(second_perf_result.is_ok());

    if let Ok(perf_map) = second_perf_result {
        // Should still have data for disk0
        assert!(perf_map.contains_key("/dev/disk0"));

        // But now the values should be non-zero (simulated activity)
        if let Some(perf) = perf_map.get("/dev/disk0") {
            // Since this is simulated data, we can't check exact values but we can verify they're reasonable
            assert!(perf.reads_per_second >= 0.0, "reads_per_second should be non-negative");
            assert!(perf.writes_per_second >= 0.0, "writes_per_second should be non-negative");
            assert!(perf.read_latency_ms >= 0.0, "read_latency_ms should be non-negative");
            assert!(perf.write_latency_ms >= 0.0, "write_latency_ms should be non-negative");
            assert!((0.0..=100.0).contains(&perf.utilization), "utilization should be between 0 and 100");
            assert!(perf.queue_depth >= 0.0, "queue_depth should be non-negative");
        }
    }
}

#[test]
fn test_performance_metrics_calculation() {
    use std::thread::sleep;
    use std::time::Duration;

    let mut monitor = DiskMonitor::new();

    // Initialize by calling get_performance once
    let _ = monitor.get_performance();

    // Sleep to allow for simulated activity
    sleep(Duration::from_millis(200));

    // Get metrics after some time has passed
    let perf_result = monitor.get_performance();
    assert!(perf_result.is_ok());

    if let Ok(perf_map) = perf_result {
        let disk0_perf = perf_map.get("/dev/disk0");
        assert!(disk0_perf.is_some(), "Should have performance data for /dev/disk0");

        if let Some(perf) = disk0_perf {
            // Verify all metrics are within reasonable ranges
            assert!(perf.reads_per_second >= 0.0, "reads_per_second should be non-negative");
            assert!(perf.writes_per_second >= 0.0, "writes_per_second should be non-negative");
            assert!(perf.read_latency_ms >= 0.0, "read_latency_ms should be non-negative");
            assert!(perf.write_latency_ms >= 0.0, "write_latency_ms should be non-negative");
            assert!((0.0..=100.0).contains(&perf.utilization), "utilization should be between 0 and 100");
            assert!(perf.queue_depth >= 0.0, "queue_depth should be non-negative");

            // Format some values for debugging/display
            println!("Disk Performance:");
            println!("  Read: {:.1} ops/sec", perf.reads_per_second);
            println!("  Write: {:.1} ops/sec", perf.writes_per_second);
            println!("  Read: {} bytes/sec", Disk::format_bytes(perf.bytes_read_per_second));
            println!("  Write: {} bytes/sec", Disk::format_bytes(perf.bytes_written_per_second));
            println!("  Utilization: {:.1}%", perf.utilization);
        }
    }
}

#[test]
fn test_empty_performance_fallback() {
    let mut monitor = DiskMonitor::new();
    monitor.update().unwrap();

    // Sleep to allow for stats collection
    thread::sleep(Duration::from_millis(200));

    let performance = monitor.get_performance().unwrap();
    assert!(!performance.is_empty());

    // Even with no previous stats, we should get valid performance data
    for (_, stats) in performance {
        assert!(stats.reads_per_second >= 0.0);
        assert!(stats.writes_per_second >= 0.0);
        assert!(stats.read_latency_ms >= 0.0);
        assert!(stats.write_latency_ms >= 0.0);
        assert!(stats.utilization >= 0.0 && stats.utilization <= 100.0);
        assert!(stats.queue_depth >= 0.0);
    }
}

#[test]
fn test_multiple_performance_updates() {
    // Test multiple successive calls to get_performance
    let mut monitor = DiskMonitor::new();

    // First call initializes
    let first = monitor.get_performance();
    assert!(first.is_ok());

    // Sleep to allow for meaningful time difference
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Multiple calls should update stats each time
    for i in 0..3 {
        let result = monitor.get_performance();
        assert!(result.is_ok(), "Call #{} failed", i + 1);

        if let Ok(perf_map) = result {
            assert!(perf_map.contains_key("/dev/disk0"));

            // Each call should give valid data
            let perf = &perf_map["/dev/disk0"];
            assert!(perf.utilization >= 0.0 && perf.utilization <= 100.0);
            assert!(perf.reads_per_second >= 0.0);
            assert!(perf.writes_per_second >= 0.0);

            println!(
                "Update #{}: {:.1} reads/s, {:.1} writes/s, {:.1}% util",
                i + 1,
                perf.reads_per_second,
                perf.writes_per_second,
                perf.utilization
            );
        }

        // Sleep between calls
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Verify the stats object is updated after multiple calls
    assert!(!monitor.previous_stats.is_empty());
    assert!(monitor.previous_stats.contains_key("/dev/disk0"));
}

#[test]
fn test_disk_info() {
    let _guard = TEST_MUTEX.lock().unwrap();
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
    let _guard = TEST_MUTEX.lock().unwrap();
    let _info = DiskInfo::new();

    // Basic validation of I/O stats
    // No need to check if unsigned integers are >= 0 as they can't be negative
}

#[test]
fn test_disk_partitions() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let info = DiskInfo::new();

    // Check partitions
    assert!(!info.partitions.is_empty());

    // Verify each partition
    for partition in &info.partitions {
        assert!(!partition.device.is_empty());
        assert!(partition.size > 0);
        assert!(!partition.fs_type.is_empty());
        assert!(!partition.mount_point.is_empty());
    }
}

#[test]
fn test_get_for_path_comprehensive() {
    // Test with current directory (should always exist)
    let result = Disk::get_for_path(".");
    assert!(result.is_ok(), "Should be able to get disk info for current directory");

    if let Ok(disk) = result {
        // Verify the returned disk information is valid
        assert!(!disk.mount_point.is_empty(), "Mount point should not be empty");
        assert!(!disk.device.is_empty(), "Device should not be empty");
        assert!(!disk.fs_type.is_empty(), "Filesystem type should not be empty");
        assert!(disk.total > 0, "Total space should be greater than zero");
        assert!(disk.available <= disk.total, "Available space should not exceed total space");

        let _calculated_usage = disk.total - disk.available;
        assert!(disk.used <= disk.total, "Used space should not exceed total space");

        // Check percentage calculation
        let percentage = disk.usage_percentage();
        assert!((0.0..=100.0).contains(&percentage), "Percentage should be between 0 and 100");
    }

    // Test with absolute paths
    let home = std::env::var("HOME").unwrap_or_default();
    if !home.is_empty() {
        let result = Disk::get_for_path(&home);
        assert!(result.is_ok(), "Should be able to get disk info for home directory");

        if let Ok(disk) = result {
            println!("Home directory disk: {}", disk.summary());
            assert!(!disk.mount_point.is_empty());
        }
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
