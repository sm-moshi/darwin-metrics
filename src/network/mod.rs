//! # Network Monitoring Module
//!
//! The Network module provides comprehensive monitoring for network interfaces
//! and traffic statistics on macOS systems. It uses a combination of macOS
//! native APIs and command-line utilities to collect real-time information
//! about network interfaces, their status, and data transfer metrics.
//!
//! ## macOS Implementation Details
//!
//! The module uses:
//! - **getifaddrs()**: For network interface enumeration and IP/MAC address
//!   collection
//! - **netstat**: For network traffic statistics collection
//! - **IOKit flags**: To determine interface capabilities and state
//!
//! ## Features
//!
//! - **Interface Discovery**: Automatically detect and monitor all network
//!   interfaces on macOS
//! - **Interface Classification**: Identify interface types (Ethernet, WiFi,
//!   Loopback, Virtual)
//! - **Traffic Statistics**: Track bytes and packets sent/received in real-time
//! - **Error Monitoring**: Track packet errors, collisions, and drops
//! - **State Tracking**: Monitor interface up/down status and flags
//! - **Interface Information**: Get MAC addresses, IP addresses, and interface
//!   capabilities
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
//! - For real-time monitoring, call `update()` at regular intervals (1-5
//!   seconds)
//! - Speed calculations require at least two measurements over time
//! - The implementation is designed to be lightweight with minimal system
//!   impact
//! - The API is not thread-safe by default; use mutex locks when sharing across
//!   threads

mod interface;
mod traffic;

pub use interface::{Interface, InterfaceType, NetworkManager};

/// Trait defining the standard interface for accessing network metrics.
///
/// This trait provides a consistent API for retrieving common network metrics
/// regardless of the specific network interface or implementation details.
/// The trait is implemented by the `Interface` struct and provides methods
/// for querying data transfer statistics, error rates, and interface status.
pub trait NetworkMetrics {
    /// Returns the total number of bytes received.
    ///
    /// This represents the cumulative amount of data received by the interface
    /// since it was initialized or since the system was booted.
    fn bytes_received(&self) -> u64;

    /// Returns the total number of bytes sent.
    ///
    /// This represents the cumulative amount of data sent through the interface
    /// since it was initialized or since the system was booted.
    fn bytes_sent(&self) -> u64;

    /// Returns the total number of packets received.
    ///
    /// A packet represents a unit of data at the network layer, regardless of
    /// size.
    fn packets_received(&self) -> u64;

    /// Returns the total number of packets sent.
    ///
    /// A packet represents a unit of data at the network layer, regardless of
    /// size.
    fn packets_sent(&self) -> u64;

    /// Returns the number of receive errors.
    ///
    /// These are packets that were detected as erroneous and discarded during
    /// reception.
    fn receive_errors(&self) -> u64;

    /// Returns the number of send errors.
    ///
    /// These are errors that occurred during packet transmission.
    fn send_errors(&self) -> u64;

    /// Returns the number of packet collisions.
    ///
    /// Collisions occur when multiple devices on the same network segment
    /// attempt to transmit simultaneously. Most relevant for older Ethernet
    /// networks.
    fn collisions(&self) -> u64;

    /// Returns the current download speed in bytes per second.
    ///
    /// This is calculated based on the difference in bytes received between
    /// two measurements divided by the elapsed time. Requires multiple calls
    /// to update() over time to calculate an accurate rate.
    fn download_speed(&self) -> f64;

    /// Returns the current upload speed in bytes per second.
    ///
    /// This is calculated based on the difference in bytes sent between
    /// two measurements divided by the elapsed time. Requires multiple calls
    /// to update() over time to calculate an accurate rate.
    fn upload_speed(&self) -> f64;

    /// Returns true if the network interface is currently active.
    ///
    /// An active interface is one that is both UP and RUNNING according to
    /// the interface flags. This generally means the interface is properly
    /// configured and physically connected.
    fn is_active(&self) -> bool;
}

#[cfg(test)]
/// Unit tests for the network module
///
/// These tests verify the functionality of the network module using mock data
/// rather than actual system calls, ensuring the implementation behaves
/// correctly regardless of the system environment.
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::network::interface::{Interface, InterfaceType};

    // Create a mock network interface for testing
    fn create_mock_interface(
        name: &str,
        interface_type: InterfaceType,
        is_loopback: bool,
    ) -> Interface {
        let flags = if is_loopback {
            crate::utils::bindings::if_flags::IFF_UP
                | crate::utils::bindings::if_flags::IFF_RUNNING
                | crate::utils::bindings::if_flags::IFF_LOOPBACK
        } else {
            crate::utils::bindings::if_flags::IFF_UP | crate::utils::bindings::if_flags::IFF_RUNNING
        };

        Interface::new(
            name.to_string(),
            interface_type,
            flags,
            Some("00:00:00:00:00:00".to_string()),
            vec![],
            1000, // bytes_received
            2000, // bytes_sent
            100,  // packets_received
            200,  // packets_sent
            0,    // receive_errors
            0,    // send_errors
            0,    // collisions
        )
    }

    // Create a test NetworkManager
    fn create_test_network_manager() -> NetworkManager {
        let mut interfaces = HashMap::new();

        // Add loopback interface
        let lo0 = create_mock_interface("lo0", InterfaceType::Loopback, true);
        interfaces.insert("lo0".to_string(), lo0);

        // Add ethernet interface
        let en0 = create_mock_interface("en0", InterfaceType::Ethernet, false);
        interfaces.insert("en0".to_string(), en0);

        NetworkManager { interfaces }
    }

    #[test]
    fn test_network_metrics_trait() {
        // Create a test interface
        let interface = create_mock_interface("test0", InterfaceType::Ethernet, false);

        // Test NetworkMetrics trait implementation
        assert_eq!(interface.bytes_received(), 1000);
        assert_eq!(interface.bytes_sent(), 2000);
        assert_eq!(interface.packets_received(), 100);
        assert_eq!(interface.packets_sent(), 200);
        assert_eq!(interface.receive_errors(), 0);
        assert_eq!(interface.send_errors(), 0);
        assert_eq!(interface.collisions(), 0);
        assert!(interface.is_active());

        // Initial speeds should be 0 (need two measurements)
        assert_eq!(interface.download_speed(), 0.0);
        assert_eq!(interface.upload_speed(), 0.0);
    }

    #[test]
    fn test_interface_properties() {
        // Create test interfaces
        let loopback = create_mock_interface("lo0", InterfaceType::Loopback, true);
        let ethernet = create_mock_interface("en0", InterfaceType::Ethernet, false);

        // Test interface properties
        assert!(loopback.is_loopback());
        assert!(!ethernet.is_loopback());

        assert_eq!(loopback.name(), "lo0");
        assert_eq!(ethernet.name(), "en0");

        assert_eq!(loopback.interface_type(), &InterfaceType::Loopback);
        assert_eq!(ethernet.interface_type(), &InterfaceType::Ethernet);

        assert_eq!(loopback.mac_address(), Some("00:00:00:00:00:00"));
        assert_eq!(ethernet.mac_address(), Some("00:00:00:00:00:00"));
    }

    #[test]
    fn test_network_manager() {
        let network_manager = create_test_network_manager();

        // Test interfaces retrieval
        let interfaces = network_manager.interfaces();
        assert_eq!(interfaces.len(), 2, "Should have 2 interfaces");

        // Test interface lookup by name
        let lo0 = network_manager.get_interface("lo0");
        assert!(lo0.is_some(), "Loopback interface should exist");
        assert!(lo0.unwrap().is_loopback(), "lo0 should be a loopback interface");

        let en0 = network_manager.get_interface("en0");
        assert!(en0.is_some(), "Ethernet interface should exist");
        assert!(!en0.unwrap().is_loopback(), "en0 should not be a loopback interface");

        // Test non-existent interface
        let nonexistent = network_manager.get_interface("nonexistent");
        assert!(nonexistent.is_none(), "Nonexistent interface should return None");

        // Test total speeds (should be sum of all interfaces)
        assert_eq!(
            network_manager.total_download_speed(),
            0.0,
            "Initial download speed should be 0"
        );
        assert_eq!(network_manager.total_upload_speed(), 0.0, "Initial upload speed should be 0");
    }

    // If this test runs on a real machine, it might still fail because we can't
    // guarantee the system has network interfaces. We'll skip this test in CI
    // environments.
    #[test]
    #[ignore = "This test needs a real network environment"]
    fn test_real_network_manager() {
        let network_manager = NetworkManager::new();

        // If we're on a real system with network interfaces, this should succeed
        if let Ok(manager) = network_manager {
            assert!(!manager.interfaces().is_empty(), "Should have at least one interface");
        }
    }
}
