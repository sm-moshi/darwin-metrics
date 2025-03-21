use std::error::Error;

use darwin_metrics::Gpu;
use darwin_metrics::gpu::{GpuCharacteristics, GpuMetrics};

/// Demonstrates the improved GPU hardware detection in darwin-metrics
/// This example shows detailed information about the GPU including hardware
/// characteristics and estimated performance metrics.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Darwin Metrics - Enhanced GPU Information");
    println!("=========================================");

    // Initialize the GPU module
    let gpu = Gpu::new()?;

    // Get GPU metrics with enhanced hardware detection
    let metrics = gpu.get_metrics().await?;
    let characteristics = gpu.get_characteristics().await?;

    // Display basic GPU information
    println!("GPU Model: {}", gpu.name().await?);
    println!();

    // Display GPU characteristics
    println!("Hardware Characteristics:");
    println!("------------------------");
    println!(
        "Architecture: {}",
        if characteristics.is_apple_silicon {
            "Apple Silicon GPU"
        } else if characteristics.is_integrated {
            "Integrated GPU"
        } else {
            "Discrete GPU"
        }
    );

    if let Some(cores) = characteristics.core_count {
        println!("GPU Cores: {}", cores);
    } else {
        println!("GPU Cores: Unknown");
    }

    if let Some(clock_speed) = characteristics.clock_speed_mhz {
        println!("Clock Speed: {} MHz", clock_speed);
    } else {
        println!("Clock Speed: Unknown");
    }

    println!(
        "Hardware Raytracing: {}",
        if characteristics.has_raytracing { "Yes" } else { "No" }
    );
    println!();

    // Display memory information
    display_memory_info(&metrics);

    // Display performance metrics
    println!("Performance Metrics:");
    println!("-------------------");
    println!("Utilization: {:.1}%", metrics.utilization * 100.0);

    if metrics.temperature > 0.0 {
        println!("Temperature: {:.1}°C", metrics.temperature);
    } else {
        println!("Temperature: Not available");
    }

    if let Some(power) = metrics.power_usage {
        println!("Power Usage: {:.1}W", power);
    }

    println!();
    println!("Note: Support for multiple GPUs will be added in a future release (post-1.0.0)");

    Ok(())
}

/// Formats bytes into a human-readable string with appropriate units
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

/// Formats memory metrics for display
fn display_memory_info(metrics: &GpuMetrics) {
    println!("Memory Information:");
    println!("------------------");
    println!("Total Memory: {}", format_bytes(metrics.memory_total));
    println!("Used Memory: {}", format_bytes(metrics.memory_used));
    println!(
        "Memory Utilization: {:.1}%",
        (metrics.memory_used as f64 / metrics.memory_total as f64) * 100.0
    );
    println!(
        "Free Memory: {}",
        format_bytes(metrics.memory_total - metrics.memory_used)
    );
    println!();
}
