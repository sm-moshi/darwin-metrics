use crate::common::builders::{FanInfoBuilder, ThermalInfoBuilder};
use crate::common::mocks::temperature::MockIOKitClone;
use darwin_metrics::hardware::iokit::{FanInfo, ThermalInfo};
use darwin_metrics::hardware::temperature::{FanMonitoring, Temperature, TemperatureMonitor};

#[tokio::test]
async fn test_thermal_metrics_integration() {
    // Create a mock IOKit with custom thermal info
    let thermal_info = ThermalInfoBuilder::new()
        .with_cpu_temp(45.0)
        .with_gpu_temp(40.0)
        .with_ambient_temp(25.0)
        .with_battery_temp(32.0)
        .with_heatsink_temp(35.0)
        .with_throttling(false)
        .build();

    let fan_info =
        FanInfoBuilder::new().with_speed(2000).with_min_speed(1000).with_max_speed(4000).with_percentage(33.3).build();

    let mock_iokit = MockIOKitClone::new()
        .with_thermal_info(move || Ok(thermal_info.clone()))
        .with_fan_info(move || Ok(vec![fan_info.clone()]));

    // Create Temperature instance with mock IOKit
    let temp = Temperature::new().unwrap();

    // Test comprehensive thermal metrics
    let metrics = temp.get_thermal_metrics().await.unwrap();

    // Verify CPU temperature
    assert!(metrics.cpu_temperature.is_some());
    if let Some(cpu_temp) = metrics.cpu_temperature {
        assert!(cpu_temp > 0.0 && cpu_temp < 100.0);
        println!("CPU Temperature: {:.1}°C", cpu_temp);
    }

    // Verify GPU temperature
    if let Some(gpu_temp) = metrics.gpu_temperature {
        assert!(gpu_temp > 0.0 && gpu_temp < 100.0);
        println!("GPU Temperature: {:.1}°C", gpu_temp);
    }

    // Verify ambient temperature
    if let Some(ambient_temp) = metrics.ambient_temperature {
        assert!(ambient_temp > -20.0 && ambient_temp < 60.0);
        println!("Ambient Temperature: {:.1}°C", ambient_temp);
    }

    // Verify battery temperature
    if let Some(battery_temp) = metrics.battery_temperature {
        assert!(battery_temp > 0.0 && battery_temp < 60.0);
        println!("Battery Temperature: {:.1}°C", battery_temp);
    }

    // Verify fan information
    assert!(!metrics.fans.is_empty());
    for (i, fan) in metrics.fans.iter().enumerate() {
        println!("Fan {}: {} RPM ({}%)", i, fan.speed_rpm, fan.percentage);
        assert!(fan.speed_rpm >= fan.min_speed);
        assert!(fan.speed_rpm <= fan.max_speed);
        assert!((0.0..=100.0).contains(&fan.percentage));
    }

    // Verify throttling status
    println!("Thermal Throttling: {}", if metrics.is_throttling { "Yes" } else { "No" });
}

#[tokio::test]
async fn test_temperature_monitor_integration() {
    let temp = Temperature::new().unwrap();

    // Test all temperature monitors
    let monitors = vec![
        ("CPU", temp.cpu_monitor()),
        ("GPU", temp.gpu_monitor()),
        ("Ambient", temp.ambient_monitor()),
        ("Battery", temp.battery_monitor()),
    ];

    for (name, monitor) in monitors {
        if let Ok(temperature) = monitor.temperature().await {
            println!("{} Temperature: {:.1}°C", name, temperature);
            assert!(
                temperature > -20.0 && temperature < 120.0,
                "{} temperature {} is outside valid range",
                name,
                temperature
            );

            let is_critical = monitor.is_critical().await.unwrap();
            let threshold = monitor.critical_threshold().await.unwrap();
            println!(
                "{} Critical Status: {} (Threshold: {:.1}°C)",
                name,
                if is_critical { "Yes" } else { "No" },
                threshold
            );
        } else {
            println!("{} temperature not available", name);
        }
    }
}

#[tokio::test]
async fn test_fan_monitoring_integration() {
    let temp = Temperature::new().unwrap();
    let fan_count = temp.iokit.get_all_fans().unwrap().len();

    for i in 0..fan_count {
        let monitor = temp.fan_monitor(i);

        if let Ok(speed) = monitor.speed_rpm().await {
            let min = monitor.min_speed().await.unwrap();
            let max = monitor.max_speed().await.unwrap();
            let percentage = monitor.percentage().await.unwrap();
            let name = monitor.fan_name().await.unwrap();

            println!("{}: {} RPM ({}%) - Range: {}-{} RPM", name, speed, percentage, min, max);

            assert!(speed >= min && speed <= max);
            assert!((0.0..=100.0).contains(&percentage));
        } else {
            println!("Fan {} not available", i);
        }
    }
}
