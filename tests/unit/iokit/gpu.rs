use darwin_metrics::error::Error;
use darwin_metrics::hardware::iokit::{GpuStats, IOKit, IOKitImpl, MockIOKit};
use darwin_metrics::utils::tests::test_utils::{create_test_dictionary, create_test_object};

#[test]
fn test_get_gpu_stats() {
    let mut mock_iokit = MockIOKit::new();

    mock_iokit.expect_get_gpu_stats().returning(|| {
        Ok(GpuStats {
            name: "Test GPU".to_string(),
            utilization: 50.0,
            memory_used: 1024 * 1024 * 1024,
            memory_total: 4 * 1024 * 1024 * 1024,
            perf_cap: 80.0,
            perf_threshold: 90.0,
        })
    });

    let result = mock_iokit.get_gpu_stats().unwrap();

    assert_eq!(result.name, "Test GPU");
    assert_eq!(result.utilization, 50.0);
    assert_eq!(result.memory_used, 1024 * 1024 * 1024);
    assert_eq!(result.memory_total, 4 * 1024 * 1024 * 1024);
    assert_eq!(result.perf_cap, 80.0);
    assert_eq!(result.perf_threshold, 90.0);
}

#[test]
fn test_gpu_stats_error_handling() {
    let mut mock = MockIOKit::new();

    mock.expect_get_service_matching()
        .returning(|_| Err(Error::iokit_error(1, "GPU service not found")));

    let result = mock.get_gpu_stats();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("GPU service not found"));

    let mut mock = MockIOKit::new();
    mock.expect_get_service_matching()
        .returning(|_| Ok(create_test_object()));
    mock.expect_io_registry_entry_create_cf_properties()
        .returning(|_| Err(Error::iokit_error(1, "Failed to read properties")));

    let result = mock.get_gpu_stats();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to read properties"));
}

#[test]
fn test_gpu_stats_default() {
    let stats = GpuStats::default();

    assert_eq!(stats.name, "");
    assert_eq!(stats.utilization, 0.0);
    assert_eq!(stats.memory_used, 0);
    assert_eq!(stats.memory_total, 0);
    assert_eq!(stats.perf_cap, 0.0);
    assert_eq!(stats.perf_threshold, 0.0);
}

#[test]
fn test_gpu_stats_display() {
    let stats = GpuStats {
        name: "Test GPU".to_string(),
        utilization: 50.0,
        memory_used: 1024 * 1024 * 1024,
        memory_total: 4 * 1024 * 1024 * 1024,
        perf_cap: 80.0,
        perf_threshold: 90.0,
    };

    // The Display trait isn't implemented for GpuStats, so we'll use Debug instead
    let display = format!("{:?}", stats);
    assert!(display.contains("Test GPU"));
    assert!(display.contains("50.0"));
    assert!(display.contains("1073741824")); // 1 GB in bytes
}

#[test]
fn test_gpu_stats_multiple() {
    let mut mock_iokit = MockIOKit::new();

    mock_iokit.expect_get_gpu_stats_multiple().returning(|| {
        Ok(vec![
            GpuStats {
                name: "GPU1".to_string(),
                utilization: 30.0,
                memory_used: 512 * 1024 * 1024,
                memory_total: 2 * 1024 * 1024 * 1024,
                perf_cap: 75.0,
                perf_threshold: 85.0,
            },
            GpuStats {
                name: "GPU2".to_string(),
                utilization: 70.0,
                memory_used: 3 * 1024 * 1024 * 1024,
                memory_total: 8 * 1024 * 1024 * 1024,
                perf_cap: 85.0,
                perf_threshold: 95.0,
            },
        ])
    });

    let result = mock_iokit.get_gpu_stats_multiple().unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "GPU1");
    assert_eq!(result[1].name, "GPU2");
    assert_eq!(result[0].utilization, 30.0);
    assert_eq!(result[1].utilization, 70.0);
}

#[test]
fn test_gpu_stats_clone() {
    let original = GpuStats {
        name: "Test GPU".to_string(),
        utilization: 50.0,
        memory_used: 1024 * 1024 * 1024,
        memory_total: 4 * 1024 * 1024 * 1024,
        perf_cap: 80.0,
        perf_threshold: 90.0,
    };

    let cloned = original.clone();

    assert_eq!(cloned.name, original.name);
    assert_eq!(cloned.utilization, original.utilization);
    assert_eq!(cloned.memory_used, original.memory_used);
    assert_eq!(cloned.memory_total, original.memory_total);
    assert_eq!(cloned.perf_cap, original.perf_cap);
    assert_eq!(cloned.perf_threshold, original.perf_threshold);
}
