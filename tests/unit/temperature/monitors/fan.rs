use darwin_metrics::hardware::temperature::{FanMonitoring, Temperature};

use super::super::TEST_MUTEX;

#[tokio::test]
async fn test_fan_monitor_creation() {
    let _lock = TEST_MUTEX.lock();
    let temp = Temperature::new().unwrap();
    let monitor = temp.fan_monitor(0);

    assert_eq!(monitor.fan_name().await.unwrap(), "Fan 0");
}

#[tokio::test]
async fn test_fan_speeds() {
    let _lock = TEST_MUTEX.lock();
    let temp = Temperature::new().unwrap();
    let monitor = temp.fan_monitor(0);

    if let Ok(speed) = monitor.speed_rpm().await {
        let min = monitor.min_speed().await.unwrap();
        let max = monitor.max_speed().await.unwrap();

        println!("Fan Speed: {} RPM (Min: {}, Max: {})", speed, min, max);

        // Basic sanity checks
        assert!(speed >= min, "Current speed should be >= minimum speed");
        assert!(speed <= max, "Current speed should be <= maximum speed");
        assert!(min < max, "Minimum speed should be less than maximum speed");
    } else {
        println!("Fan monitoring not available on this system");
    }
}

#[tokio::test]
async fn test_fan_percentage() {
    let _lock = TEST_MUTEX.lock();
    let temp = Temperature::new().unwrap();
    let monitor = temp.fan_monitor(0);

    if let Ok(percentage) = monitor.percentage().await {
        println!("Fan Speed Percentage: {:.1}%", percentage);

        // Check percentage bounds
        assert!(
            (0.0..=100.0).contains(&percentage),
            "Percentage should be between 0 and 100"
        );

        // Verify percentage calculation
        let speed = monitor.speed_rpm().await.unwrap() as f64;
        let min = monitor.min_speed().await.unwrap() as f64;
        let max = monitor.max_speed().await.unwrap() as f64;

        if max > min {
            let calculated = ((speed - min) / (max - min)) * 100.0;
            assert!((percentage - calculated).abs() < 0.1, "Percentage calculation mismatch");
        }
    } else {
        println!("Fan monitoring not available on this system");
    }
}

#[tokio::test]
async fn test_fan_speed_stability() {
    let _lock = TEST_MUTEX.lock();
    let temp = Temperature::new().unwrap();
    let monitor = temp.fan_monitor(0);

    let mut speeds = Vec::new();
    for _ in 0..5 {
        if let Ok(speed) = monitor.speed_rpm().await {
            speeds.push(speed);
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    if !speeds.is_empty() {
        let max_speed = speeds.iter().max().unwrap();
        let min_speed = speeds.iter().min().unwrap();

        println!("Speed range: {} - {} RPM", min_speed, max_speed);

        // Fan speed can vary but shouldn't jump drastically in short periods
        let max_variation = monitor.max_speed().await.unwrap() as f64 * 0.2; // 20% of max speed
        assert!(
            (*max_speed as f64 - *min_speed as f64) < max_variation,
            "Fan speed should not vary drastically in short periods"
        );
    } else {
        println!("Fan monitoring not available");
    }
}
