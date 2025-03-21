use std::error::Error;
use std::time::Duration;

use darwin_metrics::memory::MemoryMonitor as MemoryMonitorTrait;
use darwin_metrics::traits::MemoryMonitor;
use darwin_metrics::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let memory = Memory::new()?;

    println!("Memory Monitor Example");
    println!("=====================");

    loop {
        // Get memory info
        let info = memory.memory_info().await?;

        // Calculate usage percentage
        let usage_pct = memory.usage_percentage().await?;

        // Calculate pressure percentage
        let pressure_pct = memory.pressure_percentage().await?;

        println!("\nMemory Statistics:");
        println!("Total: {} MB", info.total / 1024 / 1024);
        println!("Used: {} MB", info.used / 1024 / 1024);
        println!("Free: {} MB", info.free / 1024 / 1024);
        println!("Usage: {:.1}%", usage_pct);
        println!("Pressure: {:.1}%", pressure_pct);

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
