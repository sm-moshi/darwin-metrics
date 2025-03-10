# Network Monitoring

The Network module in `darwin-metrics` provides comprehensive monitoring for network interfaces and traffic statistics on macOS systems. It uses a combination of macOS native APIs and system frameworks to collect real-time information about network interfaces, their status, and data transfer metrics.

## Features

-   **Interface Discovery**: Automatically detect and monitor all network interfaces on macOS
-   **Interface Classification**: Identify interface types (Ethernet, WiFi, Loopback, Virtual)
-   **Traffic Statistics**: Track bytes and packets sent/received in real-time
-   **Error Monitoring**: Track packet errors, collisions, and drops
-   **State Tracking**: Monitor interface up/down status and flags
-   **Interface Information**: Get MAC addresses, IP addresses, and interface capabilities
-   **Speed Calculation**: Calculate real-time upload and download speeds
-   **Connection Monitoring**: Track active network connections and their status

## macOS Implementation Details

The Network module is specifically designed for macOS systems and uses:

-   **getifaddrs()**: For network interface enumeration and IP/MAC address collection
-   **sysctlbyname**: Primary method for network traffic statistics using direct kernel APIs
-   **netstat**: Fallback method for traffic statistics if sysctlbyname fails
-   **SystemConfiguration framework**: For network interface configuration
-   **IOKit API**: To determine interface capabilities and state

## Usage Example

Here's a basic example of using the Network module to monitor interface traffic:

```rust,ignore
use darwin_metrics::network::{NetworkManager, NetworkMetrics};
use std::{thread, time::Duration};

fn main() -> darwin_metrics::error::Result<()> {
    // Create a new network manager
    let mut manager = NetworkManager::new()?;

    // Initial stats
    println!("Initial network statistics:");
    manager.update()?;
    print_network_stats(&manager);

    // Sleep for a while to allow traffic to occur
    thread::sleep(Duration::from_secs(5));

    // Update stats
    manager.update()?;

    // Print updated stats with speeds
    println!("\nUpdated network statistics:");
    print_network_stats(&manager);

    Ok(())
}

fn print_network_stats(manager: &NetworkManager) {
    for interface in manager.interfaces() {
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
    println!("  Download speed: {:.2} KB/s", manager.total_download_speed() / 1024.0);
    println!("  Upload speed: {:.2} KB/s", manager.total_upload_speed() / 1024.0);
}
```

## API Overview

### NetworkManager

The `NetworkManager` is the main entry point for network monitoring:

```rust,ignore
use darwin_metrics::network::{NetworkManager, NetworkMetrics};

fn example() -> darwin_metrics::error::Result<()> {
    // Create a new NetworkManager
    let mut manager = NetworkManager::new()?;

    // Update network statistics
    manager.update()?;

    // Get all interfaces
    let interfaces = manager.interfaces();

    // Get interface count
    println!("Found {} network interfaces", interfaces.len());

    // Get a specific interface by name
    if let Some(wifi) = manager.get_interface("en0") {
        println!("WiFi download: {:.2} KB/s", wifi.download_speed() / 1024.0);
    }

    // Get total network speeds across all interfaces
    println!("Total download: {:.2} MB/s", manager.total_download_speed() / (1024.0 * 1024.0));
    println!("Total upload: {:.2} MB/s", manager.total_upload_speed() / (1024.0 * 1024.0));

    Ok(())
}

// Async version
async fn async_example() -> darwin_metrics::error::Result<()> {
    // Create a new NetworkManager
    let mut manager = NetworkManager::new()?;

    // Update network statistics asynchronously
    manager.update_async().await?;

    // Get active interfaces asynchronously
    let active_interfaces = manager.active_interfaces_async().await;
    println!("Active interfaces: {}", active_interfaces.len());

    // Get throughput asynchronously
    let (download, upload) = manager.get_throughput_async().await?;
    println!("Total download: {:.2} MB/s", download / (1024.0 * 1024.0));
    println!("Total upload: {:.2} MB/s", upload / (1024.0 * 1024.0));

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
    Cellular,  // Cellular network interfaces
    Other,     // Other/unknown interface types
}
```

### Interface

The `Interface` struct represents a single network interface with all its properties and metrics:

```rust,ignore
use darwin_metrics::network::{NetworkMonitor, NetworkMetrics};
use std::net::IpAddr;

async fn example() -> darwin_metrics::Result<()> {
    let monitor = NetworkMonitor::new().await?;

    // For each interface in the system
    for interface in monitor.interfaces() {
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

        // Signal strength (for wireless interfaces)
        if interface.is_wireless() {
            if let Some(signal) = interface.signal_strength() {
                println!("Signal strength: {} dBm", signal);
            }
        }
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
    fn packet_receive_rate(&self) -> f64; // Packets per second received
    fn packet_send_rate(&self) -> f64;    // Packets per second sent

    // Status
    fn is_active(&self) -> bool;        // Whether the interface is active
}
```

### Future Connection Monitoring

In upcoming releases, the NetworkManager will provide functionality to track active network connections:

```rust,ignore
// This is a planned feature - not yet implemented
use darwin_metrics::network::{NetworkManager, Connection, Protocol};

async fn connection_monitoring() -> darwin_metrics::error::Result<()> {
    let mut manager = NetworkManager::new()?;
    
    // Example of future API for connection monitoring
    manager.update_async().await?;
    
    // Get all active connections (future feature)
    let connections = manager.connections().await?;

    for conn in connections {
        println!("Connection: {}:{} -> {}:{}",
            conn.local_addr,
            conn.local_port(),
            conn.remote_addr,
            conn.remote_port());

        println!("  Protocol: {}", match conn.protocol {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
            Protocol::Other(_) => "Other"
        });

        if let Some(state) = conn.state {
            println!("  State: {:?}", state);
        }

        if let Some(pid) = conn.pid {
            println!("  Process ID: {}", pid);
        }
    }

    Ok(())
}
```

## Error Handling

Network operations can return the following error types:

-   `Error::Network`: Errors related to network operations
-   `Error::NotAvailable`: When specific network metrics aren't available
-   `Error::System`: Underlying system errors
-   `Error::Permission`: When insufficient permissions exist to access network data

The implementation includes graceful fallbacks when some metrics aren't available, avoiding crashes or panics when running on different macOS environments.

## Performance Considerations

-   **Native Implementation**: Uses direct sysctlbyname kernel calls for optimal performance
-   **Fallback Mechanism**: Automatically falls back to netstat if native APIs are unavailable
-   **Update Frequency**: For real-time monitoring, call `update()` at regular intervals (1-5 seconds)
-   **Speed Calculations**: Require at least two measurements over time for accurate speeds
-   **Resource Usage**: The implementation is designed to be lightweight with minimal system impact
-   **Thread Safety**: Synchronous API with async alternatives via tokio for non-blocking operation
-   **64-bit Counters**: Uses 64-bit counters to handle high-bandwidth interfaces

## Common Usage Patterns

1. **Simple Interface Listing**:

    ```rust,ignore
    use darwin_metrics::network::{NetworkMonitor, NetworkMetrics};

    async fn example_interface_listing() -> darwin_metrics::Result<()> {
        let monitor = NetworkMonitor::new().await?;
        for interface in monitor.interfaces() {
            println!("{}: {}", interface.name(), interface.interface_type());
        }
        Ok(())
    }
    ```

2. **Bandwidth Monitoring**:

    ```rust,ignore
    use darwin_metrics::network::{NetworkMonitor, NetworkMetrics};
    use std::time::Duration;
    use tokio::time::sleep;

    async fn example_bandwidth_monitoring() -> darwin_metrics::Result<()> {
        let mut monitor = NetworkMonitor::new().await?;
        // This would run forever in a real application
        for _ in 0..3 {
            monitor.refresh().await?;
            println!("Download: {:.2} MB/s", monitor.total_download_speed() / (1024.0 * 1024.0));
            sleep(Duration::from_secs(1)).await;
        }
        Ok(())
    }
    ```

3. **Interface State Monitoring**:

    ```rust,ignore
    use darwin_metrics::network::{NetworkMonitor, NetworkMetrics};

    async fn example_state_monitoring() -> darwin_metrics::Result<()> {
        let mut monitor = NetworkMonitor::new().await?;
        monitor.refresh().await?;
        if let Some(wifi) = monitor.get_interface("en0") {
            println!("WiFi is {}", if wifi.is_active() { "connected" } else { "disconnected" });
            if wifi.is_active() && wifi.is_wireless() {
                if let Some(signal) = wifi.signal_strength() {
                    println!("Signal strength: {} dBm", signal);
                }
            }
        }
        Ok(())
    }
    ```

4. **Traffic Usage Statistics**:

    ```rust,ignore
    use darwin_metrics::network::{NetworkMonitor, NetworkMetrics};
    use tokio::time::sleep;
    use std::time::Duration;

    async fn example_traffic_usage() -> darwin_metrics::Result<()> {
        let mut monitor = NetworkMonitor::new().await?;
        let initial = monitor.interfaces().iter().map(|i| i.bytes_received()).sum::<u64>();
        sleep(Duration::from_secs(5)).await; // Using 5 seconds for testing, would be longer in real usage
        monitor.refresh().await?;
        let final_bytes = monitor.interfaces().iter().map(|i| i.bytes_received()).sum::<u64>();
        println!("Used {} MB in the last period",
               (final_bytes.saturating_sub(initial)) as f64 / (1024.0 * 1024.0));
        Ok(())
    }
    ```

5. **Native Implementation with Fallback**:

    ```rust,ignore
    use darwin_metrics::network::{NetworkManager, NetworkMetrics};
    use darwin_metrics::error::Result;
    
    fn example_reliable_monitoring() -> Result<()> {
        // The NetworkManager will automatically use the most efficient 
        // implementation available on the system (native sysctlbyname first,
        // falling back to netstat if needed)
        let mut manager = NetworkManager::new()?;
        
        // Initialize
        manager.update()?;
        println!("Monitoring using native implementation");
        
        // Get interface speeds
        for interface in manager.interfaces() {
            if interface.is_active() {
                println!("{}: {:.2} KB/s down, {:.2} KB/s up", 
                    interface.name(),
                    interface.download_speed() / 1024.0,
                    interface.upload_speed() / 1024.0);
            }
        }
        
        Ok(())
    }
    ```
