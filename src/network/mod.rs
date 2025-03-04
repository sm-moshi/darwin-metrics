use crate::Error;

/// Network interface information
#[derive(Debug, Clone)]
pub struct Network {
    /// Interface name
    pub name: String,
    /// Whether the interface is active
    pub is_active: bool,
    /// Bytes received
    pub bytes_received: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Packets received
    pub packets_received: u64,
    /// Packets sent
    pub packets_sent: u64,
}

impl Network {
    /// Create a new Network instance
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
    pub fn get_stats() -> Result<Vec<Self>, Error> {
        Err(Error::not_implemented("Network statistics collection"))
    }
} 