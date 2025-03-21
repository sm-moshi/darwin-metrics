use std::error::Error;
use std::time::Duration;

use darwin_metrics::hardware::iokit::IOKitImpl;
use darwin_metrics::traits::{CpuMonitor, TemperatureMonitor, UtilizationMonitor};
use darwin_metrics::CPU;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let iokit = Box::new(IOKitImpl::new()?);
    let cpu = CPU::new(iokit);
    
    // Get the specific monitors
    let temp_monitor = cpu.temperature_monitor();
    let util_monitor = cpu.utilization_monitor();
    let freq_monitor = cpu.frequency_monitor();

    println!("CPU Monitoring Example");
    println!("=====================");

    loop {
        // CPU Info
        println!("\nCPU Information:");
        println!("----------------");
        println!("Physical Cores: {}", freq_monitor.physical_cores().await?);
        println!("Logical Cores: {}", freq_monitor.logical_cores().await?);
        println!("CPU Model: {}", freq_monitor.model_name().await?);
        
        // CPU Load
        println!("\nCPU Load:");
        println!("---------");
        println!("Total CPU Load: {:.1}%", util_monitor.utilization().await? * 100.0);
        
        let core_loads = freq_monitor.core_usage().await?;
        println!("\nPer-Core Load:");
        println!("-------------");
        for (i, load) in core_loads.iter().enumerate() {
            println!("Core {}: {:.1}%", i, load * 100.0);
        }
        
        // CPU Frequency
        println!("\nCPU Frequency:");
        println!("-------------");
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
        
        // CPU Temperature
        println!("\nCPU Temperature:");
        println!("---------------");
        match temp_monitor.temperature().await {
            Ok(temp) => println!("Temperature: {:.1}°C", temp),
            Err(_) => println!("Temperature: Not available"),
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
