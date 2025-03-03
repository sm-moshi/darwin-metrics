use darwin_metrics::prelude::*;

#[cfg(test)]
mod tests {
    use darwin_metrics::battery::Battery;
    use darwin_metrics::gpu::GPU;

    #[test]
    fn test_battery_metrics() -> Result<(), Box<dyn std::error::Error>> {
        let battery = Battery::new()?;
        assert!(battery.percentage >= 0.0 && battery.percentage <= 100.0);
        assert!(battery.health_percentage >= 0.0 && battery.health_percentage <= 100.0);
        Ok(())
    }

    #[test]
    fn test_gpu_metrics() -> Result<(), Box<dyn std::error::Error>> {
        let gpu = GPU::new()?;
        let metrics = gpu.get_metrics()?;
        assert!(!metrics.name.is_empty());
        assert!(metrics.utilization >= 0.0 && metrics.utilization <= 100.0);
        Ok(())
    }
}

#[test]
fn test_multiple_metrics() -> Result<()> {
    // Test collecting multiple metrics simultaneously
    let battery = Battery::new()?;
    let gpu = GPU::new()?;

    // Both should work without interfering
    assert!(battery.percentage >= 0.0 && battery.percentage <= 100.0);
    assert!(!gpu.get_metrics()?.name.is_empty());

    Ok(())
}

#[test]
fn test_error_handling() -> Result<()> {
    use std::thread;
    use std::time::Duration;

    // Test rapid creation/destruction
    for _ in 0..5 {
        let gpu = GPU::new()?;
        let _metrics = gpu.get_metrics()?;
        thread::sleep(Duration::from_millis(10));
    }

    // Test concurrent access
    let handles: Vec<_> = (0..3)
        .map(|_| {
            thread::spawn(|| -> Result<()> {
                let gpu = GPU::new()?;
                let metrics = gpu.get_metrics()?;
                assert!(!metrics.name.is_empty());
                Ok(())
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok(())
}

#[test]
fn test_resource_cleanup() -> Result<()> {
    // Test proper cleanup of resources
    {
        let gpu = GPU::new()?;
        let _metrics = gpu.get_metrics()?;
    } // gpu should be dropped here

    // Should be able to create a new instance
    let gpu = GPU::new()?;
    assert!(!gpu.get_metrics()?.name.is_empty());

    Ok(())
}
