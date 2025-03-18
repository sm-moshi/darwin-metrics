use super::super::TEST_MUTEX;
use crate::gpu::{Gpu, GpuMemoryMonitor, HardwareMonitor, MemoryMonitor};
use std::time::Duration;

#[tokio::test]
async fn test_memory_monitor_creation() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuMemoryMonitor::new(gpu);

    assert_eq!(monitor.name(), "GPU Memory Monitor");
    assert_eq!(monitor.hardware_type(), "gpu");
    assert!(!monitor.device_id().is_empty());
}

#[tokio::test]
async fn test_memory_metrics() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuMemoryMonitor::new(gpu);

    let total = monitor.total_memory().await.unwrap();
    let used = monitor.used_memory().await.unwrap();
    let free = monitor.free_memory().await.unwrap();
    let usage = monitor.memory_usage().await.unwrap();

    assert!(total > 0, "Total memory should be greater than 0");
    assert!(used <= total, "Used memory should not exceed total");
    assert!(free <= total, "Free memory should not exceed total");
    assert!(usage >= 0.0 && usage <= 100.0, "Memory usage should be between 0-100%");

    println!("Memory Metrics:");
    println!("Total: {} bytes", total);
    println!("Used: {} bytes", used);
    println!("Free: {} bytes", free);
    println!("Usage: {}%", usage);
}

#[tokio::test]
async fn test_memory_info() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuMemoryMonitor::new(gpu);

    let info = monitor.memory_info().await.unwrap();

    assert!(info.total > 0, "Total memory should be greater than 0");
    assert!(info.used <= info.total, "Used memory should not exceed total");
    assert!(info.free <= info.total, "Free memory should not exceed total");
    assert_eq!(info.total, info.used + info.free, "Total should equal used + free");

    println!("Memory Info: {:?}", info);
}

#[tokio::test]
async fn test_memory_updates() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuMemoryMonitor::new(gpu);

    let mut prev_used = monitor.used_memory().await.unwrap();
    let total = monitor.total_memory().await.unwrap();

    // Test multiple consecutive updates
    for _ in 0..3 {
        tokio::time::sleep(Duration::from_millis(100)).await;

        let used = monitor.used_memory().await.unwrap();
        let free = monitor.free_memory().await.unwrap();
        let usage = monitor.memory_usage().await.unwrap();

        assert!(used <= total, "Used memory should not exceed total");
        assert!(free <= total, "Free memory should not exceed total");
        assert!(usage >= 0.0 && usage <= 100.0, "Usage should be between 0-100%");
        assert_eq!(total, used + free, "Total should equal used + free");

        // Memory usage can go up or down, so we just verify it's changing
        assert!(used != prev_used, "Memory usage should change between updates");
        prev_used = used;

        println!("Update - Used: {}, Free: {}, Usage: {}%", used, free, usage);
    }
}
