# Memory Analysis

The memory module provides detailed information about system memory usage, memory pressure, and swap activity. It offers both synchronous and asynchronous APIs for monitoring memory metrics with a focus on reliability and performance.

## Features

- **System Memory Metrics**: Total, available, and used memory with wired memory tracking
- **Page State Tracking**: Active, inactive, wired, free, and compressed memory page states
- **Memory Pressure Monitoring**: Real-time memory pressure monitoring with configurable thresholds
- **Pressure Level Callbacks**: Register callbacks to be notified when memory pressure levels change
- **Swap Usage Monitoring**: Track swap usage, activity rates, and pressure
- **Asynchronous Support**: Full async API for non-blocking memory metrics collection
- **Memory History**: Built-in tracking of memory usage history
- **Resilient Implementation**: Graceful fallbacks for environments where certain metrics aren't available

## Usage Examples

### Basic Memory Information

```rust
use darwin_metrics::hardware::memory::Memory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new Memory instance
    let memory = Memory::new()?;
    
    // Get basic memory metrics
    println!("Total Memory: {:.2} GB", memory.total as f64 / 1_073_741_824.0);
    println!("Used Memory: {:.2} GB", memory.used as f64 / 1_073_741_824.0);
    println!("Available Memory: {:.2} GB", memory.available as f64 / 1_073_741_824.0);
    println!("Memory Usage: {:.1}%", memory.usage_percentage());
    
    // Check memory pressure
    println!("Memory Pressure: {:.1}%", memory.pressure_percentage());
    println!("Memory Pressure Level: {:?}", memory.pressure_level());
    
    Ok(())
}
```

### Monitoring Memory Changes

```rust
use darwin_metrics::hardware::memory::Memory;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new Memory instance
    let mut memory = Memory::new()?;
    
    // Monitor memory for a period of time
    for _ in 0..5 {
        // Update memory metrics
        memory.update()?;
        
        println!("Memory Usage: {:.1}%", memory.usage_percentage());
        println!("Memory Pressure: {:.1}%", memory.pressure_percentage());
        
        // Wait before the next update
        thread::sleep(Duration::from_secs(2));
    }
    
    Ok(())
}
```

### Memory Pressure Callbacks

```rust
use darwin_metrics::hardware::memory::{Memory, PressureLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new Memory instance
    let mut memory = Memory::new()?;
    
    // Register a callback for memory pressure changes
    memory.on_pressure_change(|level| {
        match level {
            PressureLevel::Normal => println!("Memory pressure is NORMAL"),
            PressureLevel::Warning => println!("Memory pressure is HIGH - consider closing applications"),
            PressureLevel::Critical => println!("Memory pressure is CRITICAL - system may be unstable"),
        }
    });
    
    // Set custom pressure thresholds (warning at 50%, critical at 80%)
    memory.set_pressure_thresholds(0.5, 0.8)?;
    
    // Update and trigger checks
    memory.update()?;
    
    Ok(())
}
```

### Asynchronous Memory Monitoring

```rust
use darwin_metrics::hardware::memory::Memory;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new Memory instance
    let mut memory = Memory::new()?;
    
    // Start monitoring memory pressure asynchronously
    let monitor = memory.start_monitoring(500).await?;
    println!("Started memory pressure monitoring...");
    
    // Update asynchronously
    for _ in 0..5 {
        memory.update_async().await?;
        println!("Memory Usage: {:.1}%", memory.usage_percentage());
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    
    // Stop monitoring when done
    monitor.stop();
    
    Ok(())
}
```

## Page States and Memory Classifications

The `Memory` struct provides detailed information about different memory page states:

- **Active**: Memory currently in active use
- **Inactive**: Memory that hasn't been accessed recently
- **Wired**: Memory that can't be paged out (kernel, etc.)
- **Free**: Memory immediately available for allocation
- **Compressed**: Memory that has been compressed to save physical RAM

These metrics help you understand not just how much memory is in use, but how it's being used and managed by the system.

## Swap Usage and Memory Pressure

In macOS, swap usage and memory pressure are important metrics for understanding system performance:

- **Swap Usage**: Tracks total, used, and free swap space
- **Swap Activity**: Monitors swap-in and swap-out rates
- **Swap Pressure**: Shows percentage of swap utilization
- **Memory Pressure**: Overall system memory pressure indicator

High memory pressure often precedes increased swap activity and can indicate potential performance issues.

## API Reference

For complete API details, see the [Rust API documentation](https://docs.rs/darwin-metrics/latest/darwin_metrics/hardware/memory/index.html).
