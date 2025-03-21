use std::error::Error;
use std::time::Duration;

use darwin_metrics::traits::MemoryMonitor;
use darwin_metrics::memory::MemoryMonitor as ExtendedMemoryMonitor;
use darwin_metrics::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let memory = Memory::new()?;

    println!("Memory Monitoring Example");
    println!("========================");

    loop {
        // Memory Info
        println!("\nMemory Information:");
        println!("------------------");
        println!("Total Memory: {} MB", memory.total().await? / 1024 / 1024);
        println!(
            "Available Memory: {} MB",
            memory.available().await? / 1024 / 1024
        );
        println!("Used Memory: {} MB", memory.used().await? / 1024 / 1024);
        println!("Free Memory: {} MB", memory.available().await? / 1024 / 1024);

        // Memory Load
        println!("\nMemory Load:");
        println!("------------");
        println!("Memory Usage: {:.1}%", memory.usage_percentage().await? * 100.0);
        
        // Memory pressure and page stats using the extended MemoryMonitor trait
        println!("Memory Pressure: {:.1}%", memory.pressure_percentage().await? * 100.0);
        
        let page_states = memory.page_states().await?;
        println!("Active Pages: {} MB", page_states.active * 4096 / 1024 / 1024);
        println!("Inactive Pages: {} MB", page_states.inactive * 4096 / 1024 / 1024);
        println!("Wired Pages: {} MB", page_states.wired * 4096 / 1024 / 1024);
        println!("Free Pages: {} MB", page_states.free * 4096 / 1024 / 1024);
        println!("Compressed Pages: {} MB", page_states.compressed * 4096 / 1024 / 1024);

        // Swap Memory
        println!("\nSwap Memory:");
        println!("------------");
        let swap_usage = memory.swap_usage().await?;
        
        println!("Total Swap: {} MB", swap_usage.total / 1024 / 1024);
        println!("Used Swap: {} MB", swap_usage.used / 1024 / 1024);
        println!("Free Swap: {} MB", swap_usage.free / 1024 / 1024);
        println!("Swap In Rate: {:.2} pages/sec", swap_usage.ins);
        println!("Swap Out Rate: {:.2} pages/sec", swap_usage.outs);
        println!("Swap Pressure: {:.1}%", swap_usage.pressure * 100.0);

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
