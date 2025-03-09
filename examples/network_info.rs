use std::{thread::sleep, time::Duration};

use darwin_metrics::network::{NetworkManager, NetworkMetrics};

fn main() -> darwin_metrics::Result<()> {
    // Create a new network manager
    let mut network = NetworkManager::new()?;

    // Get initial network statistics
    println!("Initial network statistics:");
    print_network_stats(&network);

    // Wait a bit to allow traffic to occur
    println!("\nWaiting 3 seconds for network activity...");
    sleep(Duration::from_secs(3));

    // Update statistics and show speed
    network.update()?;
    println!("\nUpdated network statistics:");
    print_network_stats(&network);

    Ok(())
}

fn print_network_stats(network: &NetworkManager) {
    for interface in network.interfaces() {
        println!("\n{} ({})", interface.name(), interface.interface_type());
        println!("  Status: {}", if interface.is_active() { "Active" } else { "Inactive" });

        // Display interface properties
        if let Some(mac) = interface.mac_address() {
            println!("  MAC address: {}", mac);
        }

        // Display IP addresses if available
        if let Some(addresses) = interface.addresses() {
            println!("  IP addresses:");
            for addr in addresses {
                println!("    {}", addr);
            }
        }

        // Display interface capabilities
        println!("  Properties:");
        println!("    Loopback: {}", interface.is_loopback());
        println!("    Wireless: {}", interface.is_wireless());
        println!("    Supports broadcast: {}", interface.supports_broadcast());
        println!("    Supports multicast: {}", interface.supports_multicast());

        // Display traffic statistics
        println!("  Traffic:");
        println!(
            "    Received: {} bytes ({:.2} KB/s)",
            interface.bytes_received(),
            interface.download_speed() / 1024.0
        );
        println!(
            "    Sent: {} bytes ({:.2} KB/s)",
            interface.bytes_sent(),
            interface.upload_speed() / 1024.0
        );
        println!(
            "    Packets: {} received, {} sent",
            interface.packets_received(),
            interface.packets_sent()
        );

        // Display error statistics if any
        if interface.receive_errors() > 0
            || interface.send_errors() > 0
            || interface.collisions() > 0
        {
            println!("  Errors:");
            println!("    Receive errors: {}", interface.receive_errors());
            println!("    Send errors: {}", interface.send_errors());
            println!("    Collisions: {}", interface.collisions());
        }
    }

    // Display total network speeds
    println!("\nTotal network usage:");
    println!("  Download: {:.2} KB/s", network.total_download_speed() / 1024.0);
    println!("  Upload: {:.2} KB/s", network.total_upload_speed() / 1024.0);
}
