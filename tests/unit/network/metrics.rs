use std::net::IpAddr;

use crate::error::Result;
use crate::network::{Interface, InterfaceType, NetworkMetrics};
use crate::utils::bindings::if_flags;

// Create a mock network interface for testing
fn create_mock_interface(name: &str, interface_type: InterfaceType, is_loopback: bool) -> Interface {
    let flags = if is_loopback {
        if_flags::IFF_UP | if_flags::IFF_RUNNING | if_flags::IFF_LOOPBACK
    } else {
        if_flags::IFF_UP | if_flags::IFF_RUNNING
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
