use darwin_metrics::hardware::gpu::{GPU, GpuMetrics};
use objc2::rc::autoreleasepool;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;

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
    println!("  Used:  {} ({:.1}%)", 
        format_memory(metrics.memory.used),
        (metrics.memory.used as f64 / metrics.memory.total as f64) * 100.0
    );
    println!("  Free:  {}", format_memory(metrics.memory.free));
    
    if let Some(power) = metrics.power_usage {
        println!("Power Usage: {:.2} W", power);
    } else {
        println!("Power Usage: Not available");
    }
    
    println!(""); // Empty line for readability
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Darwin Metrics - GPU Monitor Example");
    println!("Press Ctrl+C to exit\n");

    // Sample rate in milliseconds
    let sample_rate = Duration::from_millis(1000);
    let mut sample_count = 0;
    let mut gpu_name = String::from("Unknown GPU");
    
    // Main monitoring loop
    loop {
        // Each iteration runs in its own autoreleasepool to ensure Objective-C memory is released
        autoreleasepool(|_| {
            // Initialize the GPU monitoring
            let gpu = match GPU::new() {
                Ok(gpu) => gpu,
                Err(e) => {
                    eprintln!("Failed to initialize GPU monitoring: {}", e);
                    return;
                }
            };
            
            // Get GPU name only on first run
            if gpu_name == "Unknown GPU" {
                match gpu.name() {
                    Ok(name) => {
                        gpu_name = name;
                        println!("Detected GPU: {}\n", gpu_name);
                    },
                    Err(e) => println!("Could not detect GPU name: {}\n", e),
                }
            }
            
            // Clear screen and move cursor to top-left for clean display
            print!("\x1B[2J\x1B[1;1H");
            let _ = io::stdout().flush();
            
            // Get metrics
            match gpu.metrics() {
                Ok(metrics) => {
                    println!("Sample #{}\n", sample_count);
                    display_metrics(&metrics);
                    
                    // Create a simple ASCII graph of utilization
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
                },
                Err(e) => {
                    println!("Error fetching GPU metrics: {}", e);
                }
            }
        });
        
        sample_count += 1;
        sleep(sample_rate);
    }
}