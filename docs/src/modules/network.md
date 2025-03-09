# Network Monitoring

The Network module in `darwin-metrics` provides comprehensive monitoring for network interfaces and traffic statistics on macOS systems. This module uses native macOS APIs to collect real-time information about network interfaces, their status, and data transfer metrics.

## Features

- **Interface Discovery**: Automatically detect and monitor all network interfaces
- **Traffic Statistics**: Track bytes and packets sent/received in real-time
- **Error Monitoring**: Track packet errors, collisions, and drops
- **State Tracking**: Monitor interface up/down status
- **Interface Information**: Get MAC addresses, IP addresses, and interface types
- **Speed Calculation**: Calculate real-time upload and download speeds

## Usage Example

Here's a basic example of using the Network module to monitor interface traffic:

```rust
use darwin_metrics::network::{NetworkManager, NetworkMetrics};
use std::thread::sleep;
use std::time::Duration;

fn main() -> darwin_metrics::Result<()> {
    // Create a new network manager
    let mut network = NetworkManager::new()?;
    
    // Initial stats
    println!("Initial network statistics:");
    print_network_stats(&network);
    
    // Sleep for a while to allow traffic to occur
    sleep(Duration::from_secs(5));
    
    // Update stats
    network.update()?;
    
    // Print updated stats with speeds
    println!("\nUpdated network statistics:");
    print_network_stats(&network);
    
    Ok(())
}

fn print_network_stats(network: &NetworkManager) {
    for interface in network.interfaces() {
        println!("Interface: {} ({})", interface.name(), interface.interface_type());
        println!("  Status: {}", if interface.is_active() { "Active" } else { "Inactive" });
        
        // Traffic statistics
        println!("  Download: {} bytes ({} bytes/s)", interface.bytes_received(), interface.download_speed());
        println!("  Upload: {} bytes ({} bytes/s)", interface.bytes_sent(), interface.upload_speed());
        
        // Error statistics
        if interface.receive_errors() > 0 || interface.send_errors() > 0 {
            println!("  Errors: {} receive, {} send", interface.receive_errors(), interface.send_errors());
        }
        
        // Print IP addresses if available
        if let Some(addresses) = interface.addresses() {
            println!("  IP Addresses:");
            for addr in addresses {
                println!("    {}", addr);
            }
        }
        
        println!("");
    }
    
    // Print total network usage
    println!("Total network usage:");
    println!("  Download speed: {:.2} KB/s", network.total_download_speed() / 1024.0);
    println!("  Upload speed: {:.2} KB/s", network.total_upload_speed() / 1024.0);
}
```

## API Overview

### NetworkManager

The `NetworkManager` is the main entry point for network monitoring:

```rust
// Create a new NetworkManager
let mut network = NetworkManager::new()?;

// Update network statistics
network.update()?;

// Get all interfaces
let interfaces = network.interfaces();

// Get a specific interface by name
if let Some(wifi) = network.get_interface("en0") {
    println!("WiFi download: {} bytes/s", wifi.download_speed());
}

// Get total network speeds across all interfaces
println!("Total download: {} bytes/s", network.total_download_speed());
println!("Total upload: {} bytes/s", network.total_upload_speed());
```

### Interface

The `Interface` struct represents a single network interface:

```rust
// Basic information
println!("Name: {}", interface.name());
println!("Type: {}", interface.interface_type());
println!("Active: {}", interface.is_active());

// Traffic statistics
println!("Bytes received: {}", interface.bytes_received());
println!("Bytes sent: {}", interface.bytes_sent());
println!("Packets received: {}", interface.packets_received());
println!("Packets sent: {}", interface.packets_sent());

// Error statistics
println!("Receive errors: {}", interface.receive_errors());
println!("Send errors: {}", interface.send_errors());
println!("Collisions: {}", interface.collisions());

// Speed calculations
println!("Download speed: {}", interface.download_speed());
println!("Upload speed: {}", interface.upload_speed());

// Interface properties
println!("Is loopback: {}", interface.is_loopback());
println!("Is wireless: {}", interface.is_wireless());
println!("Supports multicast: {}", interface.supports_multicast());
```

### NetworkMetrics Trait

The `NetworkMetrics` trait defines common methods implemented by network-related types:

```rust
pub trait NetworkMetrics {
    fn bytes_received(&self) -> u64;
    fn bytes_sent(&self) -> u64;
    fn packets_received(&self) -> u64;
    fn packets_sent(&self) -> u64;
    fn receive_errors(&self) -> u64;
    fn send_errors(&self) -> u64;
    fn collisions(&self) -> u64;
    fn download_speed(&self) -> f64;
    fn upload_speed(&self) -> f64;
    fn is_active(&self) -> bool;
}
```

## Error Handling

Network operations can return the following error types:

- `Error::Network`: Errors related to network operations
- `Error::NotAvailable`: When specific network metrics aren't available
- `Error::System`: Underlying system errors

## Implementation Notes

- Network monitoring uses macOS-specific APIs: `getifaddrs()` and `sysctl`
- Speed calculations require at least two measurements over time
- To track real-time network usage, call `update()` frequently
- For best results with bandwidth tracking, update every 1-5 seconds