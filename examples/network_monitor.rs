//! Network Interface Monitor Example
//!
//! This example demonstrates how to use darwin-metrics to track and monitor
//! network interfaces and traffic statistics.
//!
//! It displays interfaces and periodically updates traffic metrics to show
//! upload/download speeds.

use darwin_metrics::network::{NetworkManager, NetworkMetrics};
use std::{thread, time::Duration};

fn main() -> darwin_metrics::error::Result<()> {
    // Create a new network manager
    let mut manager = NetworkManager::new()?;

    println!("Initializing network monitoring...");

    // Get initial network state
    manager.update()?;

    println!("\nCurrent interfaces:");
    for interface in manager.interfaces() {
        println!(
            "  - {} ({}): {}",
            interface.name(),
            interface.interface_type(),
            if interface.is_active() { "Active" } else { "Inactive" }
        );

        // Display interface details
        if let Some(mac) = interface.mac_address() {
            println!("    MAC: {}", mac);
        }

        if let Some(addrs) = interface.addresses() {
            for addr in addrs {
                println!("    IP: {}", addr);
            }
        }
    }

    // Monitor loop - update every 2 seconds
    for i in 1..=5 {
        // Wait before updating again
        thread::sleep(Duration::from_secs(2));

        // Update network statistics
        manager.update()?;

        println!("\n[Update {}] Traffic Statistics:", i);

        // Get aggregate network traffic
        let total_download = manager.total_download_speed() / 1024.0; // KB/s
        let total_upload = manager.total_upload_speed() / 1024.0; // KB/s

        println!("Total Traffic: {:.2} KB/s down, {:.2} KB/s up", total_download, total_upload);

        // Display per-interface statistics
        println!("\nInterface Statistics:");
        println!(
            "{:<10} {:<15} {:<15} {:<10} {:<10}",
            "Interface", "Download (KB/s)", "Upload (KB/s)", "RX Errors", "TX Errors"
        );
        println!("{}", "-".repeat(65));

        for interface in manager.interfaces() {
            if interface.is_active() {
                println!(
                    "{:<10} {:<15.2} {:<15.2} {:<10} {:<10}",
                    interface.name(),
                    interface.download_speed() / 1024.0,
                    interface.upload_speed() / 1024.0,
                    interface.receive_errors(),
                    interface.send_errors()
                );
            }
        }
    }

    println!("\nNetwork monitoring complete.");
    Ok(())
}
