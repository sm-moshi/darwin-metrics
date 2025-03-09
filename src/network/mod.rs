//! # Network Monitoring Module
//!
//! The network module provides access to macOS network interface metrics and statistics.
//! It offers functionality to monitor network traffic, interface status, and detailed
//! network performance metrics.
//!
//! This module interfaces with macOS system APIs to retrieve network information using
//! the getifaddrs() family of functions and related syscalls.
//!
//! ## Features
//!
//! - **Interface Enumeration**: Discover and monitor all network interfaces
//! - **Traffic Statistics**: Track bytes and packets sent/received
//! - **Error Monitoring**: Monitor packet errors and drops
//! - **State Tracking**: Detect interface up/down status
//! - **Interface Information**: Get MAC address, IP address, and other interface details
//!
//! ## Example
//!
//! ```rust,no_run
//! use darwin_metrics::network::{NetworkManager, NetworkMetrics};
//!
//! fn main() -> darwin_metrics::error::Result<()> {
//!     // Create a new network manager to monitor interfaces
//!     let mut network = NetworkManager::new()?;
//!     
//!     // Refresh network statistics
//!     network.update()?;
//!     
//!     // Get all network interfaces
//!     let interfaces = network.interfaces();
//!     
//!     for interface in interfaces {
//!         println!("Interface: {}", interface.name());
//!         println!("  Status: {}", if interface.is_active() { "Active" } else { "Inactive" });
//!         println!("  Type: {}", interface.interface_type());
//!         
//!         // Display traffic statistics
//!         println!("  Bytes received: {}", interface.bytes_received());
//!         println!("  Bytes sent: {}", interface.bytes_sent());
//!         println!("  Packets received: {}", interface.packets_received());
//!         println!("  Packets sent: {}", interface.packets_sent());
//!         
//!         // Display error statistics
//!         println!("  Receive errors: {}", interface.receive_errors());
//!         println!("  Send errors: {}", interface.send_errors());
//!         println!("  Collisions: {}", interface.collisions());
//!         
//!         // If the interface has addresses, display them
//!         if let Some(addresses) = interface.addresses() {
//!             println!("  Addresses:");
//!             for addr in addresses {
//!                 println!("    {}", addr);
//!             }
//!         }
//!     }
//!     
//!     // Get information about a specific interface by name
//!     if let Some(en0) = network.get_interface("en0") {
//!         println!("en0 download speed: {} bytes/s", en0.download_speed());
//!         println!("en0 upload speed: {} bytes/s", en0.upload_speed());
//!     }
//!     
//!     Ok(())
//! }
//! ```

mod interface;
mod traffic;

pub use interface::{Interface, InterfaceType, NetworkManager};

/// Trait defining the standard interface for accessing network metrics.
///
/// This trait provides a consistent API for retrieving common network metrics
/// regardless of the specific network interface or implementation details.
pub trait NetworkMetrics {
    /// Returns the total number of bytes received.
    fn bytes_received(&self) -> u64;
    
    /// Returns the total number of bytes sent.
    fn bytes_sent(&self) -> u64;
    
    /// Returns the total number of packets received.
    fn packets_received(&self) -> u64;
    
    /// Returns the total number of packets sent.
    fn packets_sent(&self) -> u64;
    
    /// Returns the number of receive errors.
    fn receive_errors(&self) -> u64;
    
    /// Returns the number of send errors.
    fn send_errors(&self) -> u64;
    
    /// Returns the number of packet collisions.
    fn collisions(&self) -> u64;
    
    /// Returns the current download speed in bytes per second.
    /// This requires multiple calls to update() over time to calculate.
    fn download_speed(&self) -> f64;
    
    /// Returns the current upload speed in bytes per second.
    /// This requires multiple calls to update() over time to calculate.
    fn upload_speed(&self) -> f64;
    
    /// Returns true if the network interface is currently active.
    fn is_active(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::interface::{Interface, InterfaceType};
    use std::collections::HashMap;
    
    // Create a mock network interface for testing
    fn create_mock_interface(name: &str, interface_type: InterfaceType, is_loopback: bool) -> Interface {
        let flags = if is_loopback {
            crate::utils::bindings::if_flags::IFF_UP | 
            crate::utils::bindings::if_flags::IFF_RUNNING |
            crate::utils::bindings::if_flags::IFF_LOOPBACK
        } else {
            crate::utils::bindings::if_flags::IFF_UP | 
            crate::utils::bindings::if_flags::IFF_RUNNING
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
        assert_eq!(network_manager.total_download_speed(), 0.0, "Initial download speed should be 0");
        assert_eq!(network_manager.total_upload_speed(), 0.0, "Initial upload speed should be 0");
    }
    
    // If this test runs on a real machine, it might still fail because we can't guarantee
    // the system has network interfaces. We'll skip this test in CI environments.
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
