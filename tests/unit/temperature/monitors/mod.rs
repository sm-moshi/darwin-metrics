mod ambient;
mod battery;
mod cpu;
mod fan;
mod gpu;

use darwin_metrics::hardware::temperature::{FanMonitoring, Temperature, TemperatureMonitor};

use super::TEST_MUTEX;

#[tokio::test]
async fn test_monitor_traits() {
    let _lock = TEST_MUTEX.lock();
    let temp = Temperature::new().unwrap();

    // Test CPU monitor
    let cpu_monitor = temp.cpu_monitor();
    assert!(cpu_monitor.name().await.is_ok());
    assert!(cpu_monitor.temperature().await.is_ok());
    assert!(cpu_monitor.is_critical().await.is_ok());

    // Test GPU monitor
    let gpu_monitor = temp.gpu_monitor();
    assert!(gpu_monitor.name().await.is_ok());
    // GPU temperature might not be available on all systems
    let _ = gpu_monitor.temperature().await;
    assert!(gpu_monitor.is_critical().await.is_ok());

    // Test ambient monitor
    let ambient_monitor = temp.ambient_monitor();
    assert!(ambient_monitor.name().await.is_ok());
    // Ambient temperature might not be available on all systems
    let _ = ambient_monitor.temperature().await;
    assert!(ambient_monitor.is_critical().await.is_ok());

    // Test battery monitor
    let battery_monitor = temp.battery_monitor();
    assert!(battery_monitor.name().await.is_ok());
    // Battery temperature might not be available on all systems
    let _ = battery_monitor.temperature().await;
    assert!(battery_monitor.is_critical().await.is_ok());

    // Test fan monitor
    let fan_monitor = temp.fan_monitor(0);
    assert!(fan_monitor.fan_name().await.is_ok());
    assert!(fan_monitor.speed_rpm().await.is_ok());
    assert!(fan_monitor.min_speed().await.is_ok());
    assert!(fan_monitor.max_speed().await.is_ok());
    assert!(fan_monitor.percentage().await.is_ok());
}
