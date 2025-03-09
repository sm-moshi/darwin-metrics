use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ptr;

use crate::error::{Error, Result};
use crate::network::traffic::TrafficTracker;
use crate::network::NetworkMetrics;
use crate::utils::bindings::{
    address_family, freeifaddrs, getifaddrs, if_data, if_flags, ifaddrs, sockaddr, sockaddr_dl,
    sockaddr_in, sockaddr_in6, sysctlbyname,
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

        manager.update()?;
        Ok(manager)
    }

    /// Updates all network interfaces and their metrics.
    pub fn update(&mut self) -> Result<()> {
        // Get new interface data
        let interfaces = Self::get_interfaces()?;

        // Update existing interfaces or add new ones
        for interface in interfaces {
            let name = interface.name().to_string();

            if let Some(existing) = self.interfaces.get_mut(&name) {
                // Update existing interface
                existing.update_traffic(
                    interface.bytes_received(),
                    interface.bytes_sent(),
                    interface.packets_received(),
                    interface.packets_sent(),
                    interface.receive_errors(),
                    interface.send_errors(),
                    interface.collisions(),
                );
            } else {
                // Add new interface
                self.interfaces.insert(name, interface);
            }
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

    /// Gets interface metrics using the getifaddrs() system call.
    fn get_interfaces() -> Result<Vec<Interface>> {
        let mut result = Vec::new();
        let mut ifap: *mut ifaddrs = ptr::null_mut();

        // Call getifaddrs to get the list of interfaces
        unsafe {
            if getifaddrs(&mut ifap) != 0 {
                return Err(Error::Network(
                    "Failed to get network interfaces".to_string(),
                ));
            }

            // Safety: ifap was properly initialized by getifaddrs
            let _ = scopeguard::guard(ifap, |ifap| {
                freeifaddrs(ifap);
            });

            // Track interfaces we've seen to avoid duplicates
            let mut seen_interfaces = HashMap::new();

            // Iterate through the linked list of interfaces
            let mut current = ifap;
            while !current.is_null() {
                let ifa = &*current;
                let name = if ifa.ifa_name.is_null() {
                    continue;
                } else {
                    let c_name = CStr::from_ptr(ifa.ifa_name);
                    c_name.to_string_lossy().to_string()
                };

                // Skip invalid interfaces
                if name.is_empty() {
                    current = ifa.ifa_next;
                    continue;
                }

                // Process this interface
                Self::process_interface(&mut seen_interfaces, ifa, &name)?;

                // Move to the next interface
                current = ifa.ifa_next;
            }

            // Convert the HashMap to a Vec
            for (_, interface) in seen_interfaces {
                result.push(interface);
            }
        }

        Ok(result)
    }

    /// Processes a single interface from the getifaddrs list.
    unsafe fn process_interface(
        seen_interfaces: &mut HashMap<String, Interface>,
        ifa: &ifaddrs,
        name: &str,
    ) -> Result<()> {
        // Get or create interface entry
        let interface = seen_interfaces.entry(name.to_string()).or_insert_with(|| {
            let interface_type = Self::determine_interface_type(name, ifa.ifa_flags);

            // Initialize with zeros, we'll get real data later
            Interface::new(
                name.to_string(),
                interface_type,
                ifa.ifa_flags,
                None,
                Vec::new(),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            )
        });

        // Process address information
        if !ifa.ifa_addr.is_null() {
            Self::process_address(interface, ifa.ifa_addr);
        }

        // Get interface statistics using sysctl
        Self::update_interface_stats(interface)?;

        Ok(())
    }

    /// Determines the type of interface based on its name and flags.
    fn determine_interface_type(name: &str, flags: u32) -> InterfaceType {
        if (flags & if_flags::IFF_LOOPBACK) != 0 {
            InterfaceType::Loopback
        } else if name.starts_with("en") {
            // On macOS, en0 is usually Ethernet, but could be WiFi
            // A more accurate method would be to check IORegistry
            if name == "en0" || name == "en1" {
                // Simplified logic; in reality we should check if it's WiFi or Ethernet
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

    /// Processes an address from the interface address structure.
    unsafe fn process_address(interface: &mut Interface, addr_ptr: *mut sockaddr) {
        let addr = &*addr_ptr;

        match addr.sa_family {
            family if family == address_family::AF_INET => {
                // IPv4 address
                let addr_in = &*(addr_ptr as *mut sockaddr_in);
                let ip = Ipv4Addr::from(u32::from_be(addr_in.sin_addr.s_addr));
                interface.addresses.push(IpAddr::V4(ip));
            }
            family if family == address_family::AF_INET6 => {
                // IPv6 address
                let addr_in6 = &*(addr_ptr as *mut sockaddr_in6);
                let ip = Ipv6Addr::from(addr_in6.sin6_addr.s6_addr);
                interface.addresses.push(IpAddr::V6(ip));
            }
            family if family == address_family::AF_LINK => {
                // Link-layer (MAC) address
                let addr_dl = &*(addr_ptr as *mut sockaddr_dl);

                let mac_len = addr_dl.sdl_alen as usize;
                if mac_len == 6 {
                    // Typical MAC address length
                    let offset = addr_dl.sdl_nlen as usize;
                    let mac_bytes = &addr_dl.sdl_data[offset..offset + mac_len];

                    // Format as MAC address "xx:xx:xx:xx:xx:xx"
                    let mac = format!(
                        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                        mac_bytes[0] as u8,
                        mac_bytes[1] as u8,
                        mac_bytes[2] as u8,
                        mac_bytes[3] as u8,
                        mac_bytes[4] as u8,
                        mac_bytes[5] as u8
                    );

                    interface.mac_address = Some(mac);
                }
            }
            _ => {} // Ignore other address families
        }
    }

    /// Updates interface statistics using sysctl.
    unsafe fn update_interface_stats(interface: &mut Interface) -> Result<()> {
        // Create the sysctl name for this interface
        let sysctl_name =
            CString::new(format!("net.link.generic.system.ifdata.{}", interface.name)).map_err(
                |_| Error::Network(format!("Invalid interface name: {}", interface.name)),
            )?;

        // Prepare variables for the sysctl call
        let mut if_data_val: if_data = std::mem::zeroed();
        let mut len = std::mem::size_of::<if_data>();

        // Call sysctl to get the interface data
        if sysctlbyname(
            sysctl_name.as_ptr(),
            &mut if_data_val as *mut _ as *mut c_void,
            &mut len,
            ptr::null(),
            0,
        ) != 0
        {
            return Err(Error::Network(format!(
                "Failed to get statistics for interface {}",
                interface.name
            )));
        }

        // Update the interface with the retrieved statistics
        interface.update_traffic(
            if_data_val.ifi_ibytes as u64,
            if_data_val.ifi_obytes as u64,
            if_data_val.ifi_ipackets as u64,
            if_data_val.ifi_opackets as u64,
            if_data_val.ifi_ierrors as u64,
            if_data_val.ifi_oerrors as u64,
            if_data_val.ifi_collisions as u64,
        );

        Ok(())
    }
}
