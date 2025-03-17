use darwin_metrics::hardware::gpu::{
    Gpu, GpuCharacteristicsMonitor, GpuMemoryMonitor, GpuTemperatureMonitor, GpuUtilizationMonitor, HardwareMonitor,
    MemoryMonitor, TemperatureMonitor, UtilizationMonitor,
};

#[tokio::test]
async fn test_gpu_monitor_creation() {
    let gpu = Gpu::new();

    // Test characteristics monitor
    let char_monitor = GpuCharacteristicsMonitor::new(gpu.clone());
    assert_eq!(char_monitor.name(), "GPU Characteristics Monitor");
    assert_eq!(char_monitor.hardware_type(), "gpu");

    // Test memory monitor
    let mem_monitor = GpuMemoryMonitor::new(gpu.clone());
    assert_eq!(mem_monitor.name(), "GPU Memory Monitor");
    assert_eq!(mem_monitor.hardware_type(), "gpu");

    // Test temperature monitor
    let temp_monitor = GpuTemperatureMonitor::new(gpu.clone());
    assert_eq!(temp_monitor.name(), "GPU Temperature Monitor");
    assert_eq!(temp_monitor.hardware_type(), "gpu");

    // Test utilization monitor
    let util_monitor = GpuUtilizationMonitor::new(None, 0);
    assert_eq!(util_monitor.name(), "GPU Utilization Monitor");
    assert_eq!(util_monitor.hardware_type(), "gpu");
}

#[tokio::test]
async fn test_gpu_monitor_integration() {
    let gpu = Gpu::new();

    // Test characteristics
    let char_monitor = GpuCharacteristicsMonitor::new(gpu.clone());
    let name = char_monitor.name().await.unwrap();
    let chars = char_monitor.get_characteristics().await.unwrap();
    assert!(!name.is_empty(), "GPU name should not be empty");
    println!("GPU: {} (integrated: {})", name, chars.is_integrated);

    // Test memory
    let mem_monitor = GpuMemoryMonitor::new(gpu.clone());
    let memory_metrics = mem_monitor.get_memory_usage().await.unwrap();
    println!(
        "Memory - Total: {}, Used: {}, Free: {}",
        memory_metrics.total_bytes,
        memory_metrics.used_bytes,
        memory_metrics.total_bytes - memory_metrics.used_bytes
    );
    assert!(memory_metrics.total_bytes > 0, "Total memory should be greater than 0");
    assert!(memory_metrics.used_bytes <= memory_metrics.total_bytes, "Used memory should not exceed total");

    // Test temperature
    let temp_monitor = GpuTemperatureMonitor::new(gpu.clone());
    let temp = temp_monitor.get_temperature().await.unwrap();
    let is_critical = temp_monitor.is_critical().await.unwrap();
    assert!(temp >= 0.0 && temp < 150.0, "Temperature should be in reasonable range");
    println!("Temperature: {}°C (Critical: {})", temp, is_critical);

    // Test utilization
    let util_monitor = GpuUtilizationMonitor::new(None, 0);
    let util = util_monitor.utilization().await.unwrap();
    assert!(util >= 0.0 && util <= 100.0, "Utilization should be between 0-100%");
    println!("Utilization: {}%", util);
}

#[tokio::test]
async fn test_gpu_error_handling() {
    let gpu = Gpu::new();

    // Test characteristics error handling
    let char_monitor = GpuCharacteristicsMonitor::new(gpu.clone());
    assert!(char_monitor.name().await.is_ok());
    assert!(char_monitor.get_characteristics().await.is_ok());

    // Test memory error handling
    let mem_monitor = GpuMemoryMonitor::new(gpu.clone());
    assert!(mem_monitor.get_memory_usage().await.is_ok());

    // Test temperature error handling
    let temp_monitor = GpuTemperatureMonitor::new(gpu.clone());
    assert!(temp_monitor.get_temperature().await.is_ok());
    assert!(temp_monitor.is_critical().await.is_ok());

    // Test utilization error handling
    let util_monitor = GpuUtilizationMonitor::new(None, 0);
    assert!(util_monitor.utilization().await.is_ok());
}
