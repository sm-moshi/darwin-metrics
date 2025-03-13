use darwin_metrics::hardware::memory::{Memory, PressureLevel};
use std::error::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Memory Monitor Example");
    println!("=====================");

    // Create a new Memory instance
    let mut memory = Memory::new()?;

    // Register a callback for memory pressure changes
    memory.on_pressure_change(|level| {
        match level {
            PressureLevel::Normal => println!("\n🟢 MEMORY PRESSURE NORMAL"),
            PressureLevel::Warning => println!("\n🟠 MEMORY PRESSURE WARNING"),
            PressureLevel::Critical => println!("\n🔴 MEMORY PRESSURE CRITICAL"),
            // Handle future variants
            _ => println!("\n⚠️ MEMORY PRESSURE UNKNOWN STATE"),
        }
    });

    // Print column headers
    println!(
        "\n{:<10} | {:<12} | {:<12} | {:<12} | {:<7} | {:<12}",
        "Total", "Used", "Free", "Wired", "Usage", "Pressure"
    );
    println!("{:-<10} | {:-<12} | {:-<12} | {:-<12} | {:-<7} | {:-<12}", "", "", "", "", "", "");

    // Monitor memory for 10 seconds maximum (5 iterations x 2 seconds)
    for _ in 0..5 {
        // Update memory metrics
        memory.update()?;

        // Format all values in human-readable form
        let total_gb = memory.total as f64 / 1_073_741_824.0;
        let used_gb = memory.used as f64 / 1_073_741_824.0;
        let free_gb = memory.free as f64 / 1_073_741_824.0;
        let wired_gb = memory.page_states.wired as f64 / 1_073_741_824.0;
        let usage_pct = memory.usage_percentage();
        let pressure_pct = memory.pressure_percentage();

        println!(
            "{:<10.2} | {:<12.2} | {:<12.2} | {:<12.2} | {:<7.1}% | {:<12.1}%",
            total_gb, used_gb, free_gb, wired_gb, usage_pct, pressure_pct
        );

        // Wait 2 seconds before the next update
        thread::sleep(Duration::from_secs(2));
    }

    // Show memory breakdown by page states
    println!("\nMemory Breakdown by Page States (GB):");
    println!("{:<10} | {:<10} | {:<10} | {:<10} | {:<10}", "Active", "Inactive", "Wired", "Free", "Compressed");
    println!("{:-<10} | {:-<10} | {:-<10} | {:-<10} | {:-<10}", "", "", "", "", "");

    let active_gb = memory.page_states.active as f64 / 1_073_741_824.0;
    let inactive_gb = memory.page_states.inactive as f64 / 1_073_741_824.0;
    let wired_gb = memory.page_states.wired as f64 / 1_073_741_824.0;
    let free_gb = memory.page_states.free as f64 / 1_073_741_824.0;
    let compressed_gb = memory.page_states.compressed as f64 / 1_073_741_824.0;

    println!(
        "{:<10.2} | {:<10.2} | {:<10.2} | {:<10.2} | {:<10.2}",
        active_gb, inactive_gb, wired_gb, free_gb, compressed_gb
    );

    // Show swap usage
    println!("\nSwap Usage:");
    println!(
        "{:<10} | {:<10} | {:<10} | {:<10} | {:<10}",
        "Total (GB)", "Used (GB)", "Free (GB)", "In Rate", "Out Rate"
    );
    println!("{:-<10} | {:-<10} | {:-<10} | {:-<10} | {:-<10}", "", "", "", "", "");

    let swap_total_gb = memory.swap_usage.total as f64 / 1_073_741_824.0;
    let swap_used_gb = memory.swap_usage.used as f64 / 1_073_741_824.0;
    let swap_free_gb = memory.swap_usage.free as f64 / 1_073_741_824.0;

    println!(
        "{:<10.2} | {:<10.2} | {:<10.2} | {:<10.2}/s | {:<10.2}/s",
        swap_total_gb, swap_used_gb, swap_free_gb, memory.swap_usage.ins, memory.swap_usage.outs
    );

    println!("\nSwap Pressure: {:.1}%", memory.swap_usage.pressure * 100.0);

    Ok(())
}
