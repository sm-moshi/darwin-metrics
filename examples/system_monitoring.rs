use std::error::Error;
use std::time::Duration;

use darwin_metrics::core::metrics::hardware::{
    SystemInfoMonitor, SystemLoadMonitor, SystemResourceMonitor, SystemUptimeMonitor,
};
use darwin_metrics::system::System;
use darwin_metrics::traits::SystemMonitor;

/// Helper function to format bytes into human-readable strings
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut bytes = bytes as f64;
    let mut unit_index = 0;

    while bytes >= 1024.0 && unit_index < UNITS.len() - 1 {
        bytes /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", bytes, UNITS[unit_index])
}

/// Helper function to format duration into human-readable string
fn format_duration(seconds: u64) -> String {
    let days = seconds / (24 * 3600);
    let hours = (seconds % (24 * 3600)) / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a new system monitor
    let system = System::new()?;

    println!("Starting system monitoring...\n");

    // Monitor for 30 seconds
    for _ in 0..6 {
        let info_monitor = system.info_monitor();
        let load_monitor = system.load_monitor();
        let uptime_monitor = system.uptime_monitor();
        let resource_monitor = system.resource_monitor();

        // System Information
        println!("System Information:");
        println!("===================");
        println!("Hostname: {}", info_monitor.hostname().await?);
        println!("Architecture: {}", info_monitor.architecture().await?);
        println!("OS Version: {}", info_monitor.os_version().await?);
        println!("Kernel Version: {}", info_monitor.kernel_version().await?);

        // System Load
        println!("\nSystem Load:");
        println!("===================");
        println!("Load Average (1 min): {:.2}", load_monitor.load_average_1().await?);
        println!("Load Average (5 min): {:.2}", load_monitor.load_average_5().await?);
        println!("Load Average (15 min): {:.2}", load_monitor.load_average_15().await?);
        println!("Process Count: {}", load_monitor.process_count().await?);
        println!("Thread Count: {}", load_monitor.thread_count().await?);

        // System Uptime
        println!("\nSystem Uptime:");
        println!("===================");
        let uptime = uptime_monitor.uptime_seconds().await?;
        let boot_time = uptime_monitor.boot_time().await?;
        println!("Uptime: {}", format_duration(uptime));
        println!(
            "Boot Time: {}",
            chrono::NaiveDateTime::from_timestamp_opt(boot_time as i64, 0)
                .unwrap_or_default()
                .format("%Y-%m-%d %H:%M:%S")
        );

        // System Resources
        println!("\nSystem Resources:");
        println!("===================");
        println!("Physical CPU Cores: {}", resource_monitor.physical_cpu_count().await?);
        println!("Logical CPU Cores: {}", resource_monitor.logical_cpu_count().await?);
        println!("Total Memory: {}", format_bytes(resource_monitor.total_memory().await?));
        println!("Total Swap: {}", format_bytes(resource_monitor.total_swap().await?));

        println!("\nWaiting 5 seconds for next update...\n");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}
