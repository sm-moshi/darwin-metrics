use std::error::Error;
use std::time::Duration;

use darwin_metrics::{MemoryMonitor, MemoryUsageMonitor, System};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let system = System::new()?;

    println!("Memory Monitoring Example");
    println!("========================");

    loop {
        // Memory Info
        println!("\nMemory Information:");
        println!("------------------");
        println!("Total Memory: {} MB", system.total_memory().await? / 1024 / 1024);
        println!(
            "Available Memory: {} MB",
            system.available_memory().await? / 1024 / 1024
        );
        println!("Used Memory: {} MB", system.used_memory().await? / 1024 / 1024);
        println!("Free Memory: {} MB", system.free_memory().await? / 1024 / 1024);

        // Memory Load
        println!("\nMemory Load:");
        println!("------------");
        println!("Memory Usage: {:.1}%", system.memory_usage().await? * 100.0);
        println!("Memory Pressure: {:.1}%", system.memory_pressure().await? * 100.0);
        println!("Page Faults: {}", system.page_faults().await?);
        println!("Page Ins: {}", system.page_ins().await?);
        println!("Page Outs: {}", system.page_outs().await?);

        // Swap Memory
        println!("\nSwap Memory:");
        println!("------------");
        println!("Total Swap: {} MB", system.total_swap().await? / 1024 / 1024);
        println!("Used Swap: {} MB", system.used_swap().await? / 1024 / 1024);
        println!("Free Swap: {} MB", system.free_swap().await? / 1024 / 1024);
        println!("Swap Usage: {:.1}%", system.swap_usage().await? * 100.0);
        println!("Swap Ins: {}", system.swap_ins().await?);
        println!("Swap Outs: {}", system.swap_outs().await?);

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
