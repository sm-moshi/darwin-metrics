use super::super::TEST_MUTEX;
use crate::memory::{Memory, MemoryMonitor};

#[tokio::test]
async fn test_swap_monitor_creation() {
    let _lock = TEST_MUTEX.lock();
    let memory = Memory::new().unwrap();
    let monitor = memory.swap_monitor();

    assert!(monitor.name().await.is_ok());
    assert!(monitor.hardware_type().await.is_ok());
    assert!(monitor.device_id().await.is_ok());
}

#[tokio::test]
async fn test_swap_metrics() {
    let _lock = TEST_MUTEX.lock();
    let memory = Memory::new().unwrap();
    let monitor = memory.swap_monitor();

    let info = monitor.memory_info().await.unwrap();
    let swap = monitor.swap_usage().await.unwrap();

    // Test swap usage metrics
    assert!(swap.total >= 0);
    assert!(swap.used <= swap.total);
    assert!(swap.free <= swap.total);
    assert!(swap.pressure >= 0.0 && swap.pressure <= 1.0);
    assert!(swap.ins >= 0.0);
    assert!(swap.outs >= 0.0);
}

#[tokio::test]
async fn test_swap_usage_percentage() {
    let _lock = TEST_MUTEX.lock();
    let memory = Memory::new().unwrap();
    let monitor = memory.swap_monitor();

    let percentage = monitor.usage_percentage().await.unwrap();
    assert!((0.0..=100.0).contains(&percentage));

    let pressure = monitor.pressure_percentage().await.unwrap();
    assert!((0.0..=100.0).contains(&pressure));
}

#[tokio::test]
async fn test_swap_pressure_correlation() {
    let _lock = TEST_MUTEX.lock();
    let memory = Memory::new().unwrap();
    let monitor = memory.swap_monitor();

    let swap = monitor.swap_usage().await.unwrap();
    let pressure = monitor.pressure_percentage().await.unwrap();

    // Verify that pressure correlates with swap usage
    if swap.total > 0 {
        let calculated_pressure = (swap.used as f64 / swap.total as f64) * 100.0;
        assert!((calculated_pressure - pressure).abs() < 0.1);
    } else {
        assert_eq!(pressure, 0.0);
    }
} 