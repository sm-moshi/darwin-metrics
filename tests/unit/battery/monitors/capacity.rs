use crate::{battery::Battery, error::Result, tests::common::TestBatteryBuilder};

#[test]
fn test_capacity_monitor() -> Result<()> {
    let battery = TestBatteryBuilder::new().percentage(75.0).capacity(8000.0, 10000.0).build()?;

    let monitor = battery.capacity_monitor()?;
    assert_eq!(monitor.percentage()?, 75.0);
    assert_eq!(monitor.current_capacity()?, 8000.0);
    assert_eq!(monitor.design_capacity()?, 10000.0);

    Ok(())
}

#[test]
fn test_capacity_monitor_time_remaining() -> Result<()> {
    let battery = TestBatteryBuilder::new().percentage(75.0).time_remaining(7200).build()?;

    let monitor = battery.capacity_monitor()?;
    assert_eq!(monitor.time_remaining()?.unwrap().as_secs(), 7200);

    Ok(())
}
