use std::time::Duration;

use darwin_metrics::resource::ResourceMonitor;

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
                println!(
                    "  Memory: {} bytes total, {} bytes used",
                    update.memory.total, update.memory.used
                );
                println!("  CPU temperature: {}°C", update.temperature.0);
                println!("  Disk space: {} bytes free", update.disk.available);
                println!("  Network traffic: {} received, {} sent", "N/A", "N/A");
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
