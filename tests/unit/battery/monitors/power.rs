use crate::{battery::Battery, error::Result, tests::common::TestBatteryBuilder};

#[test]
fn test_power_monitor() -> Result<()> {
    let battery = TestBatteryBuilder::new().charging(true).build()?;

    let monitor = battery.power_monitor()?;
    assert!(monitor.is_charging()?);
    assert!(monitor.is_external_power()?);

    let battery = TestBatteryBuilder::new().charging(false).build()?;

    let monitor = battery.power_monitor()?;
    assert!(!monitor.is_charging()?);
    assert!(!monitor.is_external_power()?);

    Ok(())
}
