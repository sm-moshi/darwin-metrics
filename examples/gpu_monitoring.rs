use std::error::Error;
use std::time::Duration;

use darwin_metrics::gpu::Gpu;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let gpu = Gpu::new()?;

    println!("GPU Monitoring Example");
    println!("=====================");

    loop {
        // GPU Info
        println!("\nGPU Information:");
        println!("----------------");
        println!("GPU Name: {}", gpu.name().await?);
        
        // GPU Characteristics
        if let Ok(chars) = gpu.get_characteristics().await {
            println!("Vendor: {}", chars.vendor);
            println!("Is Integrated: {}", chars.is_integrated);
            println!("Is Apple Silicon: {}", chars.is_apple_silicon);
            if let Some(cores) = chars.core_count {
                println!("Core Count: {}", cores);
            }
        }
        
        // GPU Metrics
        println!("\nGPU Metrics:");
        println!("-----------");
        if let Ok(util) = gpu.get_utilization().await {
            println!("GPU Utilization: {:.1}%", util.value * 100.0);
        }
        
        // GPU Memory
        if let Ok(mem) = gpu.get_memory().await {
            println!("\nGPU Memory:");
            println!("-----------");
            println!("Total Memory: {} MB", mem.total / 1024 / 1024);
            println!("Used Memory: {} MB", mem.used / 1024 / 1024);
            println!("Free Memory: {} MB", mem.free / 1024 / 1024);
        }
        
        // GPU Temperature
        println!("\nGPU Temperature:");
        println!("---------------");
        if let Ok(temp) = gpu.get_temperature().await {
            println!("Temperature: {:.1}°C", temp);
        } else {
            println!("Temperature: Not available");
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
