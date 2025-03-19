use crate::battery::Battery;
use crate::error::Result;
use crate::tests::common::TestBatteryBuilder;

#[test]
fn test_health_monitor() -> Result<()> {
    let battery = TestBatteryBuilder::new()
        .cycle_count(100)
        .capacity(8000.0, 10000.0)
        .build()?;

    let monitor = battery.health_monitor()?;
    assert_eq!(monitor.cycle_count()?, 100);
    assert_eq!(monitor.health_percentage()?, 80.0); // 8000/10000 * 100

    Ok(())
}

#[test]
fn test_health_monitor_zero_capacity() -> Result<()> {
    let battery = TestBatteryBuilder::new()
        .cycle_count(100)
        .capacity(0.0, 10000.0)
        .build()?;

    let monitor = battery.health_monitor()?;
    assert_eq!(monitor.health_percentage()?, 0.0);

    Ok(())
}
