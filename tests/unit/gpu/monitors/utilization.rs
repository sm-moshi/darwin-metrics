use std::time::Duration;

use super::super::TEST_MUTEX;
use crate::gpu::{Gpu, GpuUtilizationMonitor, HardwareMonitor, UtilizationMonitor};

#[tokio::test]
async fn test_utilization_monitor_creation() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuUtilizationMonitor::new(gpu);

    assert_eq!(monitor.name(), "GPU Utilization Monitor");
    assert_eq!(monitor.hardware_type(), "gpu");
    assert!(!monitor.device_id().is_empty());
}

#[tokio::test]
async fn test_utilization_metrics() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuUtilizationMonitor::new(gpu);

    let util = monitor.utilization().await.unwrap();
    let is_high = monitor.is_high_utilization().await.unwrap();

    assert!(util >= 0.0 && util <= 100.0, "Utilization should be between 0-100%");
    assert_eq!(
        is_high,
        util >= crate::gpu::constants::utilization::HIGH_UTILIZATION_THRESHOLD
    );

    println!("Utilization: {}%", util);
    println!("High Utilization: {}", is_high);
}

#[tokio::test]
async fn test_utilization_info() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuUtilizationMonitor::new(gpu);

    let info = monitor.utilization_info().await.unwrap();

    assert!(
        info.core >= 0.0 && info.core <= 100.0,
        "Core utilization should be between 0-100%"
    );
    assert!(
        info.memory >= 0.0 && info.memory <= 100.0,
        "Memory utilization should be between 0-100%"
    );

    if let Some(encoder) = info.encoder {
        assert!(
            encoder >= 0.0 && encoder <= 100.0,
            "Encoder utilization should be between 0-100%"
        );
    }

    if let Some(decoder) = info.decoder {
        assert!(
            decoder >= 0.0 && decoder <= 100.0,
            "Decoder utilization should be between 0-100%"
        );
    }

    println!("Utilization Info: {:?}", info);
}

#[tokio::test]
async fn test_component_utilization() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuUtilizationMonitor::new(gpu);

    let core_util = monitor.utilization().await.unwrap();
    let mem_util = monitor.memory_utilization().await.unwrap();
    let encoder_util = monitor.encoder_utilization().await.unwrap();
    let decoder_util = monitor.decoder_utilization().await.unwrap();

    assert!(
        core_util >= 0.0 && core_util <= 100.0,
        "Core utilization should be between 0-100%"
    );
    assert!(
        mem_util >= 0.0 && mem_util <= 100.0,
        "Memory utilization should be between 0-100%"
    );

    if let Some(encoder) = encoder_util {
        assert!(
            encoder >= 0.0 && encoder <= 100.0,
            "Encoder utilization should be between 0-100%"
        );
    }

    if let Some(decoder) = decoder_util {
        assert!(
            decoder >= 0.0 && decoder <= 100.0,
            "Decoder utilization should be between 0-100%"
        );
    }

    println!("Component Utilization:");
    println!("Core: {}%", core_util);
    println!("Memory: {}%", mem_util);
    println!("Encoder: {:?}%", encoder_util);
    println!("Decoder: {:?}%", decoder_util);
}

#[tokio::test]
async fn test_utilization_updates() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let gpu = Gpu::new();
    let monitor = GpuUtilizationMonitor::new(gpu);

    let mut prev_util = monitor.utilization().await.unwrap();

    // Test multiple consecutive updates
    for _ in 0..3 {
        tokio::time::sleep(Duration::from_millis(100)).await;

        let util = monitor.utilization().await.unwrap();
        let is_high = monitor.is_high_utilization().await.unwrap();
        let mem_util = monitor.memory_utilization().await.unwrap();

        assert!(util >= 0.0 && util <= 100.0, "Utilization should be between 0-100%");
        assert!(
            mem_util >= 0.0 && mem_util <= 100.0,
            "Memory utilization should be between 0-100%"
        );

        // Utilization might change between readings
        assert!(
            (util - prev_util).abs() < 50.0,
            "Utilization should not change drastically between readings"
        );
        prev_util = util;

        println!("Update - Core: {}%, Memory: {}%, High: {}", util, mem_util, is_high);
    }
}
