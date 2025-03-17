use darwin_metrics::{
    core::metrics::hardware::{
        ProcessIOMonitor, ProcessInfoMonitor, ProcessRelationshipMonitor, ProcessResourceMonitor,
    },
    error::Result,
    process::Process,
};
use std::time::Duration;

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

/// Helper function to format rate in bytes per second
fn format_rate(bytes_per_sec: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bytes_per_sec >= GB {
        format!("{:.2} GB/s", bytes_per_sec / GB)
    } else if bytes_per_sec >= MB {
        format!("{:.2} MB/s", bytes_per_sec / MB)
    } else if bytes_per_sec >= KB {
        format!("{:.2} KB/s", bytes_per_sec / KB)
    } else {
        format!("{:.2} B/s", bytes_per_sec)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting process monitoring...\n");

    // Get all running processes
    let processes = Process::get_all()?;
    println!("Found {} processes\n", processes.len());

    // Monitor each process for 30 seconds, updating every 5 seconds
    for i in 0..6 {
        if i > 0 {
            println!("\nWaiting 5 seconds for next update...\n");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }

        // Sort processes by CPU usage
        let mut processes = Process::get_all()?;
        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());

        // Display top 5 processes by CPU usage
        println!("Top 5 Processes by CPU Usage:");
        println!("-----------------------------");
        for process in processes.iter().take(5) {
            let info_monitor = process.info_monitor();
            let resource_monitor = process.resource_monitor();
            let io_monitor = process.io_monitor();
            let relationship_monitor = process.relationship_monitor();

            // Process Information
            println!("\nProcess: {} (PID: {})", info_monitor.name().await?, info_monitor.pid().await?);
            if let Ok(Some(ppid)) = info_monitor.parent_pid().await {
                println!("Parent PID: {}", ppid);
            }
            println!("Start Time: {:?}", info_monitor.start_time().await?);
            println!("System Process: {}", if info_monitor.is_system_process().await? { "Yes" } else { "No" });

            // Resource Usage
            println!("\nResource Usage:");
            println!("CPU Usage: {:.1}%", resource_monitor.cpu_usage().await?);
            println!("Memory Usage: {}", format_bytes(resource_monitor.memory_usage().await?));
            println!("Thread Count: {}", resource_monitor.thread_count().await?);
            println!("Suspended: {}", if resource_monitor.is_suspended().await? { "Yes" } else { "No" });

            // I/O Operations
            println!("\nI/O Statistics:");
            println!("Total Read: {}", format_bytes(io_monitor.bytes_read().await?));
            println!("Total Written: {}", format_bytes(io_monitor.bytes_written().await?));
            println!("Read Operations: {}", io_monitor.read_operations().await?);
            println!("Write Operations: {}", io_monitor.write_operations().await?);
            println!("Read Rate: {}", format_rate(io_monitor.read_rate().await?));
            println!("Write Rate: {}", format_rate(io_monitor.write_rate().await?));

            // Process Relationships
            println!("\nProcess Relationships:");
            println!("Child Processes: {}", relationship_monitor.child_pids().await?.len());
            println!("Sibling Processes: {}", relationship_monitor.sibling_pids().await?.len());
            println!("Tree Depth: {}", relationship_monitor.tree_depth().await?);
            println!("Process Group: {}", relationship_monitor.process_group_id().await?);

            println!("\n{}", "-".repeat(50));
        }

        // System-wide statistics
        let total_processes = processes.len();
        let system_processes = processes.iter().filter(|p| p.is_system_process()).count();
        let user_processes = total_processes - system_processes;

        println!("\nSystem-wide Statistics:");
        println!("----------------------");
        println!("Total Processes: {}", total_processes);
        println!("System Processes: {}", system_processes);
        println!("User Processes: {}", user_processes);

        // Memory statistics
        let total_memory: u64 = processes.iter().map(|p| p.memory_usage).sum();
        println!("Total Memory Used: {}", format_bytes(total_memory));

        // Thread statistics
        let total_threads: u32 = processes.iter().map(|p| p.thread_count).sum();
        println!("Total Threads: {}", total_threads);
        println!("Average Threads per Process: {:.1}", total_threads as f64 / total_processes as f64);
    }

    Ok(())
}
