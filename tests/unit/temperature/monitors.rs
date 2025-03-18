use std::sync::Arc;

use darwin_metrics::hardware::iokit::{MockIOKit, ThermalInfo};
use darwin_metrics::temperature::{
    AmbientTemperatureMonitor, BatteryTemperatureMonitor, CpuTemperatureMonitor, FanMonitor, GpuTemperatureMonitor,
    SsdTemperatureMonitor, Temperature, create_monitor as create_temperature_monitor,
};
use darwin_metrics::traits::{HardwareType, TemperatureMonitor};

use super::TEST_MUTEX;

#[tokio::test]
async fn test_temperature_monitors() {
    let _lock = TEST_MUTEX.lock();

    let mut mock_iokit = MockIOKit::new();
    mock_iokit.expect_get_thermal_info().returning(|| {
        Ok(ThermalInfo::new(
            50.0,
            Some(60.0),
            Some(30.0),
            Some(40.0),
            Some(45.0),
            2000,
            None,
            false,
        ))
    });

    let io_kit = Arc::new(Box::new(mock_iokit) as Box<dyn darwin_metrics::hardware::iokit::IOKit>);

    // CPU Temperature Monitor
    let cpu_monitor = CpuTemperatureMonitor::new(io_kit.clone());
    assert_eq!(cpu_monitor.name(), "CPU Temperature");
    assert_eq!(cpu_monitor.hardware_type(), HardwareType::Cpu);

    let metric = cpu_monitor.get_metric().await.unwrap();
    assert_eq!(metric.value.as_celsius(), 50.0);
    assert!(!metric.is_critical);

    // GPU Temperature Monitor
    let gpu_monitor = GpuTemperatureMonitor::new(io_kit.clone());
    assert_eq!(gpu_monitor.name(), "GPU Temperature");
    assert_eq!(gpu_monitor.hardware_type(), HardwareType::Gpu);

    let metric = gpu_monitor.get_metric().await.unwrap();
    assert_eq!(metric.value.as_celsius(), 60.0);
    assert!(!metric.is_critical);

    // Ambient Temperature Monitor
    let ambient_monitor = AmbientTemperatureMonitor::new(io_kit.clone());
    assert_eq!(ambient_monitor.name(), "Ambient Temperature");
    assert_eq!(ambient_monitor.hardware_type(), HardwareType::System);

    let metric = ambient_monitor.get_metric().await.unwrap();
    assert_eq!(metric.value.as_celsius(), 30.0);
    assert!(!metric.is_critical);

    // Battery Temperature Monitor
    let battery_monitor = BatteryTemperatureMonitor::new(io_kit.clone());
    assert_eq!(battery_monitor.name(), "Battery Temperature");
    assert_eq!(battery_monitor.hardware_type(), HardwareType::Battery);

    let metric = battery_monitor.get_metric().await.unwrap();
    assert_eq!(metric.value.as_celsius(), 40.0);
    assert!(!metric.is_critical);

    // SSD Temperature Monitor
    let ssd_monitor = SsdTemperatureMonitor::new(io_kit.clone());
    assert_eq!(ssd_monitor.name(), "SSD Temperature");
    assert_eq!(ssd_monitor.hardware_type(), HardwareType::Storage);

    let metric = ssd_monitor.get_metric().await.unwrap();
    assert_eq!(metric.value.as_celsius(), 45.0);
    assert!(!metric.is_critical);

    // Fan Monitor
    let fan_monitor = FanMonitor::new(io_kit.clone());
    assert_eq!(fan_monitor.name(), "Main System Fan");
    assert_eq!(fan_monitor.hardware_type(), HardwareType::Cooling);

    let speed = fan_monitor.speed_rpm().await.unwrap();
    assert_eq!(speed, 2000);

    let percentage = fan_monitor.percentage().await.unwrap();
    assert!(percentage >= 0.0 && percentage <= 100.0);
}

#[tokio::test]
async fn test_temperature_metric_factory() {
    let _lock = TEST_MUTEX.lock();

    let mut mock_iokit = MockIOKit::new();
    mock_iokit.expect_get_thermal_info().returning(|| {
        Ok(ThermalInfo::new(
            50.0,
            Some(60.0),
            Some(30.0),
            Some(40.0),
            Some(45.0),
            2000,
            None,
            false,
        ))
    });

    let io_kit = Arc::new(Box::new(mock_iokit) as Box<dyn darwin_metrics::hardware::iokit::IOKit>);

    // Test creating all monitors - using individual create calls
    let cpu_monitor = create_temperature_monitor("cpu", io_kit.clone()).unwrap();
    let gpu_monitor = create_temperature_monitor("gpu", io_kit.clone()).unwrap();
    let ambient_monitor = create_temperature_monitor("ambient", io_kit.clone()).unwrap();
    let battery_monitor = create_temperature_monitor("battery", io_kit.clone()).unwrap();
    let ssd_monitor = create_temperature_monitor("ssd", io_kit.clone()).unwrap();

    // Test CPU monitor
    let metric = cpu_monitor.get_metric().await.unwrap();
    assert_eq!(metric.value.as_celsius(), 50.0);

    // Test GPU monitor
    let metric = gpu_monitor.get_metric().await.unwrap();
    assert_eq!(metric.value.as_celsius(), 60.0);
}

#[tokio::test]
async fn test_thermal_monitor_trait() {
    let _lock = TEST_MUTEX.lock();

    let mut mock_iokit = MockIOKit::new();
    mock_iokit.expect_get_thermal_info().returning(|| {
        Ok(ThermalInfo::new(
            50.0,
            Some(60.0),
            Some(30.0),
            Some(40.0),
            Some(45.0),
            2000,
            None,
            false,
        ))
    });

    let io_kit = Arc::new(Box::new(mock_iokit) as Box<dyn darwin_metrics::hardware::iokit::IOKit>);
    let temp = Temperature::with_iokit(io_kit);

    // Test CPU temperature
    let cpu_temp = temp.cpu_temperature().await.unwrap();
    assert!(cpu_temp.is_some());
    assert_eq!(cpu_temp.unwrap(), 50.0);

    // Test getting all thermal metrics
    let metrics = temp.get_thermal_metrics().await.unwrap();
    assert_eq!(metrics.cpu_temperature.unwrap(), 50.0);
    assert_eq!(metrics.gpu_temperature.unwrap(), 60.0);
    assert_eq!(metrics.ambient_temperature.unwrap(), 30.0);
    assert_eq!(metrics.battery_temperature.unwrap(), 40.0);
    assert_eq!(metrics.ssd_temperature.unwrap(), 45.0);

    // Test fan information
    let fans = temp.get_fans().await.unwrap();
    assert_eq!(fans.len(), 1);
    assert_eq!(fans[0].speed, 2000.0);
}
