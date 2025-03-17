use crate::hardware::disk::DiskMonitor;
use std::thread;
use std::time::Duration;

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
        }
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
        }

        // Sleep between calls
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Verify the stats object is updated after multiple calls
    assert!(!monitor.previous_stats.is_empty());
    assert!(monitor.previous_stats.contains_key("/dev/disk0"));
}
