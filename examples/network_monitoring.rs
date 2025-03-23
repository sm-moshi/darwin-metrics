use std::io;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use darwin_metrics::core::metrics::hardware::{
    NetworkBandwidthMonitor, NetworkErrorMonitor, NetworkInterfaceMonitor, NetworkPacketMonitor,
};
use darwin_metrics::error::Result;
use darwin_metrics::network::interface::NetworkManager;

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
    println!("Darwin Metrics - Network Monitor Example");
    println!("Press Ctrl+C to exit\n");

    // Create a new network manager
    let mut network = NetworkManager::new()?;

    // Display initial network interfaces
    println!("Detected network interfaces:");
    for interface in network.interfaces() {
        println!("  {} ({})", interface.name(), interface.interface_type());
        if let Some(mac) = interface.mac_address() {
            println!("    MAC: {}", mac);
        }
        println!(
            "    Status: {}",
            if !interface.name().is_empty() { "Up" } else { "Down" }
        );
    }

    println!("\nStarting real-time monitoring...\n");
    println!("Press Ctrl+C to exit");

    loop {
        // Update network stats
        network.update()?;

        // Clear the screen
        print!("\x1B[2J\x1B[1;1H");

        println!("════════════ NETWORK INTERFACES ════════════");

        // Display detailed information for each interface
        for interface in network.interfaces() {
            // Create monitors
            let state_monitor = interface.interface_monitor();
            let bandwidth_monitor = interface.bandwidth_monitor();
            let packet_monitor = interface.packet_monitor();
            let error_monitor = interface.error_monitor();

            // Skip inactive interfaces
            if interface.name().is_empty() {
                continue;
            }

            println!("\nInterface: {} ({})", interface.name(), interface.interface_type());

            // State
            println!("  Status: {}", if !interface.name().is_empty() { "Up" } else { "Down" });
            if let Some(mac) = interface.mac_address() {
                println!("  MAC: {}", mac);
            }

            // Bandwidth - use async trait methods
            let download = bandwidth_monitor.download_speed().await?;
            let upload = bandwidth_monitor.upload_speed().await?;
            let bytes_received = bandwidth_monitor.bytes_received().await?;
            let bytes_sent = bandwidth_monitor.bytes_sent().await?;

            println!("  Download: {}/s", format_rate(download));
            println!("  Upload: {}/s", format_rate(upload));
            println!("  Total bytes received: {}", format_bytes(bytes_received));
            println!("  Total bytes sent: {}", format_bytes(bytes_sent));

            // Packets - use async trait methods
            let packets_received = packet_monitor.packets_received().await?;
            let packets_sent = packet_monitor.packets_sent().await?;
            let packet_receive_rate = packet_monitor.packet_receive_rate().await?;
            let packet_send_rate = packet_monitor.packet_send_rate().await?;

            println!("  Packets received: {}", packets_received);
            println!("  Packets sent: {}", packets_sent);
            println!("  Packet receive rate: {:.2} pkt/s", packet_receive_rate);
            println!("  Packet send rate: {:.2} pkt/s", packet_send_rate);

            // Errors - use async trait methods
            let receive_errors = error_monitor.receive_errors().await?;
            let send_errors = error_monitor.send_errors().await?;
            let collisions = error_monitor.collisions().await?;

            if receive_errors > 0 || send_errors > 0 {
                println!("  Receive errors: {}", receive_errors);
                println!("  Send errors: {}", send_errors);
                println!("  Collisions: {}", collisions);
            }
        }

        std::thread::sleep(Duration::from_secs(1));
    }
}
