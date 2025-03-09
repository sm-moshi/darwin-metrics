use std::time::Duration;

use darwin_metrics::network::{NetworkManager, NetworkMetrics};

#[tokio::main]
async fn main() -> darwin_metrics::Result<()> {
    // Create a new network manager asynchronously
    let mut network = NetworkManager::new_async().await?;

    println!("Initial network statistics:");
    for interface in network.interfaces() {
        if interface.is_active() {
            println!("{} ({})", interface.name(), interface.interface_type());
            println!("  Bytes received: {}", interface.bytes_received());
            println!("  Bytes sent: {}", interface.bytes_sent());
        }
    }

    // Sleep asynchronously to allow traffic to occur
    println!("\nWaiting 3 seconds for network activity...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Update statistics asynchronously
    network.update_async().await?;

    // Get active interfaces only
    println!("\nActive interfaces:");
    let active_interfaces = network.active_interfaces_async().await;
    for interface in active_interfaces {
        println!("{} ({})", interface.name(), interface.interface_type());
    }

    // Get total network usage
    let (download, upload) = network.get_throughput_async().await?;
    println!("\nTotal network usage:");
    println!("  Download: {:.2} KB/s", download / 1024.0);
    println!("  Upload: {:.2} KB/s", upload / 1024.0);

    // Check for a specific interface asynchronously
    if let Some(en0) = network.get_interface_async("en0").await {
        println!("\nNetwork stats for en0:");
        println!("  Download: {:.2} KB/s", en0.download_speed() / 1024.0);
        println!("  Upload: {:.2} KB/s", en0.upload_speed() / 1024.0);
    }

    Ok(())
}
