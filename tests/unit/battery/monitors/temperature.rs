use crate::{battery::Battery, error::Result, tests::common::TestBatteryBuilder};

#[test]
fn test_temperature_monitor() -> Result<()> {
    let battery = TestBatteryBuilder::new().temperature(35.0).build()?;

    let monitor = battery.temperature_monitor()?;
    assert_eq!(monitor.temperature()?, 35.0);
    assert_eq!(monitor.temperature_celsius()?, 35.0);
    assert_eq!(monitor.temperature_fahrenheit()?, 95.0);

    Ok(())
}

#[test]
fn test_temperature_monitor_conversion() -> Result<()> {
    let battery = TestBatteryBuilder::new().temperature(0.0).build()?;

    let monitor = battery.temperature_monitor()?;
    assert_eq!(monitor.temperature_celsius()?, 0.0);
    assert_eq!(monitor.temperature_fahrenheit()?, 32.0);

    Ok(())
}
