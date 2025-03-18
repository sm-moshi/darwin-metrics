use super::super::TEST_MUTEX;
use crate::gpu::{Gpu, GpuCharacteristicsMonitor, HardwareMonitor};

#[tokio::test]
async fn test_characteristics_monitor_creation() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuCharacteristicsMonitor::new(gpu);

    assert_eq!(monitor.name(), "GPU Characteristics Monitor");
    assert_eq!(monitor.hardware_type(), "gpu");
    assert!(!monitor.device_id().is_empty());
}

#[tokio::test]
async fn test_gpu_name() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuCharacteristicsMonitor::new(gpu);

    let name = monitor.name().await.unwrap();
    assert!(!name.is_empty(), "GPU name should not be empty");
    println!("GPU Name: {}", name);
}

#[tokio::test]
async fn test_gpu_characteristics() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuCharacteristicsMonitor::new(gpu);

    let chars = monitor.characteristics().await.unwrap();

    // Verify characteristics fields
    if cfg!(target_arch = "aarch64") {
        assert!(chars.is_apple_silicon, "Should be Apple Silicon on aarch64");
        assert!(chars.is_integrated, "Apple Silicon GPU should be integrated");
    }

    // Core count should be set for Apple Silicon
    if chars.is_apple_silicon {
        assert!(chars.core_count.is_some(), "Apple Silicon GPU should have core count");
    }

    println!("GPU Characteristics: {:?}", chars);
}

#[tokio::test]
async fn test_gpu_detection() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuCharacteristicsMonitor::new(gpu);

    let chars = monitor.characteristics().await.unwrap();
    let name = monitor.name().await.unwrap();

    // Verify GPU detection logic
    if cfg!(target_arch = "aarch64") {
        assert!(name.contains("Apple"), "Name should contain 'Apple' on Apple Silicon");
        assert!(chars.is_apple_silicon, "Should be detected as Apple Silicon");
    } else {
        // On Intel, name should contain useful information
        assert!(!name.contains("Unknown"), "GPU name should be detected on Intel");
    }

    println!("Detected GPU: {} ({:?})", name, chars);
}
