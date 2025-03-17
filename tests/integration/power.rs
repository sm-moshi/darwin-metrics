use darwin_metrics::{
    error::Result,
    power::{Power, PowerState},
};

#[test]
fn test_power_integration() -> Result<()> {
    let power = Power::new();

    // Test power consumption
    let consumption = power.get_power_consumption()?;

    // Basic checks
    assert!(consumption.package >= 0.0, "Package power should be non-negative");
    assert!(consumption.cores >= 0.0, "Core power should be non-negative");

    // Power state should be one of the valid states
    assert!(matches!(consumption.power_state, PowerState::AC | PowerState::Battery | PowerState::Charging));

    // Test power throttling
    let is_throttling = power.is_power_throttling()?;
    // Just check that we get a boolean result, don't assert its value
    // as it depends on the system state
    let _ = is_throttling;

    Ok(())
}

// This test is marked as ignored because it requires specific hardware
// and might fail in CI environments
#[test]
#[ignore = "This test needs specific hardware support"]
fn test_power_monitors() -> Result<()> {
    let power = Power::new();

    // Test consumption monitor
    let consumption_monitor = power.consumption_monitor();
    let _ = consumption_monitor.current_consumption()?;

    // Test state monitor
    let state_monitor = power.state_monitor();
    let _ = state_monitor.power_state()?;

    // Test management monitor
    let management_monitor = power.management_monitor();
    let _ = management_monitor.is_thermal_throttling()?;
    let _ = management_monitor.power_impact()?;

    // Test event monitor
    let event_monitor = power.event_monitor();
    let _ = event_monitor.time_since_wake()?;

    Ok(())
}
