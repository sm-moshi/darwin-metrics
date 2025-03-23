// Remove the file-level cfg attribute
// #![cfg(feature = "mock")]

// Conditionally include these imports only when mock feature is enabled
#[cfg(feature = "mock")]
use darwin_metrics::resource::ResourceMonitor;
#[cfg(feature = "mock")]
use darwin_metrics::utils::tests::test_utils::iokit_mock::MockIOKit;

// The real implementation only when mock feature is enabled
#[cfg(feature = "mock")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only available when the 'testing' feature is enabled
    println!("Setting up mock data...");

    // Create a mock IOKit and configure it with test data
    let mut mock_iokit = MockIOKit::new()?;

    // Set CPU temperature
    mock_iokit.set_temperature(68.5);

    // Set physical and logical cores
    mock_iokit.set_physical_cores(8);
    mock_iokit.set_logical_cores(16);

    // Set core usage
    mock_iokit.set_core_usage(vec![0.25, 0.30, 0.15, 0.40, 0.10, 0.20, 0.35, 0.22])?;

    // Mock thermal information will be automatically set based on the temperature
    println!("Mock data configured");

    // Create a resource monitor that updates every second
    println!("Creating resource monitor...");
    let mut monitor = ResourceMonitor::new(Duration::from_secs(1)).await?;

    println!("Monitoring system resources with mock data (limited to 5 updates)...");

    // Main loop to get updates (limited to 5 updates for testing)
    let mut update_count = 0;
    const MAX_UPDATES: usize = 5;

    while update_count < MAX_UPDATES {
        match monitor.next_update().await {
            Ok(update) => {
                update_count += 1;
                println!("\n--- System Resources Update #{} ---", update_count);

                // Memory information - safely handle zero values
                let memory_used_pct = if update.memory.total > 0 {
                    (update.memory.used as f64 / update.memory.total as f64) * 100.0
                } else {
                    0.0
                };
                println!(
                    "Memory: {} used / {} total ({:.1}%)",
                    format_bytes(update.memory.used as f64),
                    format_bytes(update.memory.total as f64),
                    memory_used_pct
                );

                // CPU temperature (when using mock data this should show our value)
                println!("CPU Temperature: {:.1}°C", update.temperature.0);

                // Safely print disk information
                if update.disk.total > 0 {
                    let disk_used_pct = (update.disk.used as f64 / update.disk.total as f64) * 100.0;
                    println!(
                        "Disk Space: {} used / {} total ({:.1}%)",
                        format_bytes(update.disk.used as f64),
                        format_bytes(update.disk.total as f64),
                        disk_used_pct
                    );
                } else {
                    println!("Disk Space: Information not available");
                }

                // Network information - show interfaces and their IP addresses
                println!("Network Interfaces:");
                if update.network.ip_addresses.is_empty() {
                    println!("  No IP addresses available");
                } else {
                    println!(
                        "  Interface: {} ({})",
                        update.network.name,
                        if update.network.is_active { "Active" } else { "Inactive" }
                    );

                    if let Some(mac) = &update.network.mac_address {
                        println!("  MAC Address: {}", mac);
                    }

                    println!("  IP Addresses:");
                    for ip in &update.network.ip_addresses {
                        println!("    - {}", ip);
                    }
                }

                println!("Timestamp: {:?}", update.timestamp);

                // Add a slight delay before the next loop iteration
                tokio::time::sleep(Duration::from_millis(500)).await;
            },
            Err(e) => {
                eprintln!("Error getting update: {}", e);
                // Wait a bit before retrying
                tokio::time::sleep(Duration::from_secs(2)).await;
            },
        }
    }

    // Stop the monitor
    println!("\nStopping resource monitor...");
    if let Err(e) = monitor.stop().await {
        eprintln!("Error stopping monitor: {}", e);
    } else {
        println!("Resource monitor stopped successfully");
    }

    Ok(())
}

// A placeholder main function when mock feature is not enabled
#[cfg(not(feature = "mock"))]
fn main() {
    println!("This example requires the 'mock' feature to be enabled.");
    println!("Run with: cargo run --example mock_resource_monitor --features=\"mock\"");
}

// Helper function (conditionally compile based on feature as it's used by the real implementation)
#[cfg(feature = "mock")]
fn format_bytes(bytes: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bytes >= GB {
        format!("{:.2} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes / KB)
    } else {
        format!("{:.0} bytes", bytes)
    }
}
