use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;

use darwin_metrics::disk::DiskMonitor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Darwin Metrics - Disk Monitor Example");
    println!("Press Ctrl+C to exit\n");

    // Initialize the DiskMonitor
    let mut monitor = DiskMonitor::new();

    // Sample rate in milliseconds
    let sample_rate = Duration::from_millis(1000);
    let mut sample_count = 0;

    // Main monitoring loop
    loop {
        // Clear screen and move cursor to top-left for clean display
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush()?;

        println!("Sample #{}\n", sample_count);

        // Display disk volumes information
        match monitor.get_volumes() {
            Ok(volumes) => {
                println!("═════════════════════ DISK VOLUMES ═════════════════════");
                for (i, volume) in volumes.iter().enumerate() {
                    println!("Disk #{}: {}", i + 1, volume.summary());

                    // Create a simple ASCII graph for disk usage
                    let graph_width = 50;
                    let usage_percent = volume.usage_percentage();
                    println!("{}", create_ascii_graph(usage_percent, graph_width));

                    // Additional details
                    println!("  Device: {}", volume.device);
                    println!("  Mount point: {}", volume.mount_point);
                    println!("  Filesystem: {}", volume.fs_type);
                    println!("  Available: {}", volume.available_display());
                    println!("  Boot volume: {}", if volume.is_boot_volume { "Yes" } else { "No" });
                    println!();
                }
            },
            Err(e) => {
                println!("Error fetching disk volumes: {}", e);
            },
        }

        // Display disk performance metrics
        match monitor.get_performance() {
            Ok(performance) => {
                println!("═══════════════════ DISK PERFORMANCE ════════════════════");
                for (device, perf) in performance.iter() {
                    println!("Device: {}", device);
                    println!(
                        "  Read: {:.1} ops/s, {}/s",
                        perf.reads_per_second,
                        format_bytes(perf.bytes_read_per_second)
                    );
                    println!(
                        "  Write: {:.1} ops/s, {}/s",
                        perf.writes_per_second,
                        format_bytes(perf.bytes_written_per_second)
                    );
                    println!(
                        "  Latency: {:.2} ms read, {:.2} ms write",
                        perf.read_latency_ms, perf.write_latency_ms
                    );
                    println!("  Utilization: {:.1}%", perf.utilization);

                    // Create a simple ASCII graph for disk utilization
                    let graph_width = 50;
                    println!("{}", create_ascii_graph(perf.utilization, graph_width));
                    println!();
                }
            },
            Err(e) => {
                println!("Error fetching disk performance: {}", e);
            },
        }

        println!("Press Ctrl+C to exit");

        sample_count += 1;
        sleep(sample_rate);
    }
}

// Helper function to convert bytes to human-readable format
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

// Helper function to create ASCII graph
fn create_ascii_graph(value: f64, max: usize) -> String {
    let filled_chars = (value as usize * max) / 100;
    let empty_chars = max - filled_chars;

    let mut graph = String::from("[");
    graph.push_str(&"#".repeat(filled_chars));
    graph.push_str(&" ".repeat(empty_chars));
    graph.push_str(&format!(" ] {:.1}%%", value));
    graph
}
