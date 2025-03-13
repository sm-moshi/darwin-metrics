//! # Network Monitoring Module
//!
//! The Network module provides comprehensive monitoring for network interfaces and traffic statistics on macOS systems.
//! It uses a combination of macOS native APIs and command-line utilities to collect real-time information about network
//! interfaces, their status, and data transfer metrics.
//!
//! ## macOS Implementation Details
//!
//! The module uses:
//! - **getifaddrs()**: For network interface enumeration and IP/MAC address collection
//! - **sysctlbyname**: Primary method for network traffic statistics collection using direct kernel APIs
//! - **netstat**: Fallback method for traffic statistics if sysctlbyname fails
//! - **IOKit flags**: To determine interface capabilities and state
//!
//! ## Features
//!
//! - **Interface Discovery**: Automatically detect and monitor all network interfaces on macOS
//! - **Interface Classification**: Identify interface types (Ethernet, WiFi, Loopback, Virtual)
//! - **Traffic Statistics**: Track bytes and packets sent/received in real-time
//! - **Error Monitoring**: Track packet errors, collisions, and drops
//! - **State Tracking**: Monitor interface up/down status and flags
//! - **Interface Information**: Get MAC addresses, IP addresses, and interface capabilities
//! - **Speed Calculation**: Calculate real-time upload and download speeds
//!
//! ## Example
//!
//! ```rust,no_run
//! use std::{thread::sleep, time::Duration};
//!
//! use darwin_metrics::network::{NetworkManager, NetworkMetrics};
//!
//! fn main() -> darwin_metrics::error::Result<()> {
//!     // Create a new network manager
//!     let mut network = NetworkManager::new()?;
//!
//!     // Get initial stats
//!     println!("Initial network statistics:");
//!
//!     for interface in network.interfaces() {
//!         println!("Interface: {} ({})", interface.name(), interface.interface_type());
//!         println!("  Status: {}", if interface.is_active() { "Active" } else { "Inactive" });
//!
//!         // Display MAC address if available
//!         if let Some(mac) = interface.mac_address() {
//!             println!("  MAC address: {}", mac);
//!         }
//!
//!         // Display traffic statistics
//!         println!("  Bytes received: {}", interface.bytes_received());
//!         println!("  Bytes sent: {}", interface.bytes_sent());
//!         println!("  Packets received: {}", interface.packets_received());
//!         println!("  Packets sent: {}", interface.packets_sent());
//!     }
//!
//!     // Wait a bit to allow traffic to occur
//!     sleep(Duration::from_secs(3));
//!
//!     // Update statistics and show speeds
//!     network.update()?;
//!     println!("\nUpdated network statistics with speeds:");
//!
//!     // Get specific interface
//!     if let Some(en0) = network.get_interface("en0") {
//!         println!("en0 download speed: {:.2} KB/s", en0.download_speed() / 1024.0);
//!         println!("en0 upload speed: {:.2} KB/s", en0.upload_speed() / 1024.0);
//!     }
//!
//!     // Show total network usage
//!     println!("Total network usage:");
//!     println!("  Download: {:.2} MB/s", network.total_download_speed() / (1024.0 * 1024.0));
//!     println!("  Upload: {:.2} MB/s", network.total_upload_speed() / (1024.0 * 1024.0));
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - For real-time monitoring, call `update()` at regular intervals (1-5 seconds)
//! - Speed calculations require at least two measurements over time
//! - The implementation is designed to be lightweight with minimal system impact
//! - The API is not thread-safe by default; use mutex locks when sharing across threads

/// Network interface monitoring functionality
///
/// This module provides tools for monitoring network interfaces on macOS systems. It includes support for:
///
/// - Interface status
/// - IP address information
/// - Link state monitoring
/// - Interface statistics
pub mod interface;

/// Network traffic monitoring functionality
///
/// This module provides tools for monitoring network traffic on macOS systems. It includes support for:
///
/// - Bandwidth usage
/// - Packet statistics
/// - Protocol-specific metrics
/// - Traffic analysis
pub mod traffic;

pub use interface::{Interface, InterfaceType, NetworkManager};
pub use traffic::TrafficData;

/// Trait defining the standard interface for accessing network metrics.
///
/// This trait provides a consistent API for retrieving common network metrics regardless of the specific network
/// interface or implementation details. The trait is implemented by the `Interface` struct and provides methods for
/// querying data transfer statistics, error rates, and interface status.
pub trait NetworkMetrics {
    /// Returns the total number of bytes received.
    ///
    /// This represents the cumulative amount of data received by the interface since it was initialized or since the
    /// system was booted.
    fn bytes_received(&self) -> u64;

    /// Returns the total number of bytes sent.
    ///
    /// This represents the cumulative amount of data sent through the interface since it was initialized or since the
    /// system was booted.
    fn bytes_sent(&self) -> u64;

    /// Returns the total number of packets received.
    ///
    /// A packet represents a unit of data at the network layer, regardless of size.
    fn packets_received(&self) -> u64;

    /// Returns the total number of packets sent.
    ///
    /// A packet represents a unit of data at the network layer, regardless of size.
    fn packets_sent(&self) -> u64;

    /// Returns the number of receive errors.
    ///
    /// These are packets that were detected as erroneous and discarded during reception.
    fn receive_errors(&self) -> u64;

    /// Returns the number of send errors.
    ///
    /// These are errors that occurred during packet transmission.
    fn send_errors(&self) -> u64;

    /// Returns the number of packet collisions.
    ///
    /// Collisions occur when multiple devices on the same network segment attempt to transmit simultaneously. Most
    /// relevant for older Ethernet networks.
    fn collisions(&self) -> u64;

    /// Returns the current download speed in bytes per second.
    ///
    /// This is calculated based on the difference in bytes received between two measurements divided by the elapsed
    /// time. Requires multiple calls to update() over time to calculate an accurate rate.
    fn download_speed(&self) -> f64;

    /// Returns the current upload speed in bytes per second.
    ///
    /// This is calculated based on the difference in bytes sent between two measurements divided by the elapsed time.
    /// Requires multiple calls to update() over time to calculate an accurate rate.
    fn upload_speed(&self) -> f64;

    /// Returns true if the network interface is currently active.
    ///
    /// An active interface is one that is both UP and RUNNING according to the interface flags. This generally means
    /// the interface is properly configured and physically connected.
    fn is_active(&self) -> bool;
}
