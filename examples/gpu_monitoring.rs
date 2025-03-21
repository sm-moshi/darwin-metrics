use std::error::Error;
use std::time::Duration;

use darwin_metrics::{GpuMemoryMonitor, GpuMonitor, GpuTemperatureMonitor, System};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let system = System::new()?;

    println!("GPU Monitoring Example");
    println!("=====================");

    loop {
        // GPU Info
        println!("\nGPU Information:");
        println!("--------------");
        println!("GPU Model: {}", system.gpu_model().await?);
        println!("GPU Vendor: {}", system.gpu_vendor().await?);
        println!("GPU Architecture: {}", system.gpu_architecture().await?);
        println!("GPU Driver Version: {}", system.gpu_driver_version().await?);

        // GPU Load
        println!("\nGPU Load:");
        println!("---------");
        println!("GPU Utilization: {:.1}%", system.gpu_utilization().await? * 100.0);
        println!("GPU Core Load: {:.1}%", system.gpu_core_load().await? * 100.0);
        println!(
            "GPU Memory Controller Load: {:.1}%",
            system.gpu_memory_controller_load().await? * 100.0
        );
        println!(
            "GPU Video Engine Load: {:.1}%",
            system.gpu_video_engine_load().await? * 100.0
        );

        // GPU Memory
        println!("\nGPU Memory:");
        println!("-----------");
        println!("Total Memory: {} MB", system.gpu_total_memory().await? / 1024 / 1024);
        println!("Used Memory: {} MB", system.gpu_used_memory().await? / 1024 / 1024);
        println!("Free Memory: {} MB", system.gpu_free_memory().await? / 1024 / 1024);
        println!(
            "Memory Utilization: {:.1}%",
            system.gpu_memory_utilization().await? * 100.0
        );

        // GPU Temperature
        println!("\nGPU Temperature:");
        println!("---------------");
        if let Ok(temp) = system.gpu_temperature().await {
            println!("GPU Temperature: {:.1}°C", temp);
        } else {
            println!("GPU Temperature: Not available");
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
