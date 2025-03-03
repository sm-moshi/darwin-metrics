use super::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use test_log::test;

#[test]
fn test_battery_info_ffi_lifecycle() {
    unsafe {
        // Get battery info
        let battery_ptr = get_battery_info();
        assert!(!battery_ptr.is_null());

        // Test fields
        let battery = &*battery_ptr;
        assert!(battery.percentage >= 0.0 && battery.percentage <= 100.0);
        assert!(battery.health_percentage >= 0.0 && battery.health_percentage <= 100.0);
        assert!(battery.temperature >= 0.0);
        assert!(matches!(battery.power_source, 0..=2));

        // Clean up - explicitly drop the Box
        drop(Box::from_raw(battery_ptr));
    }
}

#[test]
fn test_battery_info_ffi_power_source_conversion() {
    use crate::battery::PowerSource;
    
    let battery = BatteryInfoFFI {
        is_present: true,
        is_charging: false,
        percentage: 85.5,
        time_remaining_minutes: 60,
        power_source: 1, // Battery
        cycle_count: 500,
        health_percentage: 85.0,
        temperature: 35.0,
    };

    assert_eq!(battery.power_source(), PowerSource::Battery);

    let battery = BatteryInfoFFI {
        power_source: 2, // AC
        ..battery
    };
    assert_eq!(battery.power_source(), PowerSource::AC);

    let battery = BatteryInfoFFI {
        power_source: 0, // Unknown
        ..battery
    };
    assert_eq!(battery.power_source(), PowerSource::Unknown);

    let battery = BatteryInfoFFI {
        power_source: 99, // Invalid
        ..battery
    };
    assert_eq!(battery.power_source(), PowerSource::Unknown);
}

#[test]
fn test_battery_info_ffi_clone() {
    let original = BatteryInfoFFI {
        is_present: true,
        is_charging: false,
        percentage: 85.5,
        time_remaining_minutes: 60,
        power_source: 1,
        cycle_count: 500,
        health_percentage: 85.0,
        temperature: 35.0,
    };

    let cloned = original.clone();
    
    assert_eq!(original.is_present, cloned.is_present);
    assert_eq!(original.is_charging, cloned.is_charging);
    assert_eq!(original.percentage, cloned.percentage);
    assert_eq!(original.time_remaining_minutes, cloned.time_remaining_minutes);
    assert_eq!(original.power_source, cloned.power_source);
    assert_eq!(original.cycle_count, cloned.cycle_count);
    assert_eq!(original.health_percentage, cloned.health_percentage);
    assert_eq!(original.temperature, cloned.temperature);
}

#[test]
fn test_battery_info_ffi_cache() {
    let manager = MetricsManager::new();
    
    // First call should populate cache
    let first_result = manager.get_battery_info();
    assert!(first_result.is_ok());
    
    // Immediate second call should use cache
    let second_result = manager.get_battery_info();
    assert!(second_result.is_ok());
    
    // Values should be identical due to caching
    let first = first_result.unwrap();
    let second = second_result.unwrap();
    assert_eq!(first.percentage, second.percentage);
    assert_eq!(first.power_source, second.power_source);
    assert_eq!(first.cycle_count, second.cycle_count);
    assert_eq!(first.health_percentage, second.health_percentage);
}

#[test]
fn test_battery_info_ffi_thread_safety() {
    use std::thread;
    use std::sync::Arc;
    
    let manager = Arc::new(MetricsManager::new());
    let mut handles = vec![];
    
    for _ in 0..10 {
        let manager_clone = manager.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                let result = manager_clone.get_battery_info();
                assert!(result.is_ok());
                thread::sleep(std::time::Duration::from_millis(1));
            }
        }));
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_cpu_info_ffi_clone() {
    let info = CPUInfoFFI {
        usage_percentage: 50.0,
        temperature: 45.0,
        core_count: 4,
        thread_count: 8,
    };
    
    let cloned = info.clone();
    assert_eq!(info.usage_percentage, cloned.usage_percentage);
    assert_eq!(info.temperature, cloned.temperature);
    assert_eq!(info.core_count, cloned.core_count);
    assert_eq!(info.thread_count, cloned.thread_count);
}

#[test]
fn test_resource_manager_cleanup() {
    let manager = ResourceManager::new();
    
    // Simulate battery service
    {
        let mut service = manager.battery_service.lock();
        *service = Some(123); // Mock service handle
    }
    
    // Simulate CPU stats
    {
        let mut stats = manager.cpu_stats.lock();
        *stats = Some(vec![0.5, 0.6, 0.7].into_boxed_slice());
    }
    
    // Test cleanup
    manager.cleanup();
    
    assert!(manager.battery_service.lock().is_none());
    assert!(manager.cpu_stats.lock().is_none());
    assert!(manager.memory_stats.lock().is_none());
}

#[tokio::test]
async fn test_metrics_manager_thread_safety() {
    let manager = Arc::new(MetricsManager::new());
    let mut handles = vec![];
    
    // Mock the battery info implementation
    for _ in 0..10 {
        let manager_clone = manager.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..100 {
                let result = manager_clone.get_battery_info();
                assert!(result.is_ok());
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }));
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

#[test]
fn test_metrics_manager_cache_timeout() {
    let manager = MetricsManager::new();
    
    // First call should populate cache
    let first_result = manager.get_battery_info();
    assert!(first_result.is_ok());
    
    // Immediate second call should use cache
    let second_result = manager.get_battery_info();
    assert!(second_result.is_ok());
    
    // Sleep past cache timeout
    thread::sleep(Duration::from_secs(2));
    
    // Call after timeout should refresh cache
    let third_result = manager.get_battery_info();
    assert!(third_result.is_ok());
}

#[test]
fn test_memory_info_ffi() {
    let info = MemoryInfoFFI {
        total_bytes: 16_000_000_000,
        used_bytes: 8_000_000_000,
        available_bytes: 8_000_000_000,
        swap_total_bytes: 2_000_000_000,
        swap_used_bytes: 1_000_000_000,
    };
    
    let cloned = info.clone();
    assert_eq!(info.total_bytes, cloned.total_bytes);
    assert_eq!(info.used_bytes, cloned.used_bytes);
    assert_eq!(info.available_bytes, cloned.available_bytes);
    assert_eq!(info.swap_total_bytes, cloned.swap_total_bytes);
    assert_eq!(info.swap_used_bytes, cloned.swap_used_bytes);
}

#[test]
fn test_ffi_null_safety() {
    unsafe {
        let battery_ptr = get_battery_info();
        assert!(!battery_ptr.is_null());
        drop(Box::from_raw(battery_ptr));
        
        let cpu_ptr = get_cpu_info();
        assert!(!cpu_ptr.is_null());
        drop(Box::from_raw(cpu_ptr));
        
        let memory_ptr = get_memory_info();
        assert!(!memory_ptr.is_null());
        drop(Box::from_raw(memory_ptr));
    }
}

// Test cleanup of resources in Drop implementations
#[test]
fn test_drop_cleanup() {
    // Test CPUInfoFFI cleanup
    {
        let _cpu_info = CPUInfoFFI {
            usage_percentage: 50.0,
            temperature: 45.0,
            core_count: 4,
            thread_count: 8,
        };
        // _cpu_info will be dropped here
    }
    
    // Test GPUInfoFFI cleanup
    {
        let _gpu_info = GPUInfoFFI {
            usage_percentage: 75.0,
            temperature: 65.0,
            memory_total_bytes: 8_000_000_000,
            memory_used_bytes: 4_000_000_000,
        };
        // _gpu_info will be dropped here
    }
} 