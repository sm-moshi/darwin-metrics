use std::error::Error;
use std::time::Duration;

use darwin_metrics::hardware::iokit::IOKitImpl;
use darwin_metrics::memory::MemoryMonitor;
use darwin_metrics::{Cpu, CpuMonitor, Gpu, GpuMonitor, Memory, MemoryMonitor};

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
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hardware Monitoring Example");
    println!("==========================");

    // Create hardware monitors
    let iokit = IOKitImpl::new()?;
    let mut cpu = Cpu::new(Box::new(iokit.clone()))?;
    let gpu = Gpu::new()?;
    let memory = Memory::new()?;

    loop {
        println!("\nCPU Information:");
        println!("-----------------");
        println!("Model: {}", cpu.model_name().await?);
        println!("Physical Cores: {}", cpu.physical_cores().await?);
        println!("Logical Cores: {}", cpu.logical_cores().await?);
        println!("Frequency: {:.1} MHz", cpu.frequency().await?);
        println!("Total Usage: {:.1}%", cpu.total_usage().await?);

        let core_usage = cpu.core_usage().await?;
        for (i, usage) in core_usage.iter().enumerate() {
            println!("Core {} Usage: {:.1}%", i, usage);
        }

        if let Some(temp) = cpu.temperature().await? {
            println!("Temperature: {:.1}°C", temp);
        }

        if let Some(power) = cpu.power_consumption().await? {
            println!("Power: {:.1}W", power);
        }

        println!("\nGPU Information:");
        println!("-----------------");
        println!("Name: {}", gpu.name().await?);
        println!("Utilization: {:.1}%", gpu.get_utilization().await?);

        let memory_info = gpu.get_memory().await?;
        println!("Memory Total: {} MB", memory_info.total / 1024 / 1024);
        println!("Memory Used: {} MB", memory_info.used / 1024 / 1024);
        println!("Memory Free: {} MB", memory_info.free / 1024 / 1024);
        println!("Memory Utilization: {:.1}%", gpu.memory_utilization().await?);

        if let Some(temp) = gpu.get_temperature().await? {
            println!("Temperature: {:.1}°C", temp);
        }

        println!("\nMemory Information:");
        println!("-------------------");
        let memory_info = memory.memory_info().await?;
        let page_states = memory.page_states().await?;

        println!("Total: {} MB", memory_info.total / 1024 / 1024);
        println!("Used: {} MB", memory_info.used / 1024 / 1024);
        println!("Free: {} MB", memory_info.free / 1024 / 1024);
        println!("Usage: {:.1}%", memory.usage_percentage().await?);
        println!("Pressure: {:.1}%", memory.pressure_percentage().await?);

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
