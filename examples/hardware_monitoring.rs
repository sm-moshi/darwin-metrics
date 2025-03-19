use std::time::Duration;

use darwin_metrics::IOKitImpl;
use darwin_metrics::core::metrics::hardware::{CpuMonitor, GpuMonitor, MemoryMonitor, ThermalMonitor};
use darwin_metrics::core::prelude::{PowerEventMonitor, PowerManagementMonitor};
use darwin_metrics::error::Result;
use darwin_metrics::hardware::{CPU, Gpu, Memory};
use darwin_metrics::temperature::Temperature;
use darwin_metrics::traits::{
    CpuMonitor, GpuMonitor, MemoryMonitor, PowerConsumptionMonitor, PowerEventMonitor, PowerManagementMonitor,
    PowerStateMonitor, SystemResourceMonitor, ThermalMonitor,
};
use tokio::time::sleep;

/// Helper function to format bytes in a human-readable format
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Helper function to format frequency in MHz
fn format_frequency(mhz: f64) -> String {
    if mhz >= 1000.0 {
        format!("{:.2} GHz", mhz / 1000.0)
    } else {
        format!("{:.2} MHz", mhz)
    }
}

fn format_power(watts: f32) -> String {
    format!("{:.2} W", watts)
}

fn format_temperature(celsius: f64) -> String {
    format!("{:.1}°C", celsius)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let monitor = IOKitImpl::new()?;
    println!("Starting hardware monitoring...\n");

    for i in 0..6 {
        if i > 0 {
            sleep(Duration::from_secs(5)).await;
        }
        println!("=== Hardware Monitoring Update {} ===", i + 1);

        // CPU Monitoring
        println!("\nCPU Information:");
        println!("Model: {}", monitor.model_name().await?);
        println!("Physical Cores: {}", monitor.physical_cores().await?);
        println!("Logical Cores: {}", monitor.logical_cores().await?);
        println!("Current Frequency: {:.2} GHz", monitor.frequency().await? / 1000.0);
        println!("Min Frequency: {:.2} GHz", monitor.min_frequency().await? / 1000.0);
        println!("Max Frequency: {:.2} GHz", monitor.max_frequency().await? / 1000.0);

        if let Some(temp) = monitor.temperature().await? {
            println!("CPU Temperature: {}", format_temperature(temp));
        }

        let core_usage = monitor.core_usage().await?;
        println!("Core Usage:");
        for (i, usage) in core_usage.iter().enumerate() {
            println!("  Core {}: {:.1}%", i, usage);
        }
        println!("Total CPU Usage: {:.1}%", monitor.total_usage().await?);

        // GPU Monitoring
        println!("\nGPU Information:");
        println!("Name: {}", monitor.name().await?);
        println!("Utilization: {:.1}%", monitor.utilization().await?);
        if let Some(temp) = monitor.temperature().await? {
            println!("GPU Temperature: {}", format_temperature(temp as f64));
        }
        println!("Total Memory: {}", format_bytes(monitor.total_memory().await?));
        println!("Used Memory: {}", format_bytes(monitor.used_memory().await?));
        println!("Free Memory: {}", format_bytes(monitor.free_memory().await?));
        println!("Memory Utilization: {:.1}%", monitor.memory_utilization().await?);
        println!(
            "Hardware Acceleration: {}",
            if monitor.supports_hardware_acceleration().await? {
                "Supported"
            } else {
                "Not Supported"
            }
        );
        if let Some(bandwidth) = monitor.memory_bandwidth().await? {
            println!("Memory Bandwidth: {} GB/s", bandwidth as f64 / 1024.0 / 1024.0 / 1024.0);
        }

        // Thermal Monitoring
        println!("\nThermal Information:");
        if let Some(temp) = monitor.cpu_temperature().await? {
            println!("CPU Temperature: {}", format_temperature(temp));
        }
        if let Some(temp) = monitor.gpu_temperature().await? {
            println!("GPU Temperature: {}", format_temperature(temp));
        }
        if let Some(temp) = monitor.memory_temperature().await? {
            println!("Memory Temperature: {}", format_temperature(temp));
        }
        if let Some(temp) = monitor.battery_temperature().await? {
            println!("Battery Temperature: {}", format_temperature(temp));
        }
        if let Some(temp) = monitor.ambient_temperature().await? {
            println!("Ambient Temperature: {}", format_temperature(temp));
        }
        println!(
            "Thermal Throttling: {}",
            if monitor.is_throttling().await? { "Yes" } else { "No" }
        );

        let fans = monitor.get_fans().await?;
        println!("\nFan Information:");
        for (i, fan) in fans.iter().enumerate() {
            println!("Fan {}:", i + 1);
            println!("  Current Speed: {:.0} RPM", fan.speed);
            println!("  Min Speed: {:.0} RPM", fan.min_speed);
            println!("  Max Speed: {:.0} RPM", fan.max_speed);
        }

        // Power Consumption Monitoring
        println!("\nPower Consumption:");
        println!("Package Power: {}", format_power(monitor.package_power().await?));
        println!("Cores Power: {}", format_power(monitor.cores_power().await?));
        if let Some(power) = monitor.gpu_power().await? {
            println!("GPU Power: {}", format_power(power));
        }
        if let Some(power) = monitor.dram_power().await? {
            println!("DRAM Power: {}", format_power(power));
        }
        if let Some(power) = monitor.neural_engine_power().await? {
            println!("Neural Engine Power: {}", format_power(power));
        }
        println!("Total Power: {}", format_power(monitor.total_power().await?));

        // Power State Monitoring
        println!("\nPower State:");
        println!("Current State: {:?}", monitor.power_state().await?);
        if let Some(percentage) = monitor.battery_percentage().await? {
            println!("Battery Level: {:.1}%", percentage);
        }
        if let Some(time) = monitor.time_remaining().await? {
            println!("Time Remaining: {} minutes", time);
        }
        println!("On Battery: {}", monitor.is_on_battery().await?);
        println!("Charging: {}", monitor.is_charging().await?);

        // Power Management
        println!("\nPower Management:");
        println!("Thermal Throttling: {}", monitor.is_thermal_throttling().await?);
        if let Some(impact) = monitor.power_impact().await? {
            println!("Power Impact: {:.1}", impact);
        }
        println!("Thermal Pressure: {}", monitor.thermal_pressure().await?);
        println!("Performance Mode: {}", monitor.performance_mode().await?);

        // Power Events
        println!("\nPower Events:");
        println!(
            "Time Since Wake: {:.1} seconds",
            monitor.time_since_wake().await?.as_secs_f64()
        );
        println!("Thermal Event Count: {}", monitor.thermal_event_count().await?);
        if let Some(sleep_time) = monitor.time_until_sleep().await? {
            println!("Time Until Sleep: {:.1} seconds", sleep_time.as_secs_f64());
        }
        println!("Sleep Prevention: {}", monitor.is_sleep_prevented().await?);

        println!("\n{}", "=".repeat(50));
    }

    Ok(())
}
