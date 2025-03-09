use std::{
    io::{self, Write},
    thread::sleep,
    time::Duration,
};

use darwin_metrics::hardware::gpu::{Gpu, GpuMetrics};

// Format memory size in a human-readable way
fn format_memory(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} bytes", bytes)
    }
}

fn display_metrics(metrics: &GpuMetrics) {
    println!("GPU: {}", metrics.name);
    println!("Utilization: {:.1}%", metrics.utilization);

    if let Some(temp) = metrics.temperature {
        println!("Temperature: {:.1}°C", temp);
    } else {
        println!("Temperature: Not available");
    }

    println!("Memory:");
    println!("  Total: {}", format_memory(metrics.memory.total));
    println!(
        "  Used:  {} ({:.1}%)",
        format_memory(metrics.memory.used),
        (metrics.memory.used as f64 / metrics.memory.total as f64) * 100.0
    );
    println!("  Free:  {}", format_memory(metrics.memory.free));

    println!(""); // Empty line for readability
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Darwin Metrics - GPU Monitor Example");
    println!("Using simplified, robust GPU monitoring");
    println!("Press Ctrl+C to exit\n");

    // Sample rate in milliseconds
    let sample_rate = Duration::from_millis(1000);
    let mut sample_count = 0;

    // Get GPU info only once to reduce overhead
    let gpu = Gpu::new()?;
    let gpu_name = gpu.name()?;
    println!("Detected GPU: {}\n", gpu_name);

    // Main monitoring loop with 10 second timeout
    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(10);

    loop {
        // Check if we've exceeded the 10 second timeout
        if start_time.elapsed() >= timeout {
            println!("\nTest completed after 10 seconds");
            break Ok(());
        }

        // Clear screen
        print!("\x1B[2J\x1B[1;1H");
        let _ = io::stdout().flush();

        println!("GPU Monitor - Sample #{}\n", sample_count);

        // Get real metrics directly from the GPU
        let metrics = gpu.metrics()?;
        display_metrics(&metrics);

        // Create a visual bar for GPU usage
        let graph_width = 50;
        let filled_chars = (metrics.utilization as usize * graph_width) / 100;
        let empty_chars = graph_width - filled_chars;

        print!("Utilization: [");
        for _ in 0..filled_chars {
            print!("#");
        }
        for _ in 0..empty_chars {
            print!(" ");
        }
        println!("] {:.1}%", metrics.utilization);

        // Memory usage graph
        let memory_percentage = (metrics.memory.used as f64 / metrics.memory.total as f64) * 100.0;
        let filled_chars = (memory_percentage as usize * graph_width) / 100;
        let empty_chars = graph_width - filled_chars;

        print!("Memory:      [");
        for _ in 0..filled_chars {
            print!("#");
        }
        for _ in 0..empty_chars {
            print!(" ");
        }
        println!("] {:.1}%", memory_percentage);

        println!("\nPress Ctrl+C to exit");

        sample_count += 1;
        sleep(sample_rate);
    }
}
