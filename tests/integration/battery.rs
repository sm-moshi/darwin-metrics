use darwin_metrics::{error::Result, hardware::battery::Battery};

#[test]
fn test_battery_integration() -> Result<()> {
    let battery = Battery::new_system()?;

    // Basic presence check
    assert!(battery.is_present());

    // Verify all metrics can be retrieved
    let _ = battery.percentage()?;
    let _ = battery.cycle_count()?;
    let _ = battery.temperature()?;
    let _ = battery.current_capacity()?;
    let _ = battery.design_capacity()?;
    let _ = battery.is_charging()?;
    let _ = battery.time_remaining()?;

    Ok(())
}
