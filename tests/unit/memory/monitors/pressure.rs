use super::super::TEST_MUTEX;
use crate::hardware::memory::{Memory, MemoryMonitor, PressureLevel};

#[tokio::test]
async fn test_pressure_monitor_creation() {
    let _lock = TEST_MUTEX.lock();
    let memory = Memory::new().unwrap();
    let monitor = memory.pressure_monitor();

    assert!(monitor.name().await.is_ok());
    assert!(monitor.hardware_type().await.is_ok());
    assert!(monitor.device_id().await.is_ok());
}

#[tokio::test]
async fn test_pressure_thresholds() {
    let _lock = TEST_MUTEX.lock();
    let memory = Memory::new().unwrap();
    let mut monitor = memory.pressure_monitor();

    // Test default thresholds
    let pressure = monitor.pressure_percentage().await.unwrap();
    let level = monitor.pressure_level().await.unwrap();
    assert!((0.0..=100.0).contains(&pressure));

    // Test custom thresholds
    assert!(monitor.set_thresholds(70.0, 90.0).is_ok());
    assert!(monitor.set_thresholds(-1.0, 90.0).is_err()); // Invalid threshold
    assert!(monitor.set_thresholds(90.0, 70.0).is_err()); // Warning > Critical
}

#[tokio::test]
async fn test_pressure_metrics() {
    let _lock = TEST_MUTEX.lock();
    let memory = Memory::new().unwrap();
    let monitor = memory.pressure_monitor();

    let info = monitor.memory_info().await.unwrap();
    assert!(info.total > 0);
    assert!(info.free <= info.total);
    assert!(info.used <= info.total);
    assert!(info.pressure >= 0.0 && info.pressure <= 1.0);

    let pressure = monitor.pressure_percentage().await.unwrap();
    assert!((0.0..=100.0).contains(&pressure));
}

#[tokio::test]
async fn test_pressure_level_transitions() {
    let _lock = TEST_MUTEX.lock();
    let memory = Memory::new().unwrap();
    let mut monitor = memory.pressure_monitor();

    // Set thresholds for testing
    monitor.set_thresholds(50.0, 75.0).unwrap();

    // Test pressure level determination
    let level = monitor.pressure_level().await.unwrap();
    match level {
        PressureLevel::Normal => {
            let pressure = monitor.pressure_percentage().await.unwrap();
            assert!(pressure < 50.0);
        }
        PressureLevel::Warning => {
            let pressure = monitor.pressure_percentage().await.unwrap();
            assert!(pressure >= 50.0 && pressure < 75.0);
        }
        PressureLevel::Critical => {
            let pressure = monitor.pressure_percentage().await.unwrap();
            assert!(pressure >= 75.0);
        }
    }
} 