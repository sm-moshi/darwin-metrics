use std::error::Error;
use std::time::Duration;

use darwin_metrics::{CpuFrequencyMonitor, CpuInfoMonitor, CpuLoadMonitor, CpuTemperatureMonitor, System};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let system = System::new()?;

    println!("CPU Monitoring Example");
    println!("=====================");

    loop {
        // CPU Info
        println!("\nCPU Information:");
        println!("--------------");
        println!("CPU Model: {}", system.cpu_model().await?);
        println!("CPU Brand: {}", system.cpu_brand().await?);
        println!("CPU Family: {}", system.cpu_family().await?);
        println!("CPU Vendor: {}", system.cpu_vendor().await?);
        println!("Physical Cores: {}", system.physical_cpu_count().await?);
        println!("Logical Cores: {}", system.logical_cpu_count().await?);

        // CPU Load
        println!("\nCPU Load:");
        println!("---------");
        let cpu_load = system.cpu_load().await?;
        println!("Total CPU Load: {:.1}%", cpu_load.total * 100.0);
        println!("User Load: {:.1}%", cpu_load.user * 100.0);
        println!("System Load: {:.1}%", cpu_load.system * 100.0);
        println!("Idle: {:.1}%", cpu_load.idle * 100.0);
        println!("Nice: {:.1}%", cpu_load.nice * 100.0);

        // Per-Core Load
        println!("\nPer-Core Load:");
        println!("-------------");
        let core_loads = system.cpu_load_per_core().await?;
        for (i, load) in core_loads.iter().enumerate() {
            println!(
                "Core {}: {:.1}% (User: {:.1}%, System: {:.1}%, Idle: {:.1}%)",
                i,
                load.total * 100.0,
                load.user * 100.0,
                load.system * 100.0,
                load.idle * 100.0
            );
        }

        // CPU Temperature
        println!("\nCPU Temperature:");
        println!("---------------");
        if let Ok(temp) = system.cpu_temperature().await {
            println!("CPU Temperature: {:.1}°C", temp);
        } else {
            println!("CPU Temperature: Not available");
        }

        // CPU Frequency
        println!("\nCPU Frequency:");
        println!("--------------");
        println!("Current Frequency: {} MHz", system.cpu_frequency().await?);
        println!("Min Frequency: {} MHz", system.cpu_min_frequency().await?);
        println!("Max Frequency: {} MHz", system.cpu_max_frequency().await?);

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
