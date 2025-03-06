//! Network interface metrics and information for macOS systems.
//!
//! This module provides functionality to gather network-related metrics and information
//! on macOS systems. It supports monitoring of:
//! - Network interface status and activity
//! - Bytes and packets transmitted/received
//! - Interface errors and dropped packets
//!
//! # Examples
//!
//! ```no_run
//! use darwin_metrics::network::Network;
//!
//! fn main() -> darwin_metrics::Result<()> {
//!     // Get statistics for all network interfaces
//!     let interfaces = Network::get_stats()?;
//!     
//!     for interface in interfaces {
//!         println!("Interface: {}", interface.name);
//!         println!("  Status: {}", if interface.is_active { "Active" } else { "Inactive" });
//!         println!("  Received: {} bytes ({} packets)", interface.bytes_received, interface.packets_received);
//!         println!("  Sent: {} bytes ({} packets)", interface.bytes_sent, interface.packets_sent);
//!     }
//!     Ok(())
//! }
//!

use crate::Error;

/// Represents network interface information and metrics
///
/// This struct provides access to various network interface metrics, including:
/// - Interface name and status
/// - Bytes and packets transmitted/received
///
/// # Examples
///
/// ```
/// use darwin_metrics::network::Network;
///
/// let interface = Network::new("en0");
/// println!("Interface: {}", interface.name);
/// ```
#[derive(Debug, Clone)]
pub struct Network {
    /// Network interface name (e.g., "en0", "lo0")
    pub name: String,
    /// Whether the interface is active
    pub is_active: bool,
    /// Total bytes received
    pub bytes_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total packets received
    pub packets_received: u64,
    /// Total packets sent
    pub packets_sent: u64,
}

impl Network {
    /// Create a new Network instance
    ///
    /// # Arguments
    /// * `name` - Network interface name
    ///
    /// # Examples
    ///
    /// ```
    /// use darwin_metrics::network::Network;
    ///
    /// let interface = Network::new("en0");
    /// assert_eq!(interface.name, "en0");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_active: false,
            bytes_received: 0,
            bytes_sent: 0,
            packets_received: 0,
            packets_sent: 0,
        }
    }

    /// Get network interface statistics
    ///
    /// This method retrieves statistics for all network interfaces in the system.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Vec<Network>>` which is:
    /// - `Ok(Vec<Network>)` containing information for all network interfaces
    /// - `Err(Error)` if the information cannot be retrieved
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use darwin_metrics::network::Network;
    ///
    /// match Network::get_stats() {
    ///     Ok(interfaces) => {
    ///         for interface in interfaces {
    ///             println!("Interface: {}", interface.name);
    ///         }
    ///     }
    ///     Err(err) => eprintln!("Failed to get network stats: {}", err),
    /// }
    /// ```
    pub fn get_stats() -> Result<Vec<Self>, Error> {
        Err(Error::not_implemented("Network statistics collection"))
    }
}
