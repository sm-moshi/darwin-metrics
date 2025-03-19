use crate::error::Result;
use crate::network::{Interface, InterfaceType, NetworkManager, NetworkMetrics};
use crate::tests::common::builders::network::TestNetworkBuilder;

pub mod interface;
pub mod metrics;

#[test]
fn test_network_manager_creation() -> Result<()> {
    // This just tests that we can create a NetworkManager without errors
    let manager = TestNetworkBuilder::new().build_manager()?;
    assert!(!manager.interfaces().is_empty());
    Ok(())
}

#[test]
fn test_network_manager_interface_access() -> Result<()> {
    let manager = TestNetworkBuilder::new()
        .with_interface("test0", InterfaceType::Ethernet)
        .build_manager()?;

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

    Ok(())
}

#[test]
fn test_determine_interface_type() {
    use crate::utils::bindings::if_flags;

    // Test loopback detection
    assert_eq!(
        NetworkManager::determine_interface_type("lo0", if_flags::IFF_LOOPBACK),
        InterfaceType::Loopback
    );

    // Test ethernet detection
    assert_eq!(
        NetworkManager::determine_interface_type("en1", 0),
        InterfaceType::Ethernet
    );

    // Test WiFi detection (en0 on macOS)
    assert_eq!(NetworkManager::determine_interface_type("en0", 0), InterfaceType::WiFi);

    // Test WiFi detection (wl prefix)
    assert_eq!(
        NetworkManager::determine_interface_type("wlan0", 0),
        InterfaceType::WiFi
    );

    // Test virtual interface detection
    assert_eq!(
        NetworkManager::determine_interface_type("vnic0", 0),
        InterfaceType::Virtual
    );
    assert_eq!(
        NetworkManager::determine_interface_type("bridge0", 0),
        InterfaceType::Virtual
    );
    assert_eq!(
        NetworkManager::determine_interface_type("utun0", 0),
        InterfaceType::Virtual
    );

    // Test other interface
    assert_eq!(
        NetworkManager::determine_interface_type("unknown0", 0),
        InterfaceType::Other
    );
}
