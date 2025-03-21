use std::error::Error;
use std::time::Duration;

use darwin_metrics::hardware::iokit::IOKitImpl;
use darwin_metrics::traits::{CpuMonitor, MemoryMonitor, UtilizationMonitor};
use darwin_metrics::{CPU, Memory};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize hardware components
    let iokit = Box::new(IOKitImpl::new()?);
    let cpu = CPU::new(iokit);
    let memory = Memory::new()?;
    
    // Get the specific monitors
    let freq_monitor = cpu.frequency_monitor();
    let util_monitor = cpu.utilization_monitor();

    println!("System Monitoring Example");
    println!("========================");

    loop {
        // CPU Information
        println!("\nCPU Information:");
        println!("----------------");
        println!("Physical Cores: {}", freq_monitor.physical_cores().await?);
        println!("Logical Cores: {}", freq_monitor.logical_cores().await?);
        println!("CPU Model: {}", freq_monitor.model_name().await?);
        
        // CPU Performance
        println!("\nCPU Performance:");
        println!("----------------");
        println!("Total CPU Usage: {:.1}%", util_monitor.utilization().await? * 100.0);
        println!("Current Frequency: {:.0} MHz", freq_monitor.frequency().await?);
        
        if let Some(min_freq) = freq_monitor.min_frequency() {
            println!("Min Frequency: {:.0} MHz", min_freq);
        } else {
            println!("Min Frequency: Not available");
        }
        
        if let Some(max_freq) = freq_monitor.max_frequency() {
            println!("Max Frequency: {:.0} MHz", max_freq);
        } else {
            println!("Max Frequency: Not available");
        }
        
        // Per-core usage
        let core_loads = freq_monitor.core_usage().await?;
        println!("\nPer-Core Usage:");
        println!("--------------");
        for (i, load) in core_loads.iter().enumerate() {
            println!("Core {}: {:.1}%", i, load * 100.0);
        }
        
        // Memory Information
        println!("\nMemory Information:");
        println!("------------------");
        println!("Total Memory: {}", format_bytes(memory.total().await?));
        println!("Available Memory: {}", format_bytes(memory.available().await?));
        println!("Used Memory: {}", format_bytes(memory.used().await?));
        println!("Memory Usage: {:.1}%", memory.usage_percentage().await? * 100.0);

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
