use std::io;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use darwin_metrics::error::Result;
use darwin_metrics::network::interface::NetworkManager;
use darwin_metrics::traits::{CpuMonitor, HardwareMonitor, MemoryMonitor};

fn main() -> Result<()> {
    println!("Darwin Metrics - Network Monitor Example");
    println!("Press Ctrl+C to exit\n");

    // Initialize the NetworkManager
    let mut manager = NetworkManager::new()?;

    // Sample rate in milliseconds
    let sample_rate = Duration::from_millis(1000);
    let mut sample_count = 0;

    // Main monitoring loop
    loop {
        // Clear screen and move cursor to top-left for clean display
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush()?;

        println!("Sample #{}\n", sample_count);

        // Display network interfaces
        println!("════════════════════ NETWORK INTERFACES ════════════════════");
        for (i, interface) in manager.interfaces().iter().enumerate() {
            println!("Interface #{}: {}", i + 1, interface.name());
            println!("  Type: {}", interface.interface_type());

            if let Some(mac) = interface.mac_address() {
                println!("  MAC: {}", mac);
            }

            if let Some(addresses) = interface.addresses() {
                for (j, addr) in addresses.iter().enumerate() {
                    println!("  IP#{}: {}", j + 1, addr);
                }
            }

            println!();
        }

        // Display network statistics
        println!("═══════════════════ NETWORK STATISTICS ═══════════════════");
        for interface in manager.interfaces() {
            println!("Interface: {}", interface.name());

            // Calculate speeds directly using the interface methods
            let download_speed = interface.packet_receive_rate();
            let upload_speed = interface.packet_send_rate();

            // Download/upload speeds
            println!("  Download speed: {:.1} packets/s", download_speed);
            println!("  Upload speed: {:.1} packets/s", upload_speed);

            // Error rates
            let recv_err_rate = interface.receive_error_rate();
            let send_err_rate = interface.send_error_rate();

            if recv_err_rate > 0.0 || send_err_rate > 0.0 {
                println!("  Error rates: {:.6} receive, {:.6} send", recv_err_rate, send_err_rate);
            }

            // Create a simple ASCII graph for network activity
            let graph_width = 50;
            let max_packets = 1000.0; // 1000 packets/s for scaling

            let recv_percent = if max_packets > 0.0 {
                (download_speed / max_packets) * 100.0
            } else {
                0.0
            };

            let sent_percent = if max_packets > 0.0 {
                (upload_speed / max_packets) * 100.0
            } else {
                0.0
            };

            println!(
                "  Packet receive: {}",
                create_ascii_graph(recv_percent.min(100.0), graph_width)
            );
            println!(
                "  Packet send:    {}",
                create_ascii_graph(sent_percent.min(100.0), graph_width)
            );
            println!();
        }

        println!("Press Ctrl+C to exit");

        // Update statistics for next iteration
        manager.update()?;

        sample_count += 1;
        sleep(sample_rate);
    }
}

// Helper function to convert bytes to human-readable format
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

// Helper function to create ASCII graph
fn create_ascii_graph(value: f64, max: usize) -> String {
    let filled_chars = (value as usize * max) / 100;
    let empty_chars = max - filled_chars;

    let mut graph = String::from("[");
    graph.push_str(&"#".repeat(filled_chars));
    graph.push_str(&" ".repeat(empty_chars));
    graph.push_str(&format!(" ] {:.1}%", value));
    graph
}
