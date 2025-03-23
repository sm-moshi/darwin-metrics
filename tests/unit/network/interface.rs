use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::error::Result;
use crate::network::{Interface, InterfaceType, NetworkMetrics};
use crate::utils::bindings::if_flags;

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
    assert_eq!(
        interface.addresses().unwrap()[0],
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))
    );

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
fn test_interface_builder_basic() {
    let interface = Interface::builder()
        .name("en0")
        .interface_type(InterfaceType::Ethernet)
        .flags(0)
        .build()
        .expect("Failed to build interface");

    assert_eq!(interface.name(), "en0");
    assert_eq!(interface.interface_type(), &InterfaceType::Ethernet);
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
        .traffic_stats(crate::network::interface::TrafficStatsParams {
            bytes_received: 100,
            bytes_sent: 200,
            packets_received: 10,
            packets_sent: 20,
            receive_errors: 1,
            send_errors: 2,
            collisions: 0,
        })
        .build()
        .expect("Failed to build interface");

    assert_eq!(interface.name(), "en0");
    assert_eq!(interface.interface_type(), &InterfaceType::Ethernet);
    assert_eq!(interface.flags(), 1);
    assert_eq!(interface.mac_address(), Some("00:11:22:33:44:55"));
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
    let result = Interface::builder().interface_type(InterfaceType::Ethernet).build();
    assert!(result.is_err());

    // Test missing interface type
    let result = Interface::builder().name("en0").build();
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
