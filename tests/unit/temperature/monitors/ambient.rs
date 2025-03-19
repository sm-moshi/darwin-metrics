use darwin_metrics::hardware::temperature::{Temperature, TemperatureMonitor};

use super::super::TEST_MUTEX;

#[tokio::test]
async fn test_ambient_monitor_creation() {
    let _lock = TEST_MUTEX.lock();
    let temp = Temperature::new().unwrap();
    let monitor = temp.ambient_monitor();

    assert_eq!(monitor.name().await.unwrap(), "Ambient Temperature Monitor");
    assert_eq!(monitor.hardware_type().await.unwrap(), "Ambient");
    assert_eq!(monitor.device_id().await.unwrap(), "ambient0");
}

#[tokio::test]
async fn test_ambient_temperature() {
    let _lock = TEST_MUTEX.lock();
    let temp = Temperature::new().unwrap();
    let monitor = temp.ambient_monitor();

    if let Ok(temperature) = monitor.temperature().await {
        println!("Ambient Temperature: {:.1}°C", temperature);

        // Basic sanity checks for ambient temperature
        assert!(temperature > -20.0, "Ambient temperature should be above -20°C");
        assert!(temperature < 60.0, "Ambient temperature should be below 60°C");
    } else {
        println!("Ambient temperature not available on this system");
    }
}

#[tokio::test]
async fn test_ambient_critical_threshold() {
    let _lock = TEST_MUTEX.lock();
    let temp = Temperature::new().unwrap();
    let monitor = temp.ambient_monitor();

    let threshold = monitor.critical_threshold().await.unwrap();
    assert!(threshold > 0.0, "Critical threshold should be positive");

    if let Ok(temperature) = monitor.temperature().await {
        let is_critical = monitor.is_critical().await.unwrap();
        println!(
            "Ambient Temperature: {:.1}°C, Threshold: {:.1}°C",
            temperature, threshold
        );
        assert_eq!(is_critical, temperature >= threshold);
    }
}

#[tokio::test]
async fn test_ambient_temperature_stability() {
    let _lock = TEST_MUTEX.lock();
    let temp = Temperature::new().unwrap();
    let monitor = temp.ambient_monitor();

    let mut temperatures = Vec::new();
    for _ in 0..5 {
        if let Ok(temp) = monitor.temperature().await {
            temperatures.push(temp);
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    if !temperatures.is_empty() {
        // Ambient temperature should be very stable over short periods
        let max_temp = temperatures.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_temp = temperatures.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        println!("Temperature range: {:.1}°C - {:.1}°C", min_temp, max_temp);
        assert!(
            max_temp - min_temp < 5.0,
            "Ambient temperature should be relatively stable"
        );
    } else {
        println!("Ambient temperature monitoring not available");
    }
}
