use darwin_metrics::resource::{ResourceMonitor, ResourceMonitoring};
use std::time::Duration;

#[tokio::main]
async fn main() -> darwin_metrics::Result<()> {
    println!("Creating ResourceMonitor...");
    let mut monitor = ResourceMonitor::new(Duration::from_secs(1)).await?;

    println!("Getting updates...");

    // Get 3 updates
    for i in 1..=3 {
        println!("Update {}", i);
        match monitor.next_update().await {
            Ok(update) => {
                println!("  Memory: {} bytes total, {} bytes used", update.memory.total, update.memory.used);
                println!("  CPU temperature: {}°C", update.temperature.cpu);
                println!("  Disk space: {} bytes free", update.disk.free_space);
                println!(
                    "  Network: {} bytes received, {} bytes sent",
                    update.network.received_bytes, update.network.sent_bytes
                );
            },
            Err(e) => {
                println!("Error: {:?}", e);
                break;
            },
        }

        // Wait a bit
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // Stop the monitor
    println!("Stopping monitor...");
    monitor.stop().await?;
    println!("Monitor stopped successfully");

    Ok(())
}
