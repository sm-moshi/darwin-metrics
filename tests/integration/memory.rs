use darwin_metrics::memory::{Memory, MemoryMonitor, PressureLevel};
use std::time::Duration;

#[tokio::test]
async fn test_memory_monitor_interactions() {
    let memory = Memory::new().unwrap();
    let usage_monitor = memory.usage_monitor();
    let pressure_monitor = memory.pressure_monitor();
    let swap_monitor = memory.swap_monitor();

    // Test concurrent monitoring
    let usage_info = usage_monitor.memory_info().await.unwrap();
    let pressure_info = pressure_monitor.memory_info().await.unwrap();
    let swap_info = swap_monitor.memory_info().await.unwrap();

    // Verify consistency across monitors
    assert_eq!(usage_info.total, pressure_info.total);
    assert_eq!(usage_info.total, swap_info.total);
    assert_eq!(usage_info.free, pressure_info.free);
    assert_eq!(usage_info.used, pressure_info.used);
}

#[tokio::test]
async fn test_memory_pressure_correlation() {
    let memory = Memory::new().unwrap();
    let usage_monitor = memory.usage_monitor();
    let pressure_monitor = memory.pressure_monitor();
    let swap_monitor = memory.swap_monitor();

    // Get metrics from all monitors
    let usage = usage_monitor.usage_percentage().await.unwrap();
    let pressure = pressure_monitor.pressure_percentage().await.unwrap();
    let swap_pressure = swap_monitor.pressure_percentage().await.unwrap();

    // Verify pressure correlations
    assert!((0.0..=100.0).contains(&usage));
    assert!((0.0..=100.0).contains(&pressure));
    assert!((0.0..=100.0).contains(&swap_pressure));

    // Test pressure level consistency
    let level = pressure_monitor.pressure_level().await.unwrap();
    match level {
        PressureLevel::Normal => assert!(pressure < 70.0),
        PressureLevel::Warning => assert!((70.0..90.0).contains(&pressure)),
        PressureLevel::Critical => assert!(pressure >= 90.0),
    }
}

#[tokio::test]
async fn test_memory_monitoring_over_time() {
    let memory = Memory::new().unwrap();
    let usage_monitor = memory.usage_monitor();
    let pressure_monitor = memory.pressure_monitor();
    let swap_monitor = memory.swap_monitor();

    // Initial readings
    let initial_usage = usage_monitor.memory_info().await.unwrap();
    let initial_swap = swap_monitor.swap_usage().await.unwrap();

    // Wait a short time for potential changes
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Take new readings
    let current_usage = usage_monitor.memory_info().await.unwrap();
    let current_swap = swap_monitor.swap_usage().await.unwrap();

    // Verify memory accounting remains consistent
    assert_eq!(initial_usage.total, current_usage.total);
    assert_eq!(initial_swap.total, current_swap.total);
    assert!(current_usage.free <= current_usage.total);
    assert!(current_usage.used <= current_usage.total);
}

#[tokio::test]
async fn test_memory_page_state_consistency() {
    let memory = Memory::new().unwrap();
    let usage_monitor = memory.usage_monitor();

    let info = usage_monitor.memory_info().await.unwrap();
    let states = usage_monitor.page_states().await.unwrap();

    // Verify page state consistency
    let total_pages = info.total / info.page_size;
    let total_accounted = states.active + states.inactive + states.wired + states.free;

    assert!(
        total_accounted <= total_pages,
        "Total accounted pages ({}) should not exceed total pages ({})",
        total_accounted,
        total_pages
    );

    // Verify individual page states are reasonable
    assert!(states.active <= total_pages);
    assert!(states.inactive <= total_pages);
    assert!(states.wired <= total_pages);
    assert!(states.free <= total_pages);
}

#[tokio::test]
async fn test_memory_subsystem_resilience() {
    let memory = Memory::new().unwrap();
    let usage_monitor = memory.usage_monitor();
    let pressure_monitor = memory.pressure_monitor();
    let swap_monitor = memory.swap_monitor();

    // Test rapid sequential monitoring
    for _ in 0..5 {
        let usage = usage_monitor.memory_info().await.unwrap();
        let pressure = pressure_monitor.pressure_percentage().await.unwrap();
        let swap = swap_monitor.swap_usage().await.unwrap();

        assert!(usage.total > 0);
        assert!((0.0..=100.0).contains(&pressure));
        assert!(swap.total >= 0);

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
