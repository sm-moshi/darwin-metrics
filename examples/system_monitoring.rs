use std::error::Error;
use std::time::Duration;

use chrono::DateTime;
use darwin_metrics::{ResourceMonitor, System, SystemInfoMonitor, SystemLoadMonitor, SystemUptimeMonitor};

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
    let system = System::new()?;

    println!("System Monitoring Example");
    println!("========================");

    loop {
        // System Info
        println!("\nSystem Information:");
        println!("------------------");
        println!("Hostname: {}", system.hostname().await?);
        println!("OS Version: {}", system.os_version().await?);
        println!("Kernel Version: {}", system.kernel_version().await?);
        println!("Architecture: {}", system.architecture().await?);

        // System Load
        println!("\nSystem Load:");
        println!("------------");
        println!("Load Average (1m): {:.2}", system.load_average_1().await?);
        println!("Load Average (5m): {:.2}", system.load_average_5().await?);
        println!("Load Average (15m): {:.2}", system.load_average_15().await?);
        println!("Process Count: {}", system.process_count().await?);
        println!("Thread Count: {}", system.thread_count().await?);

        // System Uptime
        println!("\nSystem Uptime:");
        println!("-------------");
        let uptime = system.uptime_seconds().await?;
        let days = uptime / 86400;
        let hours = (uptime % 86400) / 3600;
        let minutes = (uptime % 3600) / 60;
        let seconds = uptime % 60;
        println!("Uptime: {}d {}h {}m {}s", days, hours, minutes, seconds);

        let boot_time = system.boot_time().await?;
        if let Some(boot_datetime) = DateTime::from_timestamp(boot_time as i64, 0) {
            println!("Boot Time: {}", boot_datetime);
        }

        // System Resources
        println!("\nSystem Resources:");
        println!("----------------");
        println!("Physical CPU Cores: {}", system.physical_cpu_count().await?);
        println!("Logical CPU Cores: {}", system.logical_cpu_count().await?);
        println!("Total Memory: {} MB", system.total_memory().await? / 1024 / 1024);
        println!("Total Swap: {} MB", system.total_swap().await? / 1024 / 1024);

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
