use super::*;
use crate::network::interface::{Interface, InterfaceType, NetworkManager};
use crate::network::if_flags;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

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
            let (
                rx_bytes,
                tx_bytes,
                rx_packets,
                tx_packets,
                _rx_errors,
                _tx_errors,
                _collisions,
            ) = *lo0_stats.unwrap();

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

#[test]
fn test_network_speed_calculation() {
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

    // Initial speeds should be 0 since we need two measurements
    assert_eq!(interface.download_speed(), 0.0);
    assert_eq!(interface.upload_speed(), 0.0);

    // Update traffic with new values
    interface.update_traffic(
        2000, // bytes_received (+1000)
        4000, // bytes_sent (+2000)
        20,   // packets_received (+10)
        40,   // packets_sent (+20)
        2,    // receive_errors (+1)
        4,    // send_errors (+2)
        1,    // collisions (+1)
    );

    // Now speeds should be calculated based on the time difference
    assert!(interface.download_speed() > 0.0);
    assert!(interface.upload_speed() > 0.0);
}

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

// If this test runs on a real machine, it might still fail because we can't guarantee the system has network
// interfaces. We'll skip this test in CI environments.
#[test]
#[ignore = "This test needs a real network environment"]
fn test_real_network_manager() {
    let network_manager = NetworkManager::new();

    // If we're on a real system with network interfaces, this should succeed
    if let Ok(manager) = network_manager {
        assert!(!manager.interfaces().is_empty(), "Should have at least one interface");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_interface_builder_basic() {
        let interface = Interface::builder()
            .name("en0")
            .interface_type(InterfaceType::Ethernet)
            .flags(0)
            .build()
            .expect("Failed to build interface");

        assert_eq!(interface.name(), "en0");
        assert_eq!(interface.interface_type(), InterfaceType::Ethernet);
        assert_eq!(interface.flags(), 0);
        assert!(interface.mac_address().is_none());
        assert!(interface.addresses().is_empty());
    }

    #[test]
    fn test_interface_builder_complete() {
        let ipv4 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ipv6 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        
        let interface = Interface::builder()
            .name("en0")
            .interface_type(InterfaceType::Ethernet)
            .flags(1)
            .mac_address("00:11:22:33:44:55")
            .add_address(ipv4)
            .add_address(ipv6)
            .traffic_stats(100, 200, 10, 20, 1, 2, 0)
            .build()
            .expect("Failed to build interface");

        assert_eq!(interface.name(), "en0");
        assert_eq!(interface.interface_type(), InterfaceType::Ethernet);
        assert_eq!(interface.flags(), 1);
        assert_eq!(interface.mac_address(), Some("00:11:22:33:44:55".to_string()));
        assert_eq!(interface.addresses().len(), 2);
        assert!(interface.addresses().contains(&ipv4));
        assert!(interface.addresses().contains(&ipv6));
        assert_eq!(interface.bytes_received(), 100);
        assert_eq!(interface.bytes_sent(), 200);
        assert_eq!(interface.packets_received(), 10);
        assert_eq!(interface.packets_sent(), 20);
        assert_eq!(interface.receive_errors(), 1);
        assert_eq!(interface.send_errors(), 2);
        assert_eq!(interface.collisions(), 0);
    }

    #[test]
    fn test_interface_builder_missing_required() {
        // Test missing name
        let result = Interface::builder()
            .interface_type(InterfaceType::Ethernet)
            .build();
        assert!(result.is_err());

        // Test missing interface type
        let result = Interface::builder()
            .name("en0")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_interface_builder_addresses() {
        let addresses = vec![
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
        ];

        // Test adding addresses individually
        let interface1 = Interface::builder()
            .name("en0")
            .interface_type(InterfaceType::Ethernet)
            .add_address(addresses[0])
            .add_address(addresses[1])
            .build()
            .expect("Failed to build interface");

        // Test setting addresses as vec
        let interface2 = Interface::builder()
            .name("en0")
            .interface_type(InterfaceType::Ethernet)
            .addresses(addresses.clone())
            .build()
            .expect("Failed to build interface");

        assert_eq!(interface1.addresses(), interface2.addresses());
        assert_eq!(interface1.addresses().len(), 2);
    }
}
