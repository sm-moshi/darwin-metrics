use std::time::Duration;

use darwin_metrics::error::Result;
use darwin_metrics::network::NetworkManager;
use darwin_metrics::network::interface::{Interface, NetworkInterface};
use darwin_metrics::network::monitors::{
    NetworkBandwidthMonitor, NetworkErrorMonitor, NetworkInterfaceMonitor, NetworkPacketMonitor,
};

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

/// Helper function to format rate in a human-readable format
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
    println!("Starting network monitoring...\n");

    // Initialize network manager
    let mut network = NetworkManager::new()?;

    // Monitor network for 30 seconds, updating every 5 seconds
    for i in 0..6 {
        if i > 0 {
            println!("\nWaiting 5 seconds for next update...\n");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }

        // Update network statistics
        network.update()?;

        // Get all interfaces
        let interfaces = network.interfaces();
        println!("Found {} network interfaces\n", interfaces.len());

        // Sort interfaces by name instead of download speed since we can't use async in sort_by
        let mut interfaces = interfaces.to_vec();
        interfaces.sort_by(|a, b| a.name().cmp(&b.name()));

        // Display information for each interface
        for interface in interfaces {
            let interface_monitor = interface.interface_monitor();
            let bandwidth_monitor = interface.bandwidth_monitor();
            let packet_monitor = interface.packet_monitor();
            let error_monitor = interface.error_monitor();

            println!(
                "Interface: {} ({})",
                interface.name(),
                interface_monitor.interface_type().await?
            );
            println!(
                "Status: {}",
                if interface_monitor.is_active().await? {
                    "Active"
                } else {
                    "Inactive"
                }
            );

            // Interface properties
            if let Some(mac) = interface_monitor.mac_address().await? {
                println!("MAC Address: {}", mac);
            }
            println!(
                "Type: {}",
                if interface_monitor.is_wireless().await? {
                    "Wireless"
                } else {
                    "Wired"
                }
            );
            println!("Supports Broadcast: {}", interface_monitor.supports_broadcast().await?);
            println!("Supports Multicast: {}", interface_monitor.supports_multicast().await?);
            println!("Is Loopback: {}", interface_monitor.is_loopback().await?);

            // Bandwidth metrics
            println!("\nBandwidth Metrics:");
            println!(
                "Total Received: {}",
                format_bytes(bandwidth_monitor.bytes_received().await?)
            );
            println!("Total Sent: {}", format_bytes(bandwidth_monitor.bytes_sent().await?));
            println!(
                "Download Speed: {}",
                format_rate(bandwidth_monitor.download_speed().await?)
            );
            println!("Upload Speed: {}", format_rate(bandwidth_monitor.upload_speed().await?));

            // Packet metrics
            println!("\nPacket Metrics:");
            println!("Packets Received: {}", packet_monitor.packets_received().await?);
            println!("Packets Sent: {}", packet_monitor.packets_sent().await?);
            println!(
                "Packet Receive Rate: {:.2} packets/s",
                packet_monitor.packet_receive_rate().await?
            );
            println!(
                "Packet Send Rate: {:.2} packets/s",
                packet_monitor.packet_send_rate().await?
            );

            // Error metrics
            println!("\nError Metrics:");
            println!("Receive Errors: {}", error_monitor.receive_errors().await?);
            println!("Send Errors: {}", error_monitor.send_errors().await?);
            println!("Collisions: {}", error_monitor.collisions().await?);
            println!(
                "Receive Error Rate: {:.2}%",
                error_monitor.receive_error_rate().await? * 100.0
            );
            println!(
                "Send Error Rate: {:.2}%",
                error_monitor.send_error_rate().await? * 100.0
            );

            println!("\n{}", "-".repeat(50));
        }

        // Display system-wide network statistics
        println!("\nSystem-wide Network Statistics:");
        println!("Total Download Speed: {}", format_rate(network.total_download_speed()));
        println!("Total Upload Speed: {}", format_rate(network.total_upload_speed()));
    }

    Ok(())
}
