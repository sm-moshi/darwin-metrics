use std::{
    collections::{HashMap, HashSet},
    ffi::CStr,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ptr,
    time::Instant,
};

use crate::{
    error::{Error, Result},
    network::{traffic::TrafficTracker, NetworkMetrics},
    utils::bindings::{
        address_family, freeifaddrs, getifaddrs, if_flags, ifaddrs, sockaddr_dl, sockaddr_in,
        sockaddr_in6,
    },
};

// Type aliases to reduce clippy::type_complexity warnings
type NetworkAddressMap = HashMap<String, (u32, Option<String>, Vec<IpAddr>)>;
type TrafficStatsMap = HashMap<String, (u64, u64, u64, u64, u64, u64, u64)>;

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
///
/// This struct encapsulates all information about a single network interface on
/// macOS, including its configuration, status, and traffic statistics. It
/// provides methods to query interface properties and monitor network activity.
///
/// Each interface tracks:
/// - Basic properties (name, type, flags)
/// - Hardware information (MAC address)
/// - Network configuration (IP addresses)
/// - Traffic statistics (bytes/packets sent/received)
/// - Performance metrics (upload/download speeds)
/// - Error statistics (errors, collisions)
///
/// The interface metrics are updated via the NetworkManager's update() method.
#[derive(Debug, Clone)]
pub struct Interface {
    /// Name of the interface (e.g., "en0", "lo0")
    name: String,

    /// Type of interface (Ethernet, WiFi, Loopback, Virtual, Other)
    interface_type: InterfaceType,

    /// Flags associated with this interface (IFF_UP, IFF_RUNNING, etc.)
    flags: u32,

    /// MAC address, if available (formatted as xx:xx:xx:xx:xx:xx)
    mac_address: Option<String>,

    /// IP addresses associated with this interface (both IPv4 and IPv6)
    addresses: Vec<IpAddr>,

    /// Traffic statistics tracker for monitoring network activity
    traffic: TrafficTracker,

    /// Timestamp of the last update for calculating rates
    last_update: Instant,
}

impl Interface {
    /// Creates a new Interface with the given name, type, and initial metrics.
    #[allow(clippy::too_many_arguments)]
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
    #[allow(clippy::too_many_arguments)]
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
        self.is_flag_set(if_flags::IFF_WIRELESS)
            || self.interface_type == InterfaceType::WiFi
            || self.name.starts_with("wl")
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
///
/// The NetworkManager is the main entry point for the network monitoring
/// functionality. It handles:
///
/// - Network interface discovery and enumeration
/// - Tracking all available network interfaces on the system
/// - Aggregating traffic statistics across interfaces
/// - Providing access to individual interface metrics
/// - Updating network statistics in real-time
///
/// This implementation is specifically designed for macOS systems and uses
/// a combination of getifaddrs() for interface discovery and netstat for
/// traffic statistics, providing a reliable and efficient way to monitor
/// network activity.
#[derive(Debug)]
pub struct NetworkManager {
    /// Map of interface names to Interface objects
    pub(crate) interfaces: HashMap<String, Interface>,
}

impl NetworkManager {
    /// Creates a new NetworkManager and initializes it with the current
    /// interfaces.
    ///
    /// This constructor:
    /// 1. Creates an empty NetworkManager instance
    /// 2. Attempts to discover all network interfaces on the system
    /// 3. Initializes traffic statistics for each interface
    /// 4. Returns a ready-to-use manager instance
    ///
    /// Even if the initialization fails to retrieve network interfaces (for
    /// example, due to permission issues), the function will still return a
    /// valid but empty NetworkManager rather than failing with an error.
    pub fn new() -> Result<Self> {
        let mut manager = Self { interfaces: HashMap::new() };

        // Try to initialize interfaces, but continue even if it fails
        if let Err(e) = manager.update() {
            log::warn!("Failed to initialize network interfaces: {}", e);
            // Continue with empty interface list rather than crashing
        }

        Ok(manager)
    }

    /// Updates all network interfaces and their metrics.
    ///
    /// This method:
    /// 1. Retrieves the current state of all network interfaces
    /// 2. Updates traffic statistics for existing interfaces
    /// 3. Adds any new interfaces that were discovered
    /// 4. Calculates current upload/download speeds based on changes since last
    ///    update
    ///
    /// For accurate speed calculations, this method should be called at regular
    /// intervals (typically every 1-5 seconds).
    pub fn update(&mut self) -> Result<()> {
        // Get network interfaces list
        let interfaces = self.get_interfaces()?;

        for interface in interfaces {
            let name = interface.name().to_string();
            self.interfaces.insert(name, interface);
        }

        Ok(())
    }

    /// Gets all discovered network interfaces.
    ///
    /// Returns a vector of references to all known network interfaces.
    /// The interfaces are returned in arbitrary order.
    ///
    /// This includes active and inactive interfaces of all types:
    /// - Physical interfaces (Ethernet, WiFi)
    /// - Virtual interfaces (VPN tunnels, bridges)
    /// - Loopback interfaces
    /// - Other system interfaces
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
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0, // Initial traffic stats are 0
                );

                interface_map.insert(name, interface);
            }
        }

        // Get existing traffic data
        let existing_traffic = self.update_traffic_stats();

        // Process traffic data if we got it
        if let Some(traffic_data) = existing_traffic {
            for (
                name,
                (rx_bytes, tx_bytes, rx_packets, tx_packets, rx_errors, tx_errors, collisions),
            ) in traffic_data
            {
                if let Some(interface) = interface_map.get_mut(&name) {
                    // Update with real traffic stats
                    interface.update_traffic(
                        rx_bytes, tx_bytes, rx_packets, tx_packets, rx_errors, tx_errors,
                        collisions,
                    );
                } else if !name.is_empty() {
                    // If we have traffic data but no interface, create a placeholder
                    let interface = Interface::new(
                        name.clone(),
                        InterfaceType::Other,
                        0, // No flags
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
    fn get_network_addresses(&self) -> Result<NetworkAddressMap> {
        // Using type alias defined at the top of the file
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
                    },
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
                        },
                        family if family == address_family::AF_INET6 => {
                            // IPv6 address
                            let addr_in6 = &*(ifa.ifa_addr as *mut sockaddr_in6);
                            let ip = Ipv6Addr::from(addr_in6.sin6_addr.s6_addr);
                            entry.2.push(IpAddr::V6(ip));
                        },
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
                        },
                        _ => {}, // Ignore other address families
                    }
                }

                current = ifa.ifa_next;
            }
        }

        Ok(result)
    }

    /// Updates traffic stats using macOS native APIs.
    ///
    /// This method tries two approaches in order:
    /// 1. Use sysctlbyname with 64-bit interface data (modern approach)
    /// 2. Fall back to netstat command-line tool if API approach fails
    fn update_traffic_stats(&self) -> Option<TrafficStatsMap> {
        // First try the native implementation using sysctlbyname
        if let Some(result) = self.update_traffic_stats_native() {
            return Some(result);
        }
        
        // If native implementation fails, fall back to netstat
        self.update_traffic_stats_netstat()
    }
    
    /// Updates traffic stats using the sysctlbyname API.
    ///
    /// This is the preferred method that directly accesses kernel network statistics
    /// rather than relying on command-line tools.
    fn update_traffic_stats_native(&self) -> Option<TrafficStatsMap> {
        use crate::utils::bindings::get_network_stats_native;
        
        // Get list of interface names
        let mut ifap: *mut ifaddrs = ptr::null_mut();
        let mut result = HashMap::new();
        
        unsafe {
            // Call getifaddrs() to get list of interfaces
            if getifaddrs(&mut ifap) != 0 {
                return None;
            }
            
            // Use scopeguard to ensure ifap is freed
            let _guard = scopeguard::guard(ifap, |ifap| {
                freeifaddrs(ifap);
            });
            
            // Collect unique interface names
            let mut interface_names = HashSet::new();
            
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
                    },
                };
                
                interface_names.insert(name);
                current = ifa.ifa_next;
            }
            
            // Get stats for each interface using sysctlbyname
            for name in interface_names {
                // Use the native sysctlbyname approach to get stats
                if let Ok(if_data) = get_network_stats_native(&name) {
                    // Extract traffic statistics
                    let rx_bytes = if_data.ifi_ibytes;
                    let tx_bytes = if_data.ifi_obytes;
                    let rx_packets = if_data.ifi_ipackets;
                    let tx_packets = if_data.ifi_opackets;
                    let rx_errors = if_data.ifi_ierrors;
                    let tx_errors = if_data.ifi_oerrors;
                    let collisions = if_data.ifi_collisions;
                    
                    // Store in result map
                    result.insert(
                        name,
                        (rx_bytes, tx_bytes, rx_packets, tx_packets, rx_errors, tx_errors, collisions),
                    );
                }
            }
        }
        
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
    
    /// Updates traffic stats using netstat command-line tool.
    ///
    /// This is a fallback method that's safer but less efficient than using sysctlbyname.
    /// It parses the output of the netstat command to get interface statistics.
    fn update_traffic_stats_netstat(&self) -> Option<TrafficStatsMap> {
        // Using type alias defined at the top of the file
        // This is a fallback method that's safer than sysctlbyname
        // On macOS, we can use netstat to get network statistics
        let output = std::process::Command::new("netstat").args(["-ib"]).output().ok()?;

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
            // Columns are: Name Mtu Network Address Ipkts Ierrs Ibytes Opkts Oerrs Obytes
            // Coll
            let rx_packets = parts[4].parse::<u64>().unwrap_or(0);
            let rx_errors = parts[5].parse::<u64>().unwrap_or(0);
            let rx_bytes = parts[6].parse::<u64>().unwrap_or(0);
            let tx_packets = parts[7].parse::<u64>().unwrap_or(0);
            let tx_errors = parts[8].parse::<u64>().unwrap_or(0);
            let tx_bytes = parts[9].parse::<u64>().unwrap_or(0);
            let collisions =
                if parts.len() > 10 { parts[10].parse::<u64>().unwrap_or(0) } else { 0 };

            result.insert(
                name,
                (rx_bytes, tx_bytes, rx_packets, tx_packets, rx_errors, tx_errors, collisions),
            );
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
        } else if name.starts_with("vnic") || name.starts_with("bridge") || name.starts_with("utun")
        {
            InterfaceType::Virtual
        } else {
            InterfaceType::Other
        }
    }

    /// Creates a new NetworkManager asynchronously and initializes it with the
    /// current interfaces.
    ///
    /// This constructor:
    /// 1. Creates an empty NetworkManager instance
    /// 2. Attempts to discover all network interfaces on the system
    ///    asynchronously
    /// 3. Initializes traffic statistics for each interface
    /// 4. Returns a ready-to-use manager instance
    ///
    /// Even if the initialization fails to retrieve network interfaces (for
    /// example, due to permission issues), the function will still return a
    /// valid but empty NetworkManager rather than failing with an error.
    pub async fn new_async() -> Result<Self> {
        let mut manager = Self { interfaces: HashMap::new() };

        // Try to initialize interfaces, but continue even if it fails
        if let Err(e) = manager.update_async().await {
            log::warn!("Failed to initialize network interfaces asynchronously: {}", e);
            // Continue with empty interface list rather than crashing
        }

        Ok(manager)
    }

    /// Updates all network interfaces and their metrics asynchronously.
    ///
    /// This method works like `update()` but is designed for use in async
    /// contexts, running potentially blocking network operations in a
    /// separate blocking task to avoid blocking the async runtime.
    ///
    /// For accurate speed calculations, this method should be called at regular
    /// intervals (typically every 1-5 seconds).
    pub async fn update_async(&mut self) -> Result<()> {
        // Use tokio to spawn a blocking task for the network operations
        // This avoids blocking the async runtime with potentially slow system calls
        let interfaces = tokio::task::spawn_blocking(move || {
            // Create a temporary manager just to use get_interfaces
            // This avoids borrowing self in the blocking task
            let temp_manager = NetworkManager { interfaces: HashMap::new() };
            temp_manager.get_interfaces()
        })
        .await
        .map_err(|e| Error::Network(format!("Task join error: {}", e)))??;

        // Update our interfaces with the results from the blocking task
        for interface in interfaces {
            let name = interface.name().to_string();
            self.interfaces.insert(name, interface);
        }

        Ok(())
    }

    /// Gets all active network interfaces asynchronously.
    ///
    /// Returns only interfaces that are currently active (UP and RUNNING).
    /// This is a convenience method for filtering interfaces by their active
    /// state.
    pub async fn active_interfaces_async(&self) -> Vec<&Interface> {
        self.interfaces.values().filter(|i| i.is_active()).collect()
    }

    /// Gets the total network throughput metrics asynchronously.
    ///
    /// Returns upload and download speeds without blocking the async runtime.
    pub async fn get_throughput_async(&self) -> Result<(f64, f64)> {
        // This operation is lightweight enough not to need a blocking task
        let total_download = self.total_download_speed();
        let total_upload = self.total_upload_speed();

        Ok((total_download, total_upload))
    }

    /// Gets network interface by name asynchronously.
    ///
    /// A convenience method that doesn't block, but provides a consistent
    /// async API alongside other async methods.
    pub async fn get_interface_async(&self, name: &str) -> Option<&Interface> {
        self.get_interface(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_interface_type_display() {
        assert_eq!(InterfaceType::Ethernet.to_string(), "Ethernet");
        assert_eq!(InterfaceType::WiFi.to_string(), "WiFi");
        assert_eq!(InterfaceType::Loopback.to_string(), "Loopback");
        assert_eq!(InterfaceType::Virtual.to_string(), "Virtual");
        assert_eq!(InterfaceType::Other.to_string(), "Other");
    }

    #[test]
    fn test_interface_creation() {
        let interface = Interface::new(
            "test0".to_string(),
            InterfaceType::Ethernet,
            if_flags::IFF_UP | if_flags::IFF_RUNNING | if_flags::IFF_BROADCAST,
            Some("00:11:22:33:44:55".to_string()),
            vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))],
            1000, // bytes_received
            2000, // bytes_sent
            10,   // packets_received
            20,   // packets_sent
            1,    // receive_errors
            2,    // send_errors
            0,    // collisions
        );

        // Test basic properties
        assert_eq!(interface.name(), "test0");
        assert_eq!(interface.interface_type(), &InterfaceType::Ethernet);
        assert_eq!(interface.mac_address(), Some("00:11:22:33:44:55"));
        assert_eq!(interface.addresses().unwrap()[0], IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));

        // Test traffic metrics
        assert_eq!(interface.bytes_received(), 1000);
        assert_eq!(interface.bytes_sent(), 2000);
        assert_eq!(interface.packets_received(), 10);
        assert_eq!(interface.packets_sent(), 20);
        assert_eq!(interface.receive_errors(), 1);
        assert_eq!(interface.send_errors(), 2);
        assert_eq!(interface.collisions(), 0);

        // Test flag methods
        assert!(interface.is_active());
        assert!(interface.supports_broadcast());
        assert!(!interface.is_loopback());
        assert!(!interface.is_point_to_point());
        assert!(!interface.is_wireless());
    }

    #[test]
    fn test_interface_update_traffic() {
        let mut interface = Interface::new(
            "test0".to_string(),
            InterfaceType::Ethernet,
            if_flags::IFF_UP | if_flags::IFF_RUNNING,
            None,
            vec![],
            1000, // bytes_received
            2000, // bytes_sent
            10,   // packets_received
            20,   // packets_sent
            1,    // receive_errors
            2,    // send_errors
            0,    // collisions
        );

        // Initial metrics
        assert_eq!(interface.bytes_received(), 1000);
        assert_eq!(interface.bytes_sent(), 2000);

        // Update traffic
        interface.update_traffic(
            2000, // bytes_received
            3000, // bytes_sent
            20,   // packets_received
            30,   // packets_sent
            2,    // receive_errors
            3,    // send_errors
            1,    // collisions
        );

        // Check updated metrics
        assert_eq!(interface.bytes_received(), 2000);
        assert_eq!(interface.bytes_sent(), 3000);
        assert_eq!(interface.packets_received(), 20);
        assert_eq!(interface.packets_sent(), 30);
        assert_eq!(interface.receive_errors(), 2);
        assert_eq!(interface.send_errors(), 3);
        assert_eq!(interface.collisions(), 1);
    }

    #[test]
    fn test_interface_wireless_detection() {
        // Test WiFi interface type
        let wifi_interface = Interface::new(
            "en0".to_string(),
            InterfaceType::WiFi,
            0,
            None,
            vec![],
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        );
        assert!(wifi_interface.is_wireless());

        // Test wireless flag
        let wireless_interface = Interface::new(
            "test0".to_string(),
            InterfaceType::Ethernet,
            if_flags::IFF_WIRELESS,
            None,
            vec![],
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        );
        assert!(wireless_interface.is_wireless());

        // Test wireless name pattern
        let wlan_interface = Interface::new(
            "wlan0".to_string(),
            InterfaceType::Other,
            0,
            None,
            vec![],
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        );
        assert!(wlan_interface.is_wireless());
    }

    #[test]
    fn test_network_manager_creation() {
        // This just tests that we can create a NetworkManager without errors
        let manager = NetworkManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_traffic_stats_implementation() {
        // This test checks that at least one of our traffic stats implementations works
        let _manager = NetworkManager::new().expect("Failed to create NetworkManager");
        
        // Create a mock NetworkManager just for testing the stats methods
        let test_manager = NetworkManager { interfaces: HashMap::new() };
        
        // Try the native implementation first
        let native_stats = test_manager.update_traffic_stats_native();
        
        if native_stats.is_some() {
            // If native implementation works, validate it
            let stats = native_stats.unwrap();
            assert!(!stats.is_empty(), "Native implementation returned empty stats");
            
            // Check that we have some common interfaces like lo0
            let lo0_stats = stats.get("lo0");
            if lo0_stats.is_some() {
                let (rx_bytes, tx_bytes, rx_packets, tx_packets, _rx_errors, _tx_errors, _collisions) = 
                    *lo0_stats.unwrap();
                
                // Basic sanity checks - loopback should have some traffic and low errors
                assert!(rx_bytes > 0, "Loopback rx_bytes should be non-zero");
                assert!(tx_bytes > 0, "Loopback tx_bytes should be non-zero");
                assert!(rx_packets > 0, "Loopback rx_packets should be non-zero");
                assert!(tx_packets > 0, "Loopback tx_packets should be non-zero");
            }
        } else {
            // If native implementation fails, check the netstat fallback
            let netstat_stats = test_manager.update_traffic_stats_netstat();
            assert!(netstat_stats.is_some(), "Both native and netstat implementations failed");
            
            let stats = netstat_stats.unwrap();
            assert!(!stats.is_empty(), "Netstat implementation returned empty stats");
        }
        
        // Verify that the combined implementation works too
        let combined_stats = test_manager.update_traffic_stats();
        assert!(combined_stats.is_some(), "Combined traffic stats implementation failed");
    }

    #[test]
    fn test_network_manager_interface_access() {
        let mut manager = NetworkManager { interfaces: HashMap::new() };

        // Add a test interface
        let interface = Interface::new(
            "test0".to_string(),
            InterfaceType::Ethernet,
            if_flags::IFF_UP | if_flags::IFF_RUNNING,
            None,
            vec![],
            1000,
            2000,
            10,
            20,
            1,
            2,
            0,
        );

        manager.interfaces.insert("test0".to_string(), interface);

        // Test interfaces() method
        let interfaces = manager.interfaces();
        assert_eq!(interfaces.len(), 1);
        assert_eq!(interfaces[0].name(), "test0");

        // Test get_interface() method
        let found = manager.get_interface("test0");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "test0");

        // Test getting a non-existent interface
        let not_found = manager.get_interface("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_determine_interface_type() {
        // Test loopback detection
        assert_eq!(
            NetworkManager::determine_interface_type("lo0", if_flags::IFF_LOOPBACK),
            InterfaceType::Loopback
        );

        // Test ethernet detection
        assert_eq!(NetworkManager::determine_interface_type("en1", 0), InterfaceType::Ethernet);

        // Test WiFi detection (en0 on macOS)
        assert_eq!(NetworkManager::determine_interface_type("en0", 0), InterfaceType::WiFi);

        // Test WiFi detection (wl prefix)
        assert_eq!(NetworkManager::determine_interface_type("wlan0", 0), InterfaceType::WiFi);

        // Test virtual interface detection
        assert_eq!(NetworkManager::determine_interface_type("vnic0", 0), InterfaceType::Virtual);
        assert_eq!(NetworkManager::determine_interface_type("bridge0", 0), InterfaceType::Virtual);
        assert_eq!(NetworkManager::determine_interface_type("utun0", 0), InterfaceType::Virtual);

        // Test other interface
        assert_eq!(NetworkManager::determine_interface_type("unknown0", 0), InterfaceType::Other);
    }

    // Mock test for speed calculation - this doesn't test actual networking
    // but just verifies the calculation logic
    #[test]
    fn test_network_speed_calculation() {
        // Create interfaces with initial traffic
        let mut interface1 = Interface::new(
            "test1".to_string(),
            InterfaceType::Ethernet,
            if_flags::IFF_UP | if_flags::IFF_RUNNING,
            None,
            vec![],
            1000, // bytes_received
            2000, // bytes_sent
            10,
            20,
            0,
            0,
            0,
        );

        let mut interface2 = Interface::new(
            "test2".to_string(),
            InterfaceType::WiFi,
            if_flags::IFF_UP | if_flags::IFF_RUNNING,
            None,
            vec![],
            3000, // bytes_received
            4000, // bytes_sent
            30,
            40,
            0,
            0,
            0,
        );

        // Sleep a bit to allow time to pass
        std::thread::sleep(Duration::from_millis(100));

        // Update with new traffic values
        interface1.update_traffic(2000, 3000, 20, 30, 0, 0, 0);
        interface2.update_traffic(5000, 7000, 50, 70, 0, 0, 0);

        // Create manager with these interfaces
        let mut manager = NetworkManager { interfaces: HashMap::new() };
        manager.interfaces.insert("test1".to_string(), interface1);
        manager.interfaces.insert("test2".to_string(), interface2);

        // Check total speeds
        let download = manager.total_download_speed();
        let upload = manager.total_upload_speed();

        // We expect non-zero speeds, but can't assert exact values
        // due to timing differences in tests
        assert!(download > 0.0);
        assert!(upload > 0.0);
    }
}
