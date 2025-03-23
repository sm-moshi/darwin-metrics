use crate::battery::Battery;
use crate::error::Result;
use crate::tests::common::TestBatteryBuilder;

pub mod monitors;

#[test]
fn test_battery_creation() -> Result<()> {
    let battery = TestBatteryBuilder::new().build()?;
    assert!(battery.is_present());
    Ok(())
}

#[test]
fn test_battery_metrics() -> Result<()> {
    let battery = TestBatteryBuilder::new()
        .present(true)
        .charging(false)
        .cycle_count(100)
        .percentage(75.0)
        .temperature(35.0)
        .capacity(8000.0, 10000.0)
        .time_remaining(7200)
        .build()?;

    assert_eq!(battery.cycle_count()?, 100);
    assert_eq!(battery.percentage()?, 75.0);
    assert_eq!(battery.temperature()?, 35.0);
    assert_eq!(battery.current_capacity()?, 8000.0);
    assert_eq!(battery.design_capacity()?, 10000.0);
    assert!(!battery.is_charging()?);

    Ok(())
}
