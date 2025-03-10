use darwin_metrics::hardware::gpu::Gpu;

/// Demonstrates the improved GPU hardware detection in darwin-metrics
/// This example shows detailed information about the GPU including hardware
/// characteristics and estimated performance metrics.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Darwin Metrics - Enhanced GPU Information");
    println!("=========================================");

    // Initialize the GPU module
    let gpu = Gpu::new()?;

    // Get GPU metrics with enhanced hardware detection
    let metrics = gpu.metrics()?;

    // Display basic GPU information
    println!("GPU Model: {}", metrics.name);
    println!();

    // Display GPU characteristics
    println!("Hardware Characteristics:");
    println!("------------------------");
    println!(
        "Architecture: {}",
        if metrics.characteristics.is_apple_silicon {
            "Apple Silicon GPU"
        } else if metrics.characteristics.is_integrated {
            "Integrated GPU"
        } else {
            "Discrete GPU"
        }
    );

    if let Some(cores) = metrics.characteristics.core_count {
        println!("GPU Cores: {}", cores);
    } else {
        println!("GPU Cores: Unknown");
    }

    if let Some(clock_speed) = metrics.characteristics.clock_speed_mhz {
        println!("Clock Speed: {} MHz", clock_speed);
    } else {
        println!("Clock Speed: Unknown");
    }

    println!(
        "Hardware Raytracing: {}",
        if metrics.characteristics.has_raytracing { "Yes" } else { "No" }
    );
    println!();

    // Display memory information with proper formatting
    println!("Memory Information:");
    println!("------------------");
    println!("Total Memory: {}", format_bytes(metrics.memory.total));
    println!(
        "Used Memory: {} ({:.1}%)",
        format_bytes(metrics.memory.used),
        (metrics.memory.used as f64 / metrics.memory.total as f64) * 100.0
    );
    println!("Free Memory: {}", format_bytes(metrics.memory.free));
    println!();

    // Display performance metrics
    println!("Performance Metrics:");
    println!("-------------------");
    println!("Utilization: {:.1}%", metrics.utilization);

    if let Some(temp) = metrics.temperature {
        println!("Temperature: {:.1}°C", temp);
    } else {
        println!("Temperature: Not available");
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
        format!("{} bytes", bytes)
    }
}
