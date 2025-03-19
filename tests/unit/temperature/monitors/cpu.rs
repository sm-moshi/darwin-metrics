use darwin_metrics::hardware::temperature::{Temperature, TemperatureMonitor};

use super::super::TEST_MUTEX;

#[tokio::test]
async fn test_cpu_monitor_creation() {
    let _lock = TEST_MUTEX.lock();
    let temp = Temperature::new().unwrap();
    let monitor = temp.cpu_monitor();

    assert_eq!(monitor.name().await.unwrap(), "CPU Temperature Monitor");
    assert_eq!(monitor.hardware_type().await.unwrap(), "CPU");
    assert_eq!(monitor.device_id().await.unwrap(), "cpu0");
}

#[tokio::test]
async fn test_cpu_temperature() {
    let _lock = TEST_MUTEX.lock();
    let temp = Temperature::new().unwrap();
    let monitor = temp.cpu_monitor();

    let temperature = monitor.temperature().await.unwrap();
    println!("CPU Temperature: {:.1}°C", temperature);

    // Basic sanity checks
    assert!(temperature > 0.0, "CPU temperature should be positive");
    assert!(temperature < 120.0, "CPU temperature should be less than 120°C");
}

#[tokio::test]
async fn test_cpu_critical_threshold() {
    let _lock = TEST_MUTEX.lock();
    let temp = Temperature::new().unwrap();
    let monitor = temp.cpu_monitor();

    let threshold = monitor.critical_threshold().await.unwrap();
    let temperature = monitor.temperature().await.unwrap();
    let is_critical = monitor.is_critical().await.unwrap();

    println!("CPU Temperature: {:.1}°C, Threshold: {:.1}°C", temperature, threshold);
    assert!(threshold > 0.0, "Critical threshold should be positive");
    assert_eq!(is_critical, temperature >= threshold);
}

#[tokio::test]
async fn test_cpu_temperature_stability() {
    let _lock = TEST_MUTEX.lock();
    let temp = Temperature::new().unwrap();
    let monitor = temp.cpu_monitor();

    let mut temperatures = Vec::new();
    for _ in 0..5 {
        let temp = monitor.temperature().await.unwrap();
        temperatures.push(temp);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Check that temperatures are relatively stable
    let max_temp = temperatures.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let min_temp = temperatures.iter().fold(f64::INFINITY, |a, &b| a.min(b));

    println!("Temperature range: {:.1}°C - {:.1}°C", min_temp, max_temp);
    assert!(max_temp - min_temp < 20.0, "Temperature should not vary drastically");
}
