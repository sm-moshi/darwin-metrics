# Network Monitoring

The Network module in `darwin-metrics` provides comprehensive monitoring for network interfaces and traffic statistics on macOS systems. It uses a combination of macOS native APIs and system frameworks to collect real-time information about network interfaces, their status, and data transfer metrics.

## Features

- **Interface Discovery**: Automatically detect and monitor all network interfaces on macOS
- **Interface Classification**: Identify interface types (Ethernet, WiFi, Loopback, Virtual)
- **Traffic Statistics**: Track bytes and packets sent/received in real-time
- **Error Monitoring**: Track packet errors, collisions, and drops
- **State Tracking**: Monitor interface up/down status and flags
- **Interface Information**: Get MAC addresses, IP addresses, and interface capabilities
- **Speed Calculation**: Calculate real-time upload and download speeds
- **Connection Monitoring**: Track active network connections and their status

## macOS Implementation Details

The Network module is specifically designed for macOS systems and uses:

- **SystemConfiguration framework**: For network interface enumeration and configuration
- **Network framework**: For modern network monitoring capabilities
- **getifaddrs()**: For IP/MAC address collection
- **IOKit API**: To determine interface capabilities and state

## Usage Example

Here's a basic example of using the Network module to monitor interface traffic:

```rust,ignore
use darwin_metrics::network::{NetworkMonitor, NetworkMetrics};
use std::thread::sleep;
use std::time::Duration;

async fn main() -> darwin_metrics::Result<()> {
    // Create a new network monitor
    let mut monitor = NetworkMonitor::new().await?;

    // Initial stats
    println!("Initial network statistics:");
    print_network_stats(&monitor);

    // Sleep for a while to allow traffic to occur
    sleep(Duration::from_secs(5));

    // Update stats
    monitor.refresh().await?;

    // Print updated stats with speeds
    println!("\nUpdated network statistics:");
    print_network_stats(&monitor);

    Ok(())
}

fn print_network_stats(monitor: &NetworkMonitor) {
    for interface in monitor.interfaces() {
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
    println!("  Download speed: {:.2} KB/s", monitor.total_download_speed() / 1024.0);
    println!("  Upload speed: {:.2} KB/s", monitor.total_upload_speed() / 1024.0);
}
```

## API Overview

### NetworkMonitor

The `NetworkMonitor` is the main entry point for network monitoring:

```rust,ignore
use darwin_metrics::network::{NetworkMonitor, NetworkMetrics};

async fn example() -> darwin_metrics::Result<()> {
    // Create a new NetworkMonitor
    let mut monitor = NetworkMonitor::new().await?;

    // Update network statistics
    monitor.refresh().await?;

    // Get all interfaces
    let interfaces = monitor.interfaces();

    // Get interface count
    println!("Found {} network interfaces", interfaces.len());

    // Get a specific interface by name
    if let Some(wifi) = monitor.get_interface("en0") {
        println!("WiFi download: {:.2} KB/s", wifi.download_speed() / 1024.0);
    }

    // Get total network speeds across all interfaces
    println!("Total download: {:.2} MB/s", monitor.total_download_speed() / (1024.0 * 1024.0));
    println!("Total upload: {:.2} MB/s", monitor.total_upload_speed() / (1024.0 * 1024.0));

    // Get active connections
    let connections = monitor.connections().await?;
    println!("Active connections: {}", connections.len());

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

### Connection Monitoring

The NetworkMonitor also provides functionality to track active network connections:

```rust,ignore
use darwin_metrics::network::{NetworkMonitor, Connection, ConnectionType};

async fn connection_monitoring() -> darwin_metrics::Result<()> {
    let monitor = NetworkMonitor::new().await?;

    // Get all active connections
    let connections = monitor.connections().await?;

    for conn in connections {
        println!("Connection: {}:{} -> {}:{}",
            conn.local_address(),
            conn.local_port(),
            conn.remote_address().unwrap_or_else(|| "N/A".to_string()),
            conn.remote_port());

        println!("  Type: {}", match conn.connection_type() {
            ConnectionType::Tcp => "TCP",
            ConnectionType::Udp => "UDP",
            ConnectionType::Other => "Other"
        });

        println!("  State: {}", conn.state());

        if let Some(process) = conn.process_name() {
            println!("  Process: {} (PID: {})", process, conn.pid().unwrap_or(0));
        }
    }

    Ok(())
}
```

## Error Handling

Network operations can return the following error types:

- `Error::Network`: Errors related to network operations
- `Error::NotAvailable`: When specific network metrics aren't available
- `Error::System`: Underlying system errors
- `Error::Permission`: When insufficient permissions exist to access network data

The implementation includes graceful fallbacks when some metrics aren't available, avoiding crashes or panics when running on different macOS environments.

## Performance Considerations

- **Update Frequency**: For real-time monitoring, call `refresh()` at regular intervals (1-5 seconds)
- **Speed Calculations**: Require at least two measurements over time
- **Resource Usage**: The implementation is designed to be lightweight with minimal system impact
- **Thread Safety**: The API is designed to be used safely with tokio's async runtime
- **Caching Strategy**: Some data is cached to reduce system calls and improve performance

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

5. **Connection Monitoring**:

    ```rust,ignore
    use darwin_metrics::network::{NetworkMonitor, ConnectionType};

    async fn example_connection_monitoring() -> darwin_metrics::Result<()> {
        let monitor = NetworkMonitor::new().await?;

        // Count connections by type
        let connections = monitor.connections().await?;
        let tcp_count = connections.iter()
            .filter(|c| c.connection_type() == ConnectionType::Tcp)
            .count();
        let udp_count = connections.iter()
            .filter(|c| c.connection_type() == ConnectionType::Udp)
            .count();

        println!("Active connections: {} TCP, {} UDP", tcp_count, udp_count);

        // Find connections for a specific process
        let browser_connections = connections.iter()
            .filter(|c| c.process_name().map_or(false, |name| name.contains("firefox") || name.contains("chrome")))
            .count();
        println!("Browser connections: {}", browser_connections);

        Ok(())
    }
    ```
