use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use darwin_metrics::HardwareMonitor;
use darwin_metrics::disk::{DiskIOMonitor, DiskIOMonitorImpl};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Darwin Metrics - Disk Monitor Example");
    println!("Press Ctrl+C to exit\n");

    // Initialize the DiskIOMonitor with root disk
    let disk = darwin_metrics::disk::get_root_disk().await?;
    let disk_monitor = DiskIOMonitorImpl::new(disk.clone());

    println!("Monitoring disk: {}", disk.name);
    println!("Device: {}", disk.device);
    println!("Mount point: {}", disk.mount_point);
    println!("Filesystem: {}", disk.fs_type);
    println!("Total size: {} bytes", disk.total);
    println!("\nPress Ctrl+C to exit\n");

    loop {
        // Get current disk I/O metrics
        let metric = disk_monitor.get_metric().await?;
        let io = metric.value;

        // Calculate transfer rate
        let transfer_rate = DiskIOMonitor::get_transfer_rate(&disk_monitor).await?;

        // Clear the terminal
        print!("\x1B[2J\x1B[1;1H");

        // Print the current metrics
        println!("Disk: {} ({})", disk.name, disk.device);
        println!("Read operations: {}", io.reads);
        println!("Write operations: {}", io.writes);
        println!("Read bytes: {:?}", io.read_bytes);
        println!("Write bytes: {:?}", io.write_bytes);
        println!("Read time: {:?}", io.read_time);
        println!("Write time: {:?}", io.write_time);
        println!("Read rate: {:?} bytes/sec", transfer_rate.read);
        println!("Write rate: {:?} bytes/sec", transfer_rate.write);

        sleep(Duration::from_secs(1));
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
