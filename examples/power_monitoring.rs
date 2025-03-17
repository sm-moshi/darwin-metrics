use darwin_metrics::{
    core::metrics::hardware::{PowerConsumptionMonitor, PowerEventMonitor, PowerManagementMonitor, PowerStateMonitor},
    error::Result,
    power::{Power, PowerState},
};
use std::time::Duration;

/// Helper function to format power values in watts
fn format_watts(watts: f32) -> String {
    if watts >= 1000.0 {
        format!("{:.2} kW", watts / 1000.0)
    } else {
        format!("{:.2} W", watts)
    }
}

/// Helper function to format duration in a human-readable format
fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting power monitoring...\n");

    // Create power monitoring instance
    let power = Power::new();

    // Get different monitors
    let consumption_monitor = power.consumption_monitor();
    let state_monitor = power.state_monitor();
    let management_monitor = power.management_monitor();
    let event_monitor = power.event_monitor();

    // Monitor for 30 seconds, updating every 5 seconds
    for i in 0..6 {
        if i > 0 {
            println!("\nWaiting 5 seconds for next update...\n");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }

        // Power Consumption Metrics
        println!("Power Consumption:");
        println!("-----------------");
        println!("Package Power: {}", format_watts(consumption_monitor.package_power().await?));
        println!("CPU Cores Power: {}", format_watts(consumption_monitor.cores_power().await?));
        if let Some(gpu_power) = consumption_monitor.gpu_power().await? {
            println!("GPU Power: {}", format_watts(gpu_power));
        }
        if let Some(dram_power) = consumption_monitor.dram_power().await? {
            println!("Memory Power: {}", format_watts(dram_power));
        }
        if let Some(neural_power) = consumption_monitor.neural_engine_power().await? {
            println!("Neural Engine Power: {}", format_watts(neural_power));
        }
        println!("Total System Power: {}", format_watts(consumption_monitor.total_power().await?));

        // Power State Information
        println!("\nPower State:");
        println!("------------");
        let power_state = state_monitor.power_state().await?;
        println!(
            "Current State: {}",
            match power_state {
                PowerState::Battery => "On Battery",
                PowerState::AC => "On AC Power",
                PowerState::Charging => "Charging",
                PowerState::Unknown => "Unknown",
            }
        );
        if let Some(battery_pct) = state_monitor.battery_percentage().await? {
            println!("Battery Level: {:.1}%", battery_pct);
        }
        if let Some(time_remaining) = state_monitor.time_remaining().await? {
            println!("Time Remaining: {} minutes", time_remaining);
        }
        println!("On Battery: {}", if state_monitor.is_on_battery().await? { "Yes" } else { "No" });
        println!("Charging: {}", if state_monitor.is_charging().await? { "Yes" } else { "No" });

        // Power Management Information
        println!("\nPower Management:");
        println!("----------------");
        println!(
            "Thermal Throttling: {}",
            if management_monitor.is_thermal_throttling().await? { "Yes" } else { "No" }
        );
        if let Some(impact) = management_monitor.power_impact().await? {
            println!("Power Impact Score: {:.1}", impact);
        }
        println!("Thermal Pressure: {}%", management_monitor.thermal_pressure().await?);
        println!("Performance Mode: {}", management_monitor.performance_mode().await?);

        // Power Events
        println!("\nPower Events:");
        println!("-------------");
        println!("Time Since Wake: {}", format_duration(event_monitor.time_since_wake().await?));
        println!("Thermal Events: {}", event_monitor.thermal_event_count().await?);
        if let Some(sleep_time) = event_monitor.time_until_sleep().await? {
            println!("Time Until Sleep: {}", format_duration(sleep_time));
        }
        println!("Sleep Prevention: {}", if event_monitor.is_sleep_prevented().await? { "Active" } else { "Inactive" });

        println!("\n");
    }

    Ok(())
}
