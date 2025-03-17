#[cfg(test)]
use super::*;
use crate::hardware::cpu::{CpuMetrics, FrequencyMetrics, FrequencyMonitor, CPU};
use crate::utils::tests::test_utils::MockIOKit;

#[test]
fn test_cpu_initialization() {
    let cpu = CPU::new_with_mock().expect("Failed to create CPU instance");

    assert_eq!(cpu.physical_cores(), 8);
    assert_eq!(cpu.logical_cores(), 16);
    assert_eq!(cpu.frequency_mhz(), 3200.0);
    assert_eq!(cpu.model_name(), "Apple M1 Pro");
    assert_eq!(cpu.temperature(), Some(45.5));
    assert_eq!(cpu.core_usage().len(), 8);
}

#[test]
fn test_cpu_usage() {
    let cpu = CPU::new_with_mock().expect("Failed to create CPU instance");
    let expected_usage = 0.2;
    assert_eq!(cpu.get_cpu_usage(), expected_usage);
}

#[test]
fn test_cpu_metrics_trait() {
    let cpu = CPU::new_with_mock().expect("Failed to create CPU instance");

    assert_eq!(cpu.get_cpu_frequency(), 3200.0);
    assert_eq!(cpu.get_cpu_temperature(), Some(45.5));
    assert!(cpu.get_cpu_usage() >= 0.0 && cpu.get_cpu_usage() <= 1.0);
}

#[test]
fn test_frequency_metrics() {
    let cpu = CPU::new_with_mock().expect("Failed to create CPU instance");

    assert_eq!(cpu.frequency_mhz(), 3200.0);
    assert_eq!(cpu.min_frequency_mhz(), Some(1200.0));
    assert_eq!(cpu.max_frequency_mhz(), Some(3600.0));

    if let Some(freqs) = cpu.available_frequencies() {
        assert_eq!(freqs.len(), 5);
        assert_eq!(freqs[0], 1200.0);
        assert_eq!(freqs[4], 3600.0);
    }
}

#[test]
fn test_create_from_constructor() {
    let result = CPU::new();

    match result {
        Ok(cpu) => {
            assert!(cpu.logical_cores() > 0);
            assert!(cpu.physical_cores() > 0);
            assert!(!cpu.model_name().is_empty());
            assert!(cpu.frequency_mhz() > 0.0);
        },
        Err(e) => {
            println!("CPU::new() failed with: {}", e);
        },
    }
}

#[test]
fn test_real_update() {
    let result = CPU::new();

    if let Ok(mut cpu) = result {
        let update_result = cpu.update();

        if update_result.is_ok() {
            assert!(cpu.logical_cores() > 0);
            assert!(cpu.physical_cores() > 0);
            assert!(!cpu.model_name().is_empty());
            assert!(cpu.frequency_mhz() > 0.0);
            assert!(!cpu.core_usage().is_empty());
        }
    }
}

#[test]
fn test_frequency_monitor_new() {
    let monitor = FrequencyMonitor::new();
    assert!(matches!(monitor, FrequencyMonitor));
}

#[test]
fn test_frequency_monitor_default() {
    let monitor = FrequencyMonitor;
    assert!(matches!(monitor, FrequencyMonitor));
}

#[test]
fn test_frequency_monitor_get_metrics() {
    let monitor = FrequencyMonitor::new();
    let result = monitor.get_metrics();

    if result.is_ok() {
        let metrics = result.unwrap();
        assert!(metrics.current > 0.0);
        assert!(metrics.min > 0.0);
        assert!(metrics.max > 0.0);
        assert!(!metrics.available.is_empty());
    }
}

#[test]
fn test_frequency_metrics_struct() {
    let metrics = FrequencyMetrics {
        current: 2400.0,
        min: 1200.0,
        max: 3600.0,
        available: vec![1200.0, 1800.0, 2400.0, 3000.0, 3600.0],
    };

    assert_eq!(metrics.current, 2400.0);
    assert_eq!(metrics.min, 1200.0);
    assert_eq!(metrics.max, 3600.0);
    assert_eq!(metrics.available.len(), 5);
    assert_eq!(metrics.available[0], 1200.0);
    assert_eq!(metrics.available[4], 3600.0);
}

#[test]
fn test_frequency_metrics_clone() {
    let metrics = FrequencyMetrics {
        current: 2400.0,
        min: 1200.0,
        max: 3600.0,
        available: vec![1200.0, 1800.0, 2400.0, 3000.0, 3600.0],
    };

    let cloned = metrics.clone();

    assert_eq!(cloned.current, metrics.current);
    assert_eq!(cloned.min, metrics.min);
    assert_eq!(cloned.max, metrics.max);
    assert_eq!(cloned.available, metrics.available);
}

#[test]
fn test_frequency_metrics_debug() {
    let metrics = FrequencyMetrics {
        current: 2400.0,
        min: 1200.0,
        max: 3600.0,
        available: vec![1200.0, 1800.0, 2400.0, 3000.0, 3600.0],
    };

    let debug_str = format!("{:?}", metrics);

    assert!(debug_str.contains("current: 2400.0"));
    assert!(debug_str.contains("min: 1200.0"));
    assert!(debug_str.contains("max: 3600.0"));
    assert!(debug_str.contains("available: [1200.0, 1800.0, 2400.0, 3000.0, 3600.0]"));
}

#[test]
fn test_frequency_metrics_equality() {
    let metrics1 = FrequencyMetrics {
        current: 2400.0,
        min: 1200.0,
        max: 3600.0,
        available: vec![1200.0, 1800.0, 2400.0, 3000.0, 3600.0],
    };

    let metrics2 = FrequencyMetrics {
        current: 2400.0,
        min: 1200.0,
        max: 3600.0,
        available: vec![1200.0, 1800.0, 2400.0, 3000.0, 3600.0],
    };

    let metrics3 = FrequencyMetrics {
        current: 3000.0,
        min: 1200.0,
        max: 3600.0,
        available: vec![1200.0, 1800.0, 2400.0, 3000.0, 3600.0],
    };

    assert_eq!(metrics1, metrics2);
    assert_ne!(metrics1, metrics3);
}

#[test]
fn test_available_frequencies_calculation() {
    let metrics1 = FrequencyMetrics {
        current: 2400.0,
        min: 1200.0,
        max: 3600.0,
        available: vec![1200.0, 1800.0, 2400.0, 3000.0, 3600.0],
    };

    assert_eq!(metrics1.available[1] - metrics1.available[0], 600.0);
    assert_eq!(metrics1.available[2] - metrics1.available[1], 600.0);
    assert_eq!(metrics1.available[3] - metrics1.available[2], 600.0);
    assert_eq!(metrics1.available[4] - metrics1.available[3], 600.0);
}

#[test]
fn test_core_usage_values() {
    let mock_iokit = MockIOKit::new()
        .expect("Failed to create MockIOKit")
        .with_physical_cores(4)
        .expect("Failed to set physical cores")
        .with_logical_cores(8)
        .expect("Failed to set logical cores")
        .with_core_usage(vec![0.5, 0.6, 0.7, 0.8, 0.5, 0.6, 0.7, 0.8])
        .expect("Failed to set core usage");

    let cpu = CPU::new_with_iokit(Box::new(mock_iokit)).expect("Failed to create CPU instance");
    let core_usage = cpu.core_usage();

    assert_eq!(core_usage.len(), 8);
    assert_eq!(core_usage[0], 0.5);
    assert_eq!(core_usage[1], 0.6);
    assert_eq!(core_usage[2], 0.7);
    assert_eq!(core_usage[3], 0.8);
    assert_eq!(core_usage[4], 0.5);
    assert_eq!(core_usage[5], 0.6);
    assert_eq!(core_usage[6], 0.7);
    assert_eq!(core_usage[7], 0.8);
}

#[test]
fn test_core_usage_with_different_core_counts() {
    let mock_iokit = MockIOKit::new()
        .expect("Failed to create MockIOKit")
        .with_physical_cores(4)
        .expect("Failed to set physical cores")
        .with_logical_cores(8)
        .expect("Failed to set logical cores")
        .with_core_usage(vec![0.3, 0.4, 0.5, 0.6])
        .expect("Failed to set core usage");

    let cpu = CPU::new_with_iokit(Box::new(mock_iokit)).expect("Failed to create CPU instance");

    let core_usage = cpu.core_usage();
    assert_eq!(core_usage.len(), 8);

    for i in 0..4 {
        assert_eq!(core_usage[i], vec![0.3, 0.4, 0.5, 0.6][i]);
        assert_eq!(core_usage[i + 4], vec![0.3, 0.4, 0.5, 0.6][i]);
    }

    let mock_iokit = MockIOKit::new()
        .expect("Failed to create MockIOKit")
        .with_physical_cores(2)
        .expect("Failed to set physical cores")
        .with_logical_cores(4)
        .expect("Failed to set logical cores")
        .with_core_usage(vec![0.3, 0.4])
        .expect("Failed to set core usage");

    let cpu = CPU::new_with_iokit(Box::new(mock_iokit)).expect("Failed to create CPU instance");

    let core_usage = cpu.core_usage();
    assert_eq!(core_usage.len(), 4);

    assert_eq!(core_usage[0], 0.3);
    assert_eq!(core_usage[1], 0.4);
    assert_eq!(core_usage[2], 0.3);
    assert_eq!(core_usage[3], 0.4);
    assert_eq!(cpu.physical_cores(), 2);
    assert_eq!(cpu.logical_cores(), 4);
}

#[test]
fn test_cpu_metrics() {
    let mock_iokit = MockIOKit::new()
        .expect("Failed to create MockIOKit")
        .with_physical_cores(4)
        .expect("Failed to set physical cores")
        .with_logical_cores(4)
        .expect("Failed to set logical cores");

    let cpu = CPU::new_with_iokit(Box::new(mock_iokit)).expect("Failed to create CPU instance");
    let metrics = cpu.metrics();
    assert!(metrics.is_ok());
}

#[test]
fn test_cpu_metrics_with_usage() {
    let mock_iokit = MockIOKit::new()
        .expect("Failed to create MockIOKit")
        .with_physical_cores(4)
        .expect("Failed to set physical cores")
        .with_logical_cores(4)
        .expect("Failed to set logical cores")
        .with_core_usage(vec![0.5, 0.6, 0.7, 0.8])
        .expect("Failed to set core usage");

    let cpu = CPU::new_with_iokit(Box::new(mock_iokit)).expect("Failed to create CPU instance");
    let metrics = cpu.metrics();
    assert!(metrics.is_ok());
}

#[test]
fn test_temperature() {
    let mock_iokit = MockIOKit::new()
        .expect("Failed to create MockIOKit")
        .with_temperature(50.0)
        .expect("Failed to set temperature");

    let cpu = CPU::new_with_iokit(Box::new(mock_iokit)).expect("Failed to create CPU instance");
    assert_eq!(cpu.temperature(), Some(50.0));
}
