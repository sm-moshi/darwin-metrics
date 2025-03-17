use crate::common::builders::gpu::create_test_gpu;
use tokio::test;

#[test]
async fn test_gpu_initialization() {
    let gpu = create_test_gpu().await;
    assert!(gpu.name().await.is_ok(), "Should be able to get GPU name");
}

#[test]
async fn test_gpu_name() {
    let gpu = create_test_gpu().await;
    let name = gpu.name().await;

    assert!(name.is_ok(), "Should be able to get GPU name");
    let name = name.unwrap();
    assert!(!name.is_empty(), "GPU name should not be empty");

    println!("GPU name: {}", name);
}

#[test]
async fn test_memory_info() {
    let gpu = create_test_gpu().await;
    let memory = gpu.get_memory().await;

    assert!(memory.is_ok(), "Should be able to get memory info");
    let memory = memory.unwrap();

    assert!(memory.total > 0, "Total memory should be positive");
    assert!(memory.used <= memory.total, "Used memory should not exceed total");
    assert_eq!(memory.free, memory.total.saturating_sub(memory.used), "Free memory should be calculated correctly");

    println!("Memory: {:?}", memory);
}

#[test]
async fn test_metrics() {
    let gpu = create_test_gpu().await;
    let metrics = gpu.get_metric().await;

    assert!(metrics.is_ok(), "Should be able to get metrics");
    let metrics = metrics.unwrap();

    assert!(!metrics.value.name.is_empty(), "Name should not be empty");
    assert!(metrics.value.utilization >= 0.0 && metrics.value.utilization <= 100.0, "Utilization should be between 0-100%");

    println!("Metrics: {:?}", metrics);
}

#[test]
async fn test_gpu_characteristics() {
    let gpu = create_test_gpu().await;
    let characteristics = gpu.get_characteristics();

    if cfg!(target_arch = "aarch64") {
        assert!(characteristics.is_apple_silicon, "Should detect Apple Silicon on aarch64 hardware");
        assert!(characteristics.is_integrated, "Apple Silicon GPUs should be detected as integrated");
    }

    if characteristics.is_apple_silicon && characteristics.has_raytracing {
        if let Some(chip_info) = gpu.detect_apple_silicon_chip().await {
            assert!(
                chip_info.contains("M2") || chip_info.contains("M3"),
                "Raytracing should only be reported on M2/M3 chips, not on: {}",
                chip_info
            );
        }
    }

    if let Some(clock_speed) = characteristics.clock_speed_mhz {
        assert!(
            clock_speed > 500 && clock_speed < 3000,
            "Clock speed should be in a reasonable range: {}",
            clock_speed
        );
    }

    if let Some(core_count) = characteristics.core_count {
        assert!(core_count > 0 && core_count < 200, "Core count should be in a reasonable range: {}", core_count);
    }

    println!("Characteristics: {:?}", characteristics);
}

#[test]
async fn test_apple_silicon_detection() {
    let gpu = create_test_gpu().await;

    if cfg!(target_arch = "aarch64") {
        assert!(gpu.detect_apple_silicon_chip().await.is_some(), "Should detect chip type on Apple Silicon hardware");

        if let Some(chip_info) = gpu.detect_apple_silicon_chip().await {
            assert!(
                chip_info.contains("M1")
                    || chip_info.contains("M2")
                    || chip_info.contains("M3")
                    || chip_info.contains("Apple Silicon GPU"),
                "Should identify an M-series chip on Apple Silicon: {}",
                chip_info
            );
        }
    }
}

#[test]
async fn test_cpu_detection() {
    let gpu = create_test_gpu().await;
    let cpu_info = gpu.get_cpu_model().await;

    assert!(cpu_info.is_some(), "Should retrieve CPU model information");

    let cpu_info_clone = cpu_info.clone();

    if let Some(info) = cpu_info {
        assert!(!info.is_empty(), "CPU model info should not be empty");

        if cfg!(target_arch = "aarch64") {
            assert!(
                info.contains("Apple") || info.contains("M1") || info.contains("M2") || info.contains("M3"),
                "On Apple Silicon, CPU should be an Apple-designed chip: {}",
                info
            );
        } else {
            assert!(info.contains("Intel"), "On Intel Macs, CPU should be an Intel chip: {}", info);
        }
    }

    println!("CPU Model: {:?}", cpu_info_clone);
}

#[test]
async fn test_metrics_characteristics() {
    let gpu = create_test_gpu().await;
    let metrics = gpu.get_metric().await.unwrap();

    if cfg!(target_arch = "aarch64") {
        assert!(metrics.value.characteristics.is_apple_silicon, "Metrics should report Apple Silicon on ARM hardware");
    }

    if metrics.value.characteristics.is_apple_silicon {
        assert!(
            metrics.value.memory.total >= 1_073_741_824,
            "Apple Silicon GPU should report reasonable memory allocation: {}",
            metrics.value.memory.total
        );
    }

    if let Some(temp) = metrics.value.temperature {
        assert!((35.0..=80.0).contains(&temp), "Temperature should be in a reasonable range: {}", temp);
    }
} 