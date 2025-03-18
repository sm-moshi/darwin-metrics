use super::super::TEST_MUTEX;
use crate::memory::{Memory, MemoryMonitor};

#[tokio::test]
async fn test_usage_monitor_creation() {
    let _lock = TEST_MUTEX.lock();
    let memory = Memory::new().unwrap();
    let monitor = memory.usage_monitor();

    assert!(monitor.name().await.is_ok());
    assert!(monitor.hardware_type().await.is_ok());
    assert!(monitor.device_id().await.is_ok());
}

#[tokio::test]
async fn test_memory_info() {
    let _lock = TEST_MUTEX.lock();
    let memory = Memory::new().unwrap();
    let monitor = memory.usage_monitor();

    let info = monitor.memory_info().await.unwrap();

    // Test basic memory metrics
    assert!(info.total > 0);
    assert!(info.free <= info.total);
    assert!(info.used <= info.total);
    assert_eq!(info.total, info.free + info.used);
    assert!(info.pressure >= 0.0 && info.pressure <= 1.0);
    assert!(info.page_size > 0);

    // Test page states
    assert!(info.page_states.active <= info.total / info.page_size);
    assert!(info.page_states.inactive <= info.total / info.page_size);
    assert!(info.page_states.wired <= info.total / info.page_size);
    assert!(info.page_states.free <= info.total / info.page_size);
}

#[tokio::test]
async fn test_usage_percentage() {
    let _lock = TEST_MUTEX.lock();
    let memory = Memory::new().unwrap();
    let monitor = memory.usage_monitor();

    let percentage = monitor.usage_percentage().await.unwrap();
    assert!((0.0..=100.0).contains(&percentage));

    // Verify percentage calculation
    let info = monitor.memory_info().await.unwrap();
    let calculated = (info.used as f64 / info.total as f64) * 100.0;
    assert!((percentage - calculated).abs() < 0.1);
}

#[tokio::test]
async fn test_page_states() {
    let _lock = TEST_MUTEX.lock();
    let memory = Memory::new().unwrap();
    let monitor = memory.usage_monitor();

    let states = monitor.page_states().await.unwrap();
    let info = monitor.memory_info().await.unwrap();

    // Test page state consistency
    assert_eq!(states.active, info.page_states.active);
    assert_eq!(states.inactive, info.page_states.inactive);
    assert_eq!(states.wired, info.page_states.wired);
    assert_eq!(states.free, info.page_states.free);
    assert_eq!(states.compressed, info.page_states.compressed);

    // Test that page counts are reasonable
    let total_pages = info.total / info.page_size;
    assert!(states.active + states.inactive + states.wired + states.free <= total_pages);
} 