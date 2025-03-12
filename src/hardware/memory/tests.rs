use super::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn test_memory_initialization() {
    let memory = Memory::new();
    assert!(memory.is_ok(), "Should be able to initialize Memory");
}

#[test]
fn test_memory_update() {
    let mut memory = Memory::new().unwrap();
    let result = memory.update();
    assert!(result.is_ok(), "Update should succeed");
}

#[test]
fn test_memory_metrics() {
    let memory = Memory::new().unwrap();

    // Basic validations
    assert!(memory.total > 0, "Total memory should be positive");
    assert!(memory.available > 0, "Available memory should be positive");
    assert!(memory.used > 0, "Used memory should be positive");
    assert!(memory.used <= memory.total, "Used memory should not exceed total");
    assert!(memory.pressure >= 0.0 && memory.pressure <= 1.0, "Pressure should be between 0 and 1");

    // Page state validations
    assert!(memory.page_states.free > 0, "Free pages should be positive");
    assert!(memory.page_states.active > 0, "Active pages should be positive");

    // Swap usage validation - might be 0 on systems without swap For u64 values, they are always >= 0, so no need to
    // test that
    if memory.swap_usage.total > 0 {
        assert!(
            memory.swap_usage.used <= memory.swap_usage.total,
            "Used swap should not exceed total"
        );
    }
}

#[test]
fn test_usage_percentage() {
    let memory = Memory::new().unwrap();
    let percentage = memory.usage_percentage();

    assert!(
        (0.0..=100.0).contains(&percentage),
        "Usage percentage should be between 0 and 100, got {}",
        percentage
    );
}

#[test]
fn test_pressure_callbacks() {
    let memory = Memory::new().unwrap();
    let pressure_level = Arc::new(Mutex::new(PressureLevel::Normal));
    let pressure_level_clone = pressure_level.clone();

    // Add a callback that updates the pressure level
    memory.on_pressure_change(move |level| {
        let mut guard = pressure_level_clone.lock().unwrap();
        *guard = level;
    });

    // Now force a check
    memory.check_pressure_thresholds();

    // The level should match the current pressure
    let level = memory.pressure_level();
    let callback_level = *pressure_level.lock().unwrap();

    assert_eq!(level, callback_level, "Callback pressure level should match current level");
}

#[test]
fn test_custom_thresholds() {
    let mut memory = Memory::new().unwrap();

    // Set custom thresholds
    let result = memory.set_pressure_thresholds(0.25, 0.75);
    assert!(result.is_ok(), "Setting thresholds should succeed");

    // Test invalid thresholds (warning > critical)
    let result = memory.set_pressure_thresholds(0.8, 0.5);
    assert!(result.is_err(), "Setting invalid thresholds should fail");

    // Test invalid thresholds (out of range)
    let result = memory.set_pressure_thresholds(-0.1, 0.5);
    assert!(result.is_err(), "Setting out-of-range thresholds should fail");

    let result = memory.set_pressure_thresholds(0.3, 1.5);
    assert!(result.is_err(), "Setting out-of-range thresholds should fail");
}

#[test]
fn test_memory_info_functions() {
    // Test individual info functions
    let total = Memory::get_total_memory();
    assert!(total.is_ok(), "Should get total memory");
    assert!(total.unwrap() > 0, "Total memory should be positive");

    let page_size = Memory::get_page_size();
    assert!(page_size.is_ok(), "Should get page size");
    assert!(page_size.unwrap() > 0, "Page size should be positive");

    let vm_stats = Memory::get_vm_statistics();
    assert!(vm_stats.is_ok(), "Should get VM statistics");

    let swap = Memory::get_swap_usage();
    assert!(swap.is_ok(), "Should get swap usage");
}

#[tokio::test]
async fn test_memory_monitoring() {
    let memory = Memory::new().unwrap();
    let monitor_handle = memory.start_monitoring(100).await;

    assert!(monitor_handle.is_ok(), "Should start monitoring successfully");

    let handle = monitor_handle.unwrap();
    assert!(handle.is_active(), "Monitor should be active initially");

    // Sleep briefly to allow monitor to run
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Stop the monitor
    handle.stop();
    assert!(!handle.is_active(), "Monitor should be inactive after stopping");
}

#[tokio::test]
async fn test_update_async() {
    let mut memory = Memory::new().unwrap();
    let result = memory.update_async().await;
    assert!(result.is_ok(), "Async update should succeed");
}

#[tokio::test]
async fn test_get_info_async() {
    let memory_result = Memory::get_info_async().await;
    assert!(memory_result.is_ok(), "Async get_info should succeed");

    let memory = memory_result.unwrap();
    assert!(memory.total > 0, "Total memory should be positive");
}
