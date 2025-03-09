# Network Monitoring

The Network module in `darwin-metrics` provides comprehensive monitoring for network interfaces and traffic statistics on macOS systems. It uses a combination of macOS native APIs and command-line utilities to collect real-time information about network interfaces, their status, and data transfer metrics.

## Features

- **Interface Discovery**: Automatically detect and monitor all network interfaces on macOS
- **Interface Classification**: Identify interface types (Ethernet, WiFi, Loopback, Virtual)
- **Traffic Statistics**: Track bytes and packets sent/received in real-time
- **Error Monitoring**: Track packet errors, collisions, and drops
- **State Tracking**: Monitor interface up/down status and flags
- **Interface Information**: Get MAC addresses, IP addresses, and interface capabilities
- **Speed Calculation**: Calculate real-time upload and download speeds

## macOS Implementation Details

The Network module is specifically designed for macOS systems and uses:

- **getifaddrs()**: For network interface enumeration and IP/MAC address collection
- **netstat**: For network traffic statistics collection
- **IOKit flags**: To determine interface capabilities and state

## Usage Example

Here's a basic example of using the Network module to monitor interface traffic:

```rust,ignore
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
        
        // Display MAC address if available
        if let Some(mac) = interface.mac_address() {
            println!("  MAC address: {}", mac);
        }
        
        // Traffic statistics
        println!("  Download: {} bytes ({:.2} KB/s)", 
                interface.bytes_received(), 
                interface.download_speed() / 1024.0);
        println!("  Upload: {} bytes ({:.2} KB/s)", 
                interface.bytes_sent(), 
                interface.upload_speed() / 1024.0);
        println!("  Packets: {} received, {} sent", 
                interface.packets_received(), 
                interface.packets_sent());
        
        // Error statistics
        if interface.receive_errors() > 0 || interface.send_errors() > 0 {
            println!("  Errors: {} receive, {} send", 
                    interface.receive_errors(), 
                    interface.send_errors());
            println!("  Collisions: {}", interface.collisions());
        }
        
        // Print IP addresses if available
        if let Some(addresses) = interface.addresses() {
            println!("  IP Addresses:");
            for addr in addresses {
                println!("    {}", addr);
            }
        }
        
        // Interface properties
        println!("  Properties:");
        println!("    Loopback: {}", interface.is_loopback());
        println!("    Wireless: {}", interface.is_wireless());
        println!("    Multicast: {}", interface.supports_multicast());
        println!("    Broadcast: {}", interface.supports_broadcast());
        
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

```rust,ignore
use darwin_metrics::network::{NetworkManager, NetworkMetrics};

fn example() -> darwin_metrics::Result<()> {
    // Create a new NetworkManager
    let mut network = NetworkManager::new()?;
    
    // Update network statistics
    network.update()?;
    
    // Get all interfaces
    let interfaces = network.interfaces();
    
    // Get interface count
    println!("Found {} network interfaces", interfaces.len());
    
    // Get a specific interface by name
    if let Some(wifi) = network.get_interface("en0") {
        println!("WiFi download: {:.2} KB/s", wifi.download_speed() / 1024.0);
    }
    
    // Get total network speeds across all interfaces
    println!("Total download: {:.2} MB/s", network.total_download_speed() / (1024.0 * 1024.0));
    println!("Total upload: {:.2} MB/s", network.total_upload_speed() / (1024.0 * 1024.0));
    
    Ok(())
}
```

### Interface Types

The `InterfaceType` enum identifies different types of network interfaces:

```rust,ignore
// From darwin_metrics::network::interface
pub enum InterfaceType {
    Ethernet,  // Wired ethernet interfaces
    WiFi,      // Wireless interfaces
    Loopback,  // Loopback interface (lo0)
    Virtual,   // Virtual interfaces (utun, bridge)
    Other,     // Other/unknown interface types
}
```

### Interface

The `Interface` struct represents a single network interface with all its properties and metrics:

```rust,ignore
use darwin_metrics::network::{NetworkManager, NetworkMetrics};
use std::net::IpAddr;

fn example() -> darwin_metrics::Result<()> {
    let network = NetworkManager::new()?;
    
    // For each interface in the system
    for interface in network.interfaces() {
        // Basic information
        println!("Name: {}", interface.name());
        println!("Type: {}", interface.interface_type());
        println!("Active: {}", interface.is_active());
        
        // MAC address (if available)
        if let Some(mac) = interface.mac_address() {
            println!("MAC: {}", mac);
        }
        
        // IP addresses (if available)
        if let Some(addrs) = interface.addresses() {
            for addr in addrs {
                println!("IP: {}", addr);
            }
        }
        
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
        println!("Download speed: {:.2} KB/s", interface.download_speed() / 1024.0);
        println!("Upload speed: {:.2} KB/s", interface.upload_speed() / 1024.0);
        println!("Packet receive rate: {:.2} pps", interface.packet_receive_rate());
        println!("Packet send rate: {:.2} pps", interface.packet_send_rate());
        
        // Interface properties
        println!("Is loopback: {}", interface.is_loopback());
        println!("Is wireless: {}", interface.is_wireless());
        println!("Supports broadcast: {}", interface.supports_broadcast());
        println!("Supports multicast: {}", interface.supports_multicast());
        println!("Is point-to-point: {}", interface.is_point_to_point());
    }
    
    Ok(())
}
```

### NetworkMetrics Trait

The `NetworkMetrics` trait defines standard methods implemented by network-related types:

```rust,ignore
// From darwin_metrics::network module
pub trait NetworkMetrics {
    // Data transfer metrics
    fn bytes_received(&self) -> u64;    // Total bytes received
    fn bytes_sent(&self) -> u64;        // Total bytes sent
    fn packets_received(&self) -> u64;  // Total packets received
    fn packets_sent(&self) -> u64;      // Total packets sent
    
    // Error metrics
    fn receive_errors(&self) -> u64;    // Count of receive errors
    fn send_errors(&self) -> u64;       // Count of send errors
    fn collisions(&self) -> u64;        // Count of collisions
    
    // Rate metrics
    fn download_speed(&self) -> f64;    // Current download speed in bytes/s
    fn upload_speed(&self) -> f64;      // Current upload speed in bytes/s
    
    // Status
    fn is_active(&self) -> bool;        // Whether the interface is active
}
```

## Error Handling

Network operations can return the following error types:

- `Error::Network`: Errors related to network operations
- `Error::NotAvailable`: When specific network metrics aren't available
- `Error::System`: Underlying system errors

The implementation includes graceful fallbacks when some metrics aren't available, avoiding crashes or panics when running on different macOS environments.

## Performance Considerations

- **Update Frequency**: For real-time monitoring, call `update()` at regular intervals (1-5 seconds)
- **Speed Calculations**: Require at least two measurements over time
- **Resource Usage**: The implementation is designed to be lightweight with minimal system impact
- **Thread Safety**: The API is not thread-safe by default; use mutex locks when sharing across threads

## Common Usage Patterns

1. **Simple Interface Listing**:
   ```rust,ignore
   use darwin_metrics::network::{NetworkManager, NetworkMetrics};
   
   fn example_interface_listing() -> darwin_metrics::Result<()> {
       let network = NetworkManager::new()?;
       for interface in network.interfaces() {
           println!("{}: {}", interface.name(), interface.interface_type());
       }
       Ok(())
   }
   ```

2. **Bandwidth Monitoring**:
   ```rust,ignore
   use darwin_metrics::network::{NetworkManager, NetworkMetrics};
   use std::thread::sleep;
   use std::time::Duration;
   
   fn example_bandwidth_monitoring() -> darwin_metrics::Result<()> {
       let mut network = NetworkManager::new()?;
       // This would run forever in a real application
       for _ in 0..3 {
           network.update()?;
           println!("Download: {:.2} MB/s", network.total_download_speed() / (1024.0 * 1024.0));
           sleep(Duration::from_secs(1));
       }
       Ok(())
   }
   ```

3. **Interface State Monitoring**:
   ```rust,ignore
   use darwin_metrics::network::{NetworkManager, NetworkMetrics};
   
   fn example_state_monitoring() -> darwin_metrics::Result<()> {
       let mut network = NetworkManager::new()?;
       network.update()?;
       if let Some(wifi) = network.get_interface("en0") {
           println!("WiFi is {}", if wifi.is_active() { "connected" } else { "disconnected" });
       }
       Ok(())
   }
   ```

4. **Traffic Usage Statistics**:
   ```rust,ignore
   use darwin_metrics::network::{NetworkManager, NetworkMetrics};
   use std::thread::sleep;
   use std::time::Duration;
   
   fn example_traffic_usage() -> darwin_metrics::Result<()> {
       let mut network = NetworkManager::new()?;
       let initial = network.interfaces().iter().map(|i| i.bytes_received()).sum::<u64>();
       sleep(Duration::from_secs(5)); // Using 5 seconds for testing, would be longer in real usage
       network.update()?;
       let final_bytes = network.interfaces().iter().map(|i| i.bytes_received()).sum::<u64>();
       println!("Used {} MB in the last period", 
              (final_bytes.saturating_sub(initial)) as f64 / (1024.0 * 1024.0));
       Ok(())
   }
   ```