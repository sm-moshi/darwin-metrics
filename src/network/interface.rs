use std::collections::{HashMap, HashSet};
use std::ffi::CStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ptr;

use crate::core::metrics::hardware::{
    NetworkBandwidthMonitor, NetworkErrorMonitor, NetworkInterfaceMonitor, NetworkPacketMonitor,
};
use crate::error::{Error, Result};
use crate::utils::bindings::{
    address_family, freeifaddrs, getifaddrs, if_flags, ifaddrs, sockaddr_dl, sockaddr_in, sockaddr_in6,
};
use crate::utils::ffi::bindings::get_network_stats_native;

/// Macro to define flag-checking methods for network interface flags.
/// Each method is generated with proper documentation and returns a boolean
/// indicating whether the specified flag is set.
macro_rules! define_flag_methods {
    ($(
        $(#[$attr:meta])*
        $fn_name:ident => $flag:expr
    ),* $(,)?) => {
        impl Interface {
            $(
                $(#[$attr])*
                pub fn $fn_name(&self) -> bool {
                    self.is_flag_set($flag)
                }
            )*
        }
    };
}

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

/// Represents a network interface with its properties and metrics
#[derive(Debug, Clone)]
pub struct Interface {
    /// Name of the network interface (e.g., "en0", "lo0")
    name: Option<String>,
    /// Type of network interface (Ethernet, WiFi, etc.)
    interface_type: Option<InterfaceType>,
    /// Interface flags containing status information
    flags: u32,
    /// MAC address of the interface as a formatted string
    mac_address: Option<String>,
    /// List of IP addresses assigned to this interface
    addresses: Vec<IpAddr>,
    /// Total bytes received on this interface since boot
    bytes_received: u64,
    /// Total bytes sent on this interface since boot
    bytes_sent: u64,
    /// Total packets received on this interface since boot
    packets_received: u64,
    /// Total packets sent on this interface since boot
    packets_sent: u64,
    /// Count of receive errors encountered on this interface
    receive_errors: u64,
    /// Count of send errors encountered on this interface
    send_errors: u64,
    /// Number of packet collisions detected on this interface
    collisions: u64,
}

/// Struct to hold traffic statistics parameters
pub struct TrafficStatsParams {
    /// Total number of bytes received on the interface
    pub bytes_received: u64,
    /// Total number of bytes sent from the interface
    pub bytes_sent: u64,
    /// Total number of packets received on the interface
    pub packets_received: u64,
    /// Total number of packets sent from the interface
    pub packets_sent: u64,
    /// Number of receive errors encountered on the interface
    pub receive_errors: u64,
    /// Number of send errors encountered on the interface
    pub send_errors: u64,
    /// Number of packet collisions detected on the interface
    pub collisions: u64,
}

/// Builder for creating Interface instances with a fluent API.
#[derive(Debug, Default)]
pub struct InterfaceBuilder {
    name: Option<String>,
    interface_type: Option<InterfaceType>,
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
}

impl InterfaceBuilder {
    /// Creates a new InterfaceBuilder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the interface name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the interface type.
    pub fn interface_type(mut self, interface_type: InterfaceType) -> Self {
        self.interface_type = Some(interface_type);
        self
    }

    /// Sets the interface flags.
    pub fn flags(mut self, flags: u32) -> Self {
        self.flags = flags;
        self
    }

    /// Sets the MAC address.
    pub fn mac_address(mut self, mac_address: impl Into<String>) -> Self {
        self.mac_address = Some(mac_address.into());
        self
    }

    /// Adds an IP address to the interface.
    pub fn add_address(mut self, addr: IpAddr) -> Self {
        self.addresses.push(addr);
        self
    }

    /// Sets multiple IP addresses at once.
    pub fn addresses(mut self, addresses: Vec<IpAddr>) -> Self {
        self.addresses = addresses;
        self
    }

    /// Sets traffic statistics.
    pub fn traffic_stats(mut self, params: TrafficStatsParams) -> Self {
        self.bytes_received = params.bytes_received;
        self.bytes_sent = params.bytes_sent;
        self.packets_received = params.packets_received;
        self.packets_sent = params.packets_sent;
        self.receive_errors = params.receive_errors;
        self.send_errors = params.send_errors;
        self.collisions = params.collisions;
        self
    }

    /// Builds the Interface instance.
    ///
    /// # Errors
    /// Returns an error if required fields (name, interface_type) are not set.
    pub fn build(self) -> Result<Interface> {
        let name = self
            .name
            .ok_or_else(|| Error::invalid_data("Interface name is required", None as Option<&str>))?;
        let interface_type = self
            .interface_type
            .ok_or_else(|| Error::invalid_data("Interface type is required", None as Option<&str>))?;

        Ok(Interface {
            name: Some(name),
            interface_type: Some(interface_type),
            flags: self.flags,
            mac_address: self.mac_address,
            addresses: self.addresses,
            bytes_received: self.bytes_received,
            bytes_sent: self.bytes_sent,
            packets_received: self.packets_received,
            packets_sent: self.packets_sent,
            receive_errors: self.receive_errors,
            send_errors: self.send_errors,
            collisions: self.collisions,
        })
    }
}

impl Interface {
    /// Returns a new InterfaceBuilder instance.
    pub fn builder() -> InterfaceBuilder {
        InterfaceBuilder::new()
    }

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
            name: Some(name),
            interface_type: Some(interface_type),
            flags,
            mac_address,
            addresses,
            bytes_received,
            bytes_sent,
            packets_received,
            packets_sent,
            receive_errors,
            send_errors,
            collisions,
        }
    }

    /// Get the name of this interface
    pub fn name(&self) -> &str {
        &self.name.as_ref().unwrap()
    }

    /// Get the type of this interface
    pub fn interface_type(&self) -> &InterfaceType {
        &self.interface_type.as_ref().unwrap()
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
        self.bytes_received += bytes_received;
        self.bytes_sent += bytes_sent;
        self.packets_received += packets_received;
        self.packets_sent += packets_sent;
        self.receive_errors += receive_errors;
        self.send_errors += send_errors;
        self.collisions += collisions;
    }

    /// Determines if the interface is active based on its flags.
    fn is_flag_set(&self, flag: u32) -> bool {
        (self.flags & flag) == flag
    }

    /// Gets the packet receive rate in packets per second.
    pub fn packet_receive_rate(&self) -> f64 {
        self.packets_received as f64 / self.receive_errors as f64
    }

    /// Gets the packet send rate in packets per second.
    pub fn packet_send_rate(&self) -> f64 {
        self.packets_sent as f64 / self.send_errors as f64
    }

    /// Gets the receive error rate (errors per packet).
    pub fn receive_error_rate(&self) -> f64 {
        self.receive_errors as f64 / self.packets_received as f64
    }

    /// Gets the send error rate (errors per packet).
    pub fn send_error_rate(&self) -> f64 {
        self.send_errors as f64 / self.packets_sent as f64
    }

    /// Returns a monitor for interface state
    pub fn interface_monitor(&self) -> InterfaceStateMonitor {
        InterfaceStateMonitor {
            interface: self.clone(),
        }
    }

    /// Returns a monitor for bandwidth metrics
    pub fn bandwidth_monitor(&self) -> InterfaceBandwidthMonitor {
        InterfaceBandwidthMonitor {
            interface: self.clone(),
        }
    }

    /// Returns a monitor for packet metrics
    pub fn packet_monitor(&self) -> InterfacePacketMonitor {
        InterfacePacketMonitor {
            interface: self.clone(),
        }
    }

    /// Returns a monitor for error metrics
    pub fn error_monitor(&self) -> InterfaceErrorMonitor {
        InterfaceErrorMonitor {
            interface: self.clone(),
        }
    }

    /// Check if this is a WiFi interface
    pub fn is_wifi(&self) -> bool {
        self.interface_type == Some(InterfaceType::WiFi)
    }
}

/// Monitor for network interface state
pub struct InterfaceStateMonitor {
    interface: Interface,
}

/// Monitor for network bandwidth metrics
pub struct InterfaceBandwidthMonitor {
    interface: Interface,
}

/// Monitor for network packet metrics
pub struct InterfacePacketMonitor {
    interface: Interface,
}

/// Monitor for network error metrics
pub struct InterfaceErrorMonitor {
    interface: Interface,
}

#[async_trait::async_trait]
impl NetworkInterfaceMonitor for InterfaceStateMonitor {
    async fn is_active(&self) -> Result<bool> {
        Ok(self.interface.is_flag_set(if_flags::IFF_UP) && self.interface.is_flag_set(if_flags::IFF_RUNNING))
    }

    async fn supports_broadcast(&self) -> Result<bool> {
        Ok(self.interface.is_flag_set(if_flags::IFF_BROADCAST))
    }

    async fn supports_multicast(&self) -> Result<bool> {
        Ok(self.interface.is_flag_set(if_flags::IFF_MULTICAST))
    }

    async fn is_loopback(&self) -> Result<bool> {
        Ok(self.interface.is_flag_set(if_flags::IFF_LOOPBACK))
    }

    async fn is_wireless(&self) -> Result<bool> {
        Ok(self.interface.is_flag_set(if_flags::IFF_WIRELESS)
            || self.interface.interface_type == Some(InterfaceType::WiFi)
            || self.interface.name.as_ref().unwrap().starts_with("wl"))
    }

    async fn interface_type(&self) -> Result<String> {
        Ok(self.interface.interface_type.as_ref().unwrap().to_string())
    }

    async fn mac_address(&self) -> Result<Option<String>> {
        Ok(self.interface.mac_address.clone())
    }
}

#[async_trait::async_trait]
impl NetworkBandwidthMonitor for InterfaceBandwidthMonitor {
    async fn bytes_received(&self) -> Result<u64> {
        Ok(self.interface.bytes_received)
    }

    async fn bytes_sent(&self) -> Result<u64> {
        Ok(self.interface.bytes_sent)
    }

    async fn download_speed(&self) -> Result<f64> {
        Ok(self.interface.bytes_received as f64 / self.interface.receive_errors as f64)
    }

    async fn upload_speed(&self) -> Result<f64> {
        Ok(self.interface.bytes_sent as f64 / self.interface.send_errors as f64)
    }
}

#[async_trait::async_trait]
impl NetworkPacketMonitor for InterfacePacketMonitor {
    async fn packets_received(&self) -> Result<u64> {
        Ok(self.interface.packets_received)
    }

    async fn packets_sent(&self) -> Result<u64> {
        Ok(self.interface.packets_sent)
    }

    async fn packet_receive_rate(&self) -> Result<f64> {
        Ok(self.interface.packets_received as f64 / self.interface.receive_errors as f64)
    }

    async fn packet_send_rate(&self) -> Result<f64> {
        Ok(self.interface.packets_sent as f64 / self.interface.send_errors as f64)
    }
}

#[async_trait::async_trait]
impl NetworkErrorMonitor for InterfaceErrorMonitor {
    async fn receive_errors(&self) -> Result<u64> {
        Ok(self.interface.receive_errors)
    }

    async fn send_errors(&self) -> Result<u64> {
        Ok(self.interface.send_errors)
    }

    async fn collisions(&self) -> Result<u64> {
        Ok(self.interface.collisions)
    }

    async fn receive_error_rate(&self) -> Result<f64> {
        Ok(self.interface.receive_errors as f64 / self.interface.packets_received as f64)
    }

    async fn send_error_rate(&self) -> Result<f64> {
        Ok(self.interface.send_errors as f64 / self.interface.packets_sent as f64)
    }
}

/// Manages network interfaces and provides access to network metrics.
///
/// The NetworkManager is the main entry point for the network monitoring functionality. It handles:
///
/// - Network interface discovery and enumeration
/// - Tracking all available network interfaces on the system
/// - Aggregating traffic statistics across interfaces
/// - Providing access to individual interface metrics
/// - Updating network statistics in real-time
///
/// This implementation is specifically designed for macOS systems and uses a combination of getifaddrs() for interface
/// discovery and netstat for traffic statistics, providing a reliable and efficient way to monitor network activity.
#[derive(Debug)]
pub struct NetworkManager {
    /// Map of interface names to Interface objects
    pub(crate) interfaces: HashMap<String, Interface>,
}

impl NetworkManager {
    /// Creates a new NetworkManager and initializes it with the current interfaces.
    ///
    /// This constructor:
    /// 1. Creates an empty NetworkManager instance
    /// 2. Attempts to discover all network interfaces on the system
    /// 3. Initializes traffic statistics for each interface
    /// 4. Returns a ready-to-use manager instance
    ///
    /// Even if the initialization fails to retrieve network interfaces (for example, due to permission issues), the
    /// function will still return a valid but empty NetworkManager rather than failing with an error.
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
    ///
    /// This method:
    /// 1. Retrieves the current state of all network interfaces
    /// 2. Updates traffic statistics for existing interfaces
    /// 3. Adds any new interfaces that were discovered
    /// 4. Calculates current upload/download speeds based on changes since last update
    ///
    /// For accurate speed calculations, this method should be called at regular intervals (typically every 1-5
    /// seconds).
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
    /// Returns a vector of references to all known network interfaces. The interfaces are returned in arbitrary order.
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
        self.interfaces
            .values()
            .map(|i| i.bytes_received as f64 / i.receive_errors as f64)
            .sum()
    }

    /// Gets the total upload speed across all interfaces.
    pub fn total_upload_speed(&self) -> f64 {
        self.interfaces
            .values()
            .map(|i| i.bytes_sent as f64 / i.send_errors as f64)
            .sum()
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

                let interface = Interface::builder()
                    .name(name.clone())
                    .interface_type(interface_type)
                    .flags(flags)
                    .mac_address(mac_addr.unwrap_or_default())
                    .addresses(ip_addrs)
                    .traffic_stats(TrafficStatsParams {
                        bytes_received: 0,
                        bytes_sent: 0,
                        packets_received: 0,
                        packets_sent: 0,
                        receive_errors: 0,
                        send_errors: 0,
                        collisions: 0,
                    })
                    .build()
                    .map_err(|e| Error::system(format!("Failed to create interface: {}", e)))?;

                interface_map.insert(name.to_string(), interface);
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
                        rx_bytes, tx_bytes, rx_packets, tx_packets, rx_errors, tx_errors, collisions,
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
        let mut result = HashMap::new();
        let mut ifap: *mut ifaddrs = ptr::null_mut();

        unsafe {
            // Call getifaddrs() to get list of interfaces
            if getifaddrs(&mut ifap) != 0 {
                return Err(Error::network_error(
                    "get_interfaces",
                    "Failed to get network interfaces",
                ));
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
    /// This is the preferred method that directly accesses kernel network statistics rather than relying on
    /// command-line tools.
    fn update_traffic_stats_native(&self) -> Option<TrafficStatsMap> {
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
                        (
                            rx_bytes, tx_bytes, rx_packets, tx_packets, rx_errors, tx_errors, collisions,
                        ),
                    );
                }
            }
        }

        if result.is_empty() { None } else { Some(result) }
    }

    /// Updates traffic stats using netstat command-line tool.
    ///
    /// This is a fallback method that's safer but less efficient than using sysctlbyname. It parses the output of the
    /// netstat command to get interface statistics.
    fn update_traffic_stats_netstat(&self) -> Option<TrafficStatsMap> {
        // Using type alias defined at the top of the file This is a fallback method that's safer than sysctlbyname On
        // macOS, we can use netstat to get network statistics
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

            // Try to parse network stats Columns are: Name Mtu Network Address Ipkts Ierrs Ibytes Opkts Oerrs Obytes
            // Coll
            let rx_packets = parts[4].parse::<u64>().unwrap_or(0);
            let rx_errors = parts[5].parse::<u64>().unwrap_or(0);
            let rx_bytes = parts[6].parse::<u64>().unwrap_or(0);
            let tx_packets = parts[7].parse::<u64>().unwrap_or(0);
            let tx_errors = parts[8].parse::<u64>().unwrap_or(0);
            let tx_bytes = parts[9].parse::<u64>().unwrap_or(0);
            let collisions = if parts.len() > 10 {
                parts[10].parse::<u64>().unwrap_or(0)
            } else {
                0
            };

            result.insert(
                name,
                (
                    rx_bytes, tx_bytes, rx_packets, tx_packets, rx_errors, tx_errors, collisions,
                ),
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
        } else if name.starts_with("vnic") || name.starts_with("bridge") || name.starts_with("utun") {
            InterfaceType::Virtual
        } else {
            InterfaceType::Other
        }
    }
}

/// Monitor for tracking bytes received on a network interface
pub struct BytesReceivedMonitor {
    /// Reference to the network interface being monitored
    interface: Interface,
}

/// Monitor for tracking bytes sent on a network interface
pub struct BytesSentMonitor {
    /// Reference to the network interface being monitored
    interface: Interface,
}

/// Monitor for tracking packets received on a network interface
pub struct PacketsReceivedMonitor {
    /// Reference to the network interface being monitored
    interface: Interface,
}

/// Monitor for tracking packets sent on a network interface
pub struct PacketsSentMonitor {
    /// Reference to the network interface being monitored
    interface: Interface,
}

/// Mapping of interface names to their address properties
///
/// Format: HashMap<name, (flags, mac_address, ip_addresses)>
type NetworkAddressMap = HashMap<String, (u32, Option<String>, Vec<IpAddr>)>;

/// Mapping of interface names to their traffic statistics
///
/// Format: HashMap<name, (bytes_in, bytes_out, packets_in, packets_out, rx_errors, tx_errors, collisions)>
type TrafficStatsMap = HashMap<String, (u64, u64, u64, u64, u64, u64, u64)>;
