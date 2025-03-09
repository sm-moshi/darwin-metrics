use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ptr;
use std::time::Instant;

use crate::error::{Error, Result};
use crate::network::traffic::TrafficTracker;
use crate::network::NetworkMetrics;
use crate::utils::bindings::{
    address_family, freeifaddrs, getifaddrs, if_flags, ifaddrs, sockaddr, sockaddr_dl,
    sockaddr_in, sockaddr_in6,
};

/// Represents the type of network interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceType {
    /// Ethernet interface
    Ethernet,
    /// WiFi interface
    WiFi,
    /// Loopback interface
    Loopback,
    /// Virtual interface
    Virtual,
    /// Other/unknown interface type
    Other,
}

impl std::fmt::Display for InterfaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterfaceType::Ethernet => write!(f, "Ethernet"),
            InterfaceType::WiFi => write!(f, "WiFi"),
            InterfaceType::Loopback => write!(f, "Loopback"),
            InterfaceType::Virtual => write!(f, "Virtual"),
            InterfaceType::Other => write!(f, "Other"),
        }
    }
}

/// Represents a network interface with its associated metrics and properties.
#[derive(Debug, Clone)]
pub struct Interface {
    /// Name of the interface (e.g., "en0", "lo0")
    name: String,
    
    /// Type of interface
    interface_type: InterfaceType,
    
    /// Flags associated with this interface
    flags: u32,
    
    /// MAC address, if available
    mac_address: Option<String>,
    
    /// IP addresses associated with this interface
    addresses: Vec<IpAddr>,
    
    /// Traffic statistics tracker
    traffic: TrafficTracker,
    
    /// Last update time
    last_update: Instant,
}

impl Interface {
    /// Creates a new Interface with the given name, type, and initial metrics.
    pub fn new(
        name: String,
        interface_type: InterfaceType,
        flags: u32,
        mac_address: Option<String>,
        addresses: Vec<IpAddr>,
        bytes_received: u64,
        bytes_sent: u64,
        packets_received: u64,
        packets_sent: u64,
        receive_errors: u64,
        send_errors: u64,
        collisions: u64,
    ) -> Self {
        Self {
            name,
            interface_type,
            flags,
            mac_address,
            addresses,
            traffic: TrafficTracker::new(
                bytes_received,
                bytes_sent,
                packets_received,
                packets_sent,
                receive_errors,
                send_errors,
                collisions,
            ),
            last_update: Instant::now(),
        }
    }
    
    /// Get the name of this interface
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the type of this interface
    pub fn interface_type(&self) -> &InterfaceType {
        &self.interface_type
    }
    
    /// Get the MAC address of this interface, if available
    pub fn mac_address(&self) -> Option<&str> {
        self.mac_address.as_deref()
    }
    
    /// Get the IP addresses associated with this interface
    pub fn addresses(&self) -> Option<&[IpAddr]> {
        if self.addresses.is_empty() {
            None
        } else {
            Some(&self.addresses)
        }
    }
    
    /// Updates the traffic statistics for this interface.
    pub fn update_traffic(
        &mut self,
        bytes_received: u64,
        bytes_sent: u64,
        packets_received: u64,
        packets_sent: u64,
        receive_errors: u64,
        send_errors: u64,
        collisions: u64,
    ) {
        self.traffic.update(
            bytes_received,
            bytes_sent,
            packets_received,
            packets_sent,
            receive_errors,
            send_errors,
            collisions,
        );
        self.last_update = Instant::now();
    }
    
    /// Determines if the interface is active based on its flags.
    fn is_flag_set(&self, flag: u32) -> bool {
        (self.flags & flag) == flag
    }
    
    /// Gets the packet receive rate in packets per second.
    pub fn packet_receive_rate(&self) -> f64 {
        self.traffic.packet_receive_rate()
    }
    
    /// Gets the packet send rate in packets per second.
    pub fn packet_send_rate(&self) -> f64 {
        self.traffic.packet_send_rate()
    }
    
    /// Gets the receive error rate (errors per packet).
    pub fn receive_error_rate(&self) -> f64 {
        self.traffic.receive_error_rate()
    }
    
    /// Gets the send error rate (errors per packet).
    pub fn send_error_rate(&self) -> f64 {
        self.traffic.send_error_rate()
    }
    
    /// Gets whether this is a loopback interface.
    pub fn is_loopback(&self) -> bool {
        self.is_flag_set(if_flags::IFF_LOOPBACK)
    }
    
    /// Gets whether this interface supports broadcast.
    pub fn supports_broadcast(&self) -> bool {
        self.is_flag_set(if_flags::IFF_BROADCAST)
    }
    
    /// Gets whether this interface supports multicast.
    pub fn supports_multicast(&self) -> bool {
        self.is_flag_set(if_flags::IFF_MULTICAST)
    }
    
    /// Gets whether this is a point-to-point interface.
    pub fn is_point_to_point(&self) -> bool {
        self.is_flag_set(if_flags::IFF_POINTOPOINT)
    }
    
    /// Gets whether this is a wireless interface.
    pub fn is_wireless(&self) -> bool {
        self.is_flag_set(if_flags::IFF_WIRELESS) ||
        self.interface_type == InterfaceType::WiFi ||
        self.name.starts_with("wl")
    }
}

impl NetworkMetrics for Interface {
    fn bytes_received(&self) -> u64 {
        self.traffic.bytes_received()
    }
    
    fn bytes_sent(&self) -> u64 {
        self.traffic.bytes_sent()
    }
    
    fn packets_received(&self) -> u64 {
        self.traffic.packets_received()
    }
    
    fn packets_sent(&self) -> u64 {
        self.traffic.packets_sent()
    }
    
    fn receive_errors(&self) -> u64 {
        self.traffic.receive_errors()
    }
    
    fn send_errors(&self) -> u64 {
        self.traffic.send_errors()
    }
    
    fn collisions(&self) -> u64 {
        self.traffic.collisions()
    }
    
    fn download_speed(&self) -> f64 {
        self.traffic.download_speed()
    }
    
    fn upload_speed(&self) -> f64 {
        self.traffic.upload_speed()
    }
    
    fn is_active(&self) -> bool {
        self.is_flag_set(if_flags::IFF_UP) && self.is_flag_set(if_flags::IFF_RUNNING)
    }
}

/// Manages network interfaces and provides access to network metrics.
#[derive(Debug)]
pub struct NetworkManager {
    /// Map of interface names to Interface objects
    pub(crate) interfaces: HashMap<String, Interface>,
}

impl NetworkManager {
    /// Creates a new NetworkManager and initializes it with the current interfaces.
    pub fn new() -> Result<Self> {
        let mut manager = Self {
            interfaces: HashMap::new(),
        };
        
        // Try to initialize interfaces, but continue even if it fails
        if let Err(e) = manager.update() {
            log::warn!("Failed to initialize network interfaces: {}", e);
            // Continue with empty interface list rather than crashing
        }
        
        Ok(manager)
    }
    
    /// Updates all network interfaces and their metrics.
    pub fn update(&mut self) -> Result<()> {
        // Get network interfaces list
        let interfaces = self.get_interfaces()?;
        
        for interface in interfaces {
            let name = interface.name().to_string();
            self.interfaces.insert(name, interface);
        }
        
        Ok(())
    }
    
    /// Gets all network interfaces.
    pub fn interfaces(&self) -> Vec<&Interface> {
        self.interfaces.values().collect()
    }
    
    /// Gets a specific interface by name.
    pub fn get_interface(&self, name: &str) -> Option<&Interface> {
        self.interfaces.get(name)
    }
    
    /// Gets the total download speed across all interfaces.
    pub fn total_download_speed(&self) -> f64 {
        self.interfaces.values().map(|i| i.download_speed()).sum()
    }
    
    /// Gets the total upload speed across all interfaces.
    pub fn total_upload_speed(&self) -> f64 {
        self.interfaces.values().map(|i| i.upload_speed()).sum()
    }
    
    /// Gets network interfaces using the getifaddrs() system call.
    fn get_interfaces(&self) -> Result<Vec<Interface>> {
        // Store network interfaces
        let mut interfaces = Vec::new();
        
        // First get addresses from getifaddrs()
        let addresses = self.get_network_addresses()?;
        
        // Create a map to store interface details
        let mut interface_map: HashMap<String, Interface> = HashMap::new();
        
        // Process all addresses
        for (name, data) in addresses {
            // If this interface is already in the map, update it
            let (flags, mac_addr, ip_addrs) = data;
            
            if let Some(existing) = interface_map.get_mut(&name) {
                // Only update mac_address if it's currently None and new address exists
                if existing.mac_address.is_none() && mac_addr.is_some() {
                    existing.mac_address = mac_addr;
                }
                
                // Add all new IP addresses
                for addr in ip_addrs {
                    if !existing.addresses.contains(&addr) {
                        existing.addresses.push(addr);
                    }
                }
            } else {
                // Create new interface
                let interface_type = Self::determine_interface_type(&name, flags);
                
                let interface = Interface::new(
                    name.clone(),
                    interface_type,
                    flags,
                    mac_addr,
                    ip_addrs,
                    0, 0, 0, 0, 0, 0, 0, // Initial traffic stats are 0
                );
                
                interface_map.insert(name, interface);
            }
        }
        
        // Get existing traffic data
        let existing_traffic = self.update_traffic_stats();
        
        // Process traffic data if we got it
        if let Some(traffic_data) = existing_traffic {
            for (name, (rx_bytes, tx_bytes, rx_packets, tx_packets, rx_errors, tx_errors, collisions)) in traffic_data {
                if let Some(interface) = interface_map.get_mut(&name) {
                    // Update with real traffic stats
                    interface.update_traffic(
                        rx_bytes,
                        tx_bytes,
                        rx_packets,
                        tx_packets,
                        rx_errors,
                        tx_errors,
                        collisions,
                    );
                } else if !name.is_empty() {
                    // If we have traffic data but no interface, create a placeholder
                    let interface = Interface::new(
                        name.clone(),
                        InterfaceType::Other,
                        0,  // No flags
                        None,
                        Vec::new(),
                        rx_bytes,
                        tx_bytes,
                        rx_packets,
                        tx_packets,
                        rx_errors,
                        tx_errors,
                        collisions,
                    );
                    
                    interface_map.insert(name, interface);
                }
            }
        }
        
        // Convert map to vector
        for (_, interface) in interface_map {
            interfaces.push(interface);
        }
        
        Ok(interfaces)
    }
    
    /// Gets network addresses using getifaddrs().
    fn get_network_addresses(&self) -> Result<HashMap<String, (u32, Option<String>, Vec<IpAddr>)>> {
        let mut result = HashMap::new();
        let mut ifap: *mut ifaddrs = ptr::null_mut();
        
        unsafe {
            // Call getifaddrs() to get list of interfaces
            if getifaddrs(&mut ifap) != 0 {
                return Err(Error::Network("Failed to get network interfaces".to_string()));
            }
            
            // Use scopeguard to ensure ifap is freed
            let _guard = scopeguard::guard(ifap, |ifap| {
                freeifaddrs(ifap);
            });
            
            // Iterate through linked list of interfaces
            let mut current = ifap;
            while !current.is_null() {
                let ifa = &*current;
                
                // Skip entries with null names
                if ifa.ifa_name.is_null() {
                    current = ifa.ifa_next;
                    continue;
                }
                
                // Get name as string
                let name = match CStr::from_ptr(ifa.ifa_name).to_str() {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        current = ifa.ifa_next;
                        continue;
                    }
                };
                
                // Skip empty names
                if name.is_empty() {
                    current = ifa.ifa_next;
                    continue;
                }
                
                // Get or create entry in result map
                let entry = result.entry(name).or_insert_with(|| (ifa.ifa_flags, None, Vec::new()));
                
                // Process address if available
                if !ifa.ifa_addr.is_null() {
                    let addr = &*ifa.ifa_addr;
                    
                    match addr.sa_family {
                        family if family == address_family::AF_INET => {
                            // IPv4 address
                            let addr_in = &*(ifa.ifa_addr as *mut sockaddr_in);
                            let ip = Ipv4Addr::from(u32::from_be(addr_in.sin_addr.s_addr));
                            entry.2.push(IpAddr::V4(ip));
                        }
                        family if family == address_family::AF_INET6 => {
                            // IPv6 address
                            let addr_in6 = &*(ifa.ifa_addr as *mut sockaddr_in6);
                            let ip = Ipv6Addr::from(addr_in6.sin6_addr.s6_addr);
                            entry.2.push(IpAddr::V6(ip));
                        }
                        family if family == address_family::AF_LINK => {
                            // MAC address
                            let addr_dl = &*(ifa.ifa_addr as *mut sockaddr_dl);
                            
                            let mac_len = addr_dl.sdl_alen as usize;
                            if mac_len == 6 {
                                let offset = addr_dl.sdl_nlen as usize;
                                
                                // Safety check for buffer size
                                if offset + mac_len <= addr_dl.sdl_data.len() {
                                    let mac_bytes = &addr_dl.sdl_data[offset..offset + mac_len];
                                    
                                    // Format MAC address
                                    let mac = format!(
                                        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                                        mac_bytes[0] as u8,
                                        mac_bytes[1] as u8,
                                        mac_bytes[2] as u8,
                                        mac_bytes[3] as u8,
                                        mac_bytes[4] as u8,
                                        mac_bytes[5] as u8
                                    );
                                    
                                    entry.1 = Some(mac);
                                }
                            }
                        }
                        _ => {} // Ignore other address families
                    }
                }
                
                current = ifa.ifa_next;
            }
        }
        
        Ok(result)
    }
    
    /// Updates traffic stats using a safer approach.
    ///
    /// Instead of using sysctlbyname which is causing issues,
    /// we'll use command line tools and parse the output.
    fn update_traffic_stats(&self) -> Option<HashMap<String, (u64, u64, u64, u64, u64, u64, u64)>> {
        // This is a fallback method that's safer than sysctlbyname
        // On macOS, we can use netstat to get network statistics
        let output = std::process::Command::new("netstat")
            .args(&["-ib"])
            .output()
            .ok()?;
        
        if !output.status.success() {
            return None;
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse the output
        let mut result = HashMap::new();
        
        // Skip the header line
        let lines = output_str.lines().skip(1);
        
        for line in lines {
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            // Check if the line has enough parts
            if parts.len() < 10 {
                continue;
            }
            
            // Get interface name
            let name = parts[0].to_string();
            
            // Try to parse network stats
            // Columns are: Name Mtu Network Address Ipkts Ierrs Ibytes Opkts Oerrs Obytes Coll
            let rx_packets = parts[4].parse::<u64>().unwrap_or(0);
            let rx_errors = parts[5].parse::<u64>().unwrap_or(0);
            let rx_bytes = parts[6].parse::<u64>().unwrap_or(0);
            let tx_packets = parts[7].parse::<u64>().unwrap_or(0);
            let tx_errors = parts[8].parse::<u64>().unwrap_or(0);
            let tx_bytes = parts[9].parse::<u64>().unwrap_or(0);
            let collisions = if parts.len() > 10 { parts[10].parse::<u64>().unwrap_or(0) } else { 0 };
            
            result.insert(name, (rx_bytes, tx_bytes, rx_packets, tx_packets, rx_errors, tx_errors, collisions));
        }
        
        Some(result)
    }
    
    /// Determines the type of interface based on its name and flags.
    fn determine_interface_type(name: &str, flags: u32) -> InterfaceType {
        if (flags & if_flags::IFF_LOOPBACK) != 0 {
            InterfaceType::Loopback
        } else if name.starts_with("en") {
            // On macOS, en0 is usually WiFi
            if name == "en0" {
                InterfaceType::WiFi
            } else {
                InterfaceType::Ethernet
            }
        } else if name.starts_with("wl") || name.starts_with("ath") {
            InterfaceType::WiFi
        } else if name.starts_with("vnic") || name.starts_with("bridge") || name.starts_with("utun") {
            InterfaceType::Virtual
        } else {
            InterfaceType::Other
        }
    }
}