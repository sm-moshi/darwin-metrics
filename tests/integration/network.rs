use darwin_metrics::error::Result;
use darwin_metrics::network::{NetworkManager, NetworkMetrics};

#[test]
fn test_network_manager_integration() -> Result<()> {
    let network_manager = NetworkManager::new()?;

    // Verify we can get interfaces
    let interfaces = network_manager.interfaces();

    // We should have at least one interface (loopback)
    assert!(!interfaces.is_empty(), "Should have at least one network interface");

    // Check if we have a loopback interface
    let loopback = network_manager.get_interface("lo0");
    if let Some(lo) = loopback {
        assert!(lo.is_loopback());

        // Basic metrics should be available
        assert!(lo.bytes_received() >= 0);
        assert!(lo.bytes_sent() >= 0);
        assert!(lo.packets_received() >= 0);
        assert!(lo.packets_sent() >= 0);
    }

    // Test total speeds
    let download_speed = network_manager.total_download_speed();
    let upload_speed = network_manager.total_upload_speed();

    // Speeds might be 0 on first measurement
    assert!(download_speed >= 0.0);
    assert!(upload_speed >= 0.0);

    Ok(())
}

// This test is marked as ignored because it requires a real network environment
// and might fail in CI environments without network interfaces
#[test]
#[ignore = "This test needs a real network environment"]
fn test_real_network_traffic() -> Result<()> {
    use std::thread::sleep;
    use std::time::Duration;

    let mut network_manager = NetworkManager::new()?;

    // First measurement
    let _ = network_manager.update();

    // Wait a bit to generate some traffic
    sleep(Duration::from_secs(1));

    // Second measurement
    let _ = network_manager.update();

    // Now we should have some speed measurements
    for interface in network_manager.interfaces() {
        println!("Interface: {}", interface.name());
        println!("  Download speed: {} bytes/sec", interface.download_speed());
        println!("  Upload speed: {} bytes/sec", interface.upload_speed());
    }

    Ok(())
}
