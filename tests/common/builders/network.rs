use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

use crate::{
    network::{Interface, InterfaceType, NetworkManager},
    error::Result,
    utils::bindings::if_flags,
};

/// Builder for creating test network interfaces and managers
pub struct TestNetworkBuilder {
    interfaces: HashMap<String, Interface>,
}

impl TestNetworkBuilder {
    /// Create a new TestNetworkBuilder
    pub fn new() -> Self {
        let mut builder = Self {
            interfaces: HashMap::new(),
        };
        
        // Add a default loopback interface
        builder.with_loopback();
        
        builder
    }
    
    /// Add a loopback interface
    pub fn with_loopback(self) -> Self {
        self.with_interface("lo0", InterfaceType::Loopback)
            .with_flags_for("lo0", if_flags::IFF_UP | if_flags::IFF_RUNNING | if_flags::IFF_LOOPBACK)
    }
    
    /// Add an interface with the specified name and type
    pub fn with_interface(mut self, name: &str, interface_type: InterfaceType) -> Self {
        let interface = Interface::new(
            name.to_string(),
            interface_type,
            if_flags::IFF_UP | if_flags::IFF_RUNNING,
            Some("00:00:00:00:00:00".to_string()),
            vec![IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))],
            1000, // bytes_received
            2000, // bytes_sent
            100,  // packets_received
            200,  // packets_sent
            0,    // receive_errors
            0,    // send_errors
            0,    // collisions
        );
        
        self.interfaces.insert(name.to_string(), interface);
        self
    }
    
    /// Set flags for a specific interface
    pub fn with_flags_for(mut self, name: &str, flags: u32) -> Self {
        if let Some(interface) = self.interfaces.get_mut(name) {
            let mut updated_interface = Interface::new(
                interface.name().to_string(),
                interface.interface_type().clone(),
                flags,
                interface.mac_address().map(|s| s.to_string()),
                interface.addresses().unwrap_or(&[]).to_vec(),
                interface.bytes_received(),
                interface.bytes_sent(),
                interface.packets_received(),
                interface.packets_sent(),
                interface.receive_errors(),
                interface.send_errors(),
                interface.collisions(),
            );
            
            self.interfaces.insert(name.to_string(), updated_interface);
        }
        
        self
    }
    
    /// Build a NetworkManager with the configured interfaces
    pub fn build_manager(self) -> Result<NetworkManager> {
        Ok(NetworkManager { interfaces: self.interfaces })
    }
} 