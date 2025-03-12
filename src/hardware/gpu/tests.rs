use super::*;

#[test]
fn test_gpu_initialization() {
    // Test that we can create a GPU
    let gpu = Gpu::new();
    assert!(gpu.is_ok(), "Should be able to initialize GPU");
}

#[test]
fn test_gpu_name() {
    // This test should work on all Apple hardware
    let gpu = Gpu::new().unwrap();
    let name = gpu.name();

    assert!(name.is_ok(), "Should be able to get GPU name");
    let name = name.unwrap();
    assert!(!name.is_empty(), "GPU name should not be empty");

    // Print for debugging
    println!("GPU name: {}", name);
}

#[test]
fn test_memory_info() {
    let gpu = Gpu::new().unwrap();
    let memory = gpu.estimate_memory_info();

    assert!(memory.is_ok(), "Should be able to get memory info");
    let memory = memory.unwrap();

    // Memory should be reasonable values
    assert!(memory.total > 0, "Total memory should be positive");
    assert!(memory.used <= memory.total, "Used memory should not exceed total");
    assert_eq!(
        memory.free,
        memory.total.saturating_sub(memory.used),
        "Free memory should be calculated correctly"
    );

    // Print for debugging
    println!("Memory: {:?}", memory);
}

#[test]
fn test_metrics() {
    let gpu = Gpu::new().unwrap();
    let metrics = gpu.metrics();

    assert!(metrics.is_ok(), "Should be able to get metrics");
    let metrics = metrics.unwrap();

    // Basic validations
    assert!(!metrics.name.is_empty(), "Name should not be empty");
    assert!(
        metrics.utilization >= 0.0 && metrics.utilization <= 100.0,
        "Utilization should be between 0-100%"
    );

    // Print for debugging
    println!("Metrics: {:?}", metrics);
}

#[test]
fn test_gpu_characteristics() {
    let gpu = Gpu::new().unwrap();
    let characteristics = gpu.get_characteristics();

    // Architecture validation
    if cfg!(target_arch = "aarch64") {
        assert!(
            characteristics.is_apple_silicon,
            "Should detect Apple Silicon on aarch64 hardware"
        );
        assert!(
            characteristics.is_integrated,
            "Apple Silicon GPUs should be detected as integrated"
        );
    }

    // Architecture detection tests are handled by individual cases above The original assertion was logically
    // equivalent to 'true'

    // Raytracing capability should be detected correctly
    if characteristics.is_apple_silicon && characteristics.has_raytracing {
        // Check that raytracing is only reported on M2/M3 chips, not M1
        if let Some(chip_info) = gpu.detect_apple_silicon_chip() {
            assert!(
                chip_info.contains("M2") || chip_info.contains("M3"),
                "Raytracing should only be reported on M2/M3 chips, not on: {}",
                chip_info
            );
        }
    }

    // Clock speed should be reasonable if available
    if let Some(clock_speed) = characteristics.clock_speed_mhz {
        assert!(
            clock_speed > 500 && clock_speed < 3000,
            "Clock speed should be in a reasonable range: {}",
            clock_speed
        );
    }

    // Core count should be reasonable if available
    if let Some(core_count) = characteristics.core_count {
        assert!(
            core_count > 0 && core_count < 200,
            "Core count should be in a reasonable range: {}",
            core_count
        );
    }

    // Print for debugging
    println!("Characteristics: {:?}", characteristics);
}

#[test]
fn test_apple_silicon_detection() {
    let gpu = Gpu::new().unwrap();

    if cfg!(target_arch = "aarch64") {
        // On Apple Silicon hardware, this should return a value
        assert!(
            gpu.detect_apple_silicon_chip().is_some(),
            "Should detect chip type on Apple Silicon hardware"
        );

        if let Some(chip_info) = gpu.detect_apple_silicon_chip() {
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
fn test_cpu_detection() {
    let gpu = Gpu::new().unwrap();
    let cpu_info = gpu.get_cpu_model();

    // Should get CPU info on any test system
    assert!(cpu_info.is_some(), "Should retrieve CPU model information");

    // Clone to avoid partial move issues
    let cpu_info_clone = cpu_info.clone();

    if let Some(info) = cpu_info {
        assert!(!info.is_empty(), "CPU model info should not be empty");

        if cfg!(target_arch = "aarch64") {
            // Check for Apple-designed CPU on Apple Silicon
            assert!(
                info.contains("Apple")
                    || info.contains("M1")
                    || info.contains("M2")
                    || info.contains("M3"),
                "On Apple Silicon, CPU should be an Apple-designed chip: {}",
                info
            );
        } else {
            // On Intel Macs, should be an Intel CPU
            assert!(info.contains("Intel"), "On Intel Macs, CPU should be an Intel chip: {}", info);
        }
    }

    // Print for debugging
    println!("CPU Model: {:?}", cpu_info_clone);
}

#[test]
fn test_metrics_characteristics() {
    let gpu = Gpu::new().unwrap();
    let metrics = gpu.metrics().unwrap();

    // Verify that the characteristics field is properly populated
    if cfg!(target_arch = "aarch64") {
        assert!(
            metrics.characteristics.is_apple_silicon,
            "Metrics should report Apple Silicon on ARM hardware"
        );
    }

    // Check that memory info makes sense with the characteristics
    if metrics.characteristics.is_apple_silicon {
        // For Apple Silicon, memory should be a percentage of system RAM Which should be a substantial amount (at least
        // 1GB for any reasonable test system)
        assert!(
            metrics.memory.total >= 1_073_741_824,
            "Apple Silicon GPU should report reasonable memory allocation: {}",
            metrics.memory.total
        );
    }

    // Check that temperature estimation varies by architecture
    if let Some(temp) = metrics.temperature {
        assert!(
            (35.0..=80.0).contains(&temp),
            "Temperature should be in a reasonable range: {}",
            temp
        );
    }
}
