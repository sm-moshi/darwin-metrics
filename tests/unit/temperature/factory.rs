use std::sync::Arc;

use darwin_metrics::hardware::iokit::{MockIOKit, ThermalInfo};
use darwin_metrics::temperature::create_monitor;
use darwin_metrics::traits::TemperatureMonitor;

use super::TEST_MUTEX;

#[tokio::test]
async fn test_temperature_factory() {
    let _lock = TEST_MUTEX.lock();

    let mut mock_iokit = MockIOKit::new();
    mock_iokit.expect_get_thermal_info().returning(|| {
        Ok(ThermalInfo {
            cpu_temp: 50.0,
            gpu_temp: Some(60.0),
            ambient_temp: Some(30.0),
            battery_temp: Some(40.0),
            ssd_temp: Some(45.0),
            fan_speed: 2000,
            heatsink_temp: None,
            thermal_throttling: false,
            dict: darwin_metrics::utils::core::dictionary::SafeDictionary::new(),
        })
    });

    let io_kit = Arc::new(Box::new(mock_iokit) as Box<dyn darwin_metrics::hardware::iokit::IOKit>);

    // Test creating CPU monitor
    let cpu_monitor = create_monitor("cpu", io_kit.clone()).unwrap();
    let metric = cpu_monitor.get_metric().await.unwrap();
    assert_eq!(metric.value.as_celsius(), 50.0);

    // Test creating SSD monitor
    let ssd_monitor = create_monitor("ssd", io_kit.clone()).unwrap();
    let metric = ssd_monitor.get_metric().await.unwrap();
    assert_eq!(metric.value.as_celsius(), 45.0);

    // Test creating all monitors - using individual create calls to test each type
    let types = vec!["cpu", "gpu", "ambient", "battery", "ssd"];
    for monitor_type in types {
        let monitor = create_monitor(monitor_type, io_kit.clone());
        assert!(monitor.is_ok(), "Failed to create monitor for type: {}", monitor_type);
    }
}

#[tokio::test]
async fn test_invalid_monitor_type() {
    let _lock = TEST_MUTEX.lock();

    let mock_iokit = MockIOKit::new();
    let io_kit = Arc::new(Box::new(mock_iokit) as Box<dyn darwin_metrics::hardware::iokit::IOKit>);

    let result = create_monitor("invalid_type", io_kit);
    assert!(result.is_err());

    match result {
        Err(darwin_metrics::error::Error::InvalidMonitorType(name)) => {
            assert_eq!(name, "invalid_type");
        },
        _ => panic!("Expected InvalidMonitorType error"),
    }
}
