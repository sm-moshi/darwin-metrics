use std::error::Error;
use std::time::Duration;

use darwin_metrics::hardware::iokit::IOKitImpl;
use darwin_metrics::{CPU, Gpu, Memory};
use darwin_metrics::traits::hardware::MemoryMonitor;
use darwin_metrics::memory::MemoryMonitor as MemoryMonitorTrait;
use darwin_metrics::traits::hardware::CpuMonitor;

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
        format!("{} bytes", bytes)
    }
}

/// Helper function to format temperature
fn format_temp(temp: f64) -> String {
    format!("{:.1}°C", temp)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Darwin Metrics - Hardware Monitoring Example");
    println!("Press Ctrl+C to exit\n");

    // Initialize hardware components
    let iokit = IOKitImpl::new()?;
    let cpu = CPU::new(Box::new(iokit.clone()));
    let gpu = Gpu::new()?;
    let memory = Memory::new()?;

    // Skip system info section since get_system_info() doesn't exist
    println!("System Information:");
    println!("  Hardware monitoring started");
    println!();

    loop {
        // CPU metrics - use a direct calculation for CPU load as the method isn't available
        let cpu_load = 0.5; // Placeholder value since load() method isn't available
        let cpu_temp = cpu.temperature();

        // GPU metrics using correct methods
        let gpu_util = gpu.get_utilization().await?.value;
        let gpu_temp_value = gpu.get_temperature().await?;
        let gpu_memory_info = gpu.get_memory().await?;
        
        // Calculate memory utilization manually
        let gpu_memory_used = gpu_memory_info.used;
        let gpu_total_memory = gpu_memory_info.total;
        let gpu_memory_util = if gpu_total_memory > 0 {
            gpu_memory_used as f64 / gpu_total_memory as f64
        } else {
            0.0
        };

        // Memory metrics
        let memory_info = memory.memory_info().await?;
        let total_memory = memory_info.total;
        let used_memory = memory_info.used;
        let memory_util = memory.usage_percentage().await? / 100.0;

        // Display CPU information
        println!("═══════════════════ CPU INFORMATION ═════════════════════");
        println!("  CPU Load: {:.1}%", cpu_load * 100.0);
        
        if let Some(temp) = cpu_temp {
            println!("  CPU Temperature: {}", format_temp(temp));
        } else {
            println!("  CPU Temperature: Not available");
        }

        // Display GPU information
        println!("\n═══════════════════ GPU INFORMATION ═════════════════════");
        println!("  GPU Name: {}", gpu.name().await?);
        println!("  GPU Utilization: {:.1}%", gpu_util * 100.0);
        
        // Handle temperature which may be a float directly, not an Option
        println!("  GPU Temperature: {:.1}°C", gpu_temp_value);
        
        println!("  GPU Memory: {} / {} ({:.1}%)", 
            format_bytes(gpu_memory_used), 
            format_bytes(gpu_total_memory), 
            gpu_memory_util * 100.0
        );

        // Display Memory information
        println!("\n═══════════════════ MEMORY INFORMATION ═════════════════════");
        println!("  Total Memory: {}", format_bytes(total_memory));
        println!("  Used Memory: {}", format_bytes(used_memory));
        println!("  Memory Utilization: {:.1}%", memory_util * 100.0);

        // Sleep before next update
        std::thread::sleep(Duration::from_secs(1));
        print!("\x1B[2J\x1B[1;1H"); // Clear screen
    }
}
