# CPU Monitoring

The CPU module in darwin-metrics provides a comprehensive API for monitoring CPU metrics on macOS systems. This module allows you to track CPU usage, temperature, frequency, and other important processor metrics in real-time.

## Overview

The CPU monitoring functionality is centered around the `CPU` struct, which provides access to a variety of processor metrics through a clean, ergonomic API. The module also includes the `CpuMetrics` trait, which defines a standard interface for retrieving common CPU statistics.

## Features

- **Core Statistics**: Physical and logical core counts
- **Usage Monitoring**: Overall and per-core CPU utilization tracking
- **Temperature Sensing**: CPU temperature readings (when available)
- **Frequency Analysis**: Current, minimum, and maximum CPU frequencies
- **Model Information**: CPU model identification

## Quick Start

Here's a simple example to get you started with CPU monitoring:

```rust,no_run,ignore
// Basic CPU monitoring example
use darwin_metrics::hardware::cpu::CPU;
use darwin_metrics::error::Result;

fn main() -> Result<()> {
    // Initialize the CPU monitor
    let cpu = CPU::new()?;

    // Display basic CPU information
    println!("CPU Model: {}", cpu.model_name());
    println!("Physical Cores: {}", cpu.physical_cores());
    println!("Logical Cores: {}", cpu.logical_cores());

    // Show current CPU metrics
    println!("CPU Usage: {:.1}%", cpu.get_cpu_usage() * 100.0);
    println!("CPU Frequency: {:.0} MHz", cpu.frequency_mhz());

    if let Some(temp) = cpu.temperature() {
        println!("CPU Temperature: {:.1}°C", temp);
    }

    // Display per-core usage
    for (i, usage) in cpu.core_usage().iter().enumerate() {
        println!("Core {} Usage: {:.1}%", i, usage * 100.0);
    }

    Ok(())
}
```

## The `CPU` Struct

The `CPU` struct is the primary interface for CPU monitoring. It provides methods to retrieve various CPU metrics:

### Initialization

```rust,no_run,ignore
// Create a new CPU instance with current metrics
let cpu = CPU::new()?;
```

### Basic Properties

```rust,no_run,ignore
// Get the CPU model name
let model = cpu.model_name(); // e.g., "Apple M1 Pro"

// Get core counts
let physical = cpu.physical_cores(); // e.g., 8
let logical = cpu.logical_cores();   // e.g., 10
```

### Performance Metrics

```rust,no_run,ignore
// Get the current CPU frequency in MHz
let freq = cpu.frequency_mhz(); // e.g., 3200.0

// Get overall CPU usage (0.0 to 1.0)
let usage = cpu.get_cpu_usage(); // e.g., 0.35 (35%)

// Get per-core usage as a slice
let core_usage = cpu.core_usage(); // e.g., [0.4, 0.2, 0.6, 0.1, ...]

// Get CPU temperature (if available)
if let Some(temp) = cpu.temperature() {
    println!("CPU is running at {:.1}°C", temp);
}
```

### Updating Metrics

To get the latest CPU metrics, you can update the instance:

```rust,no_run,ignore
// Refresh all CPU metrics
cpu.update()?;
```

## Frequency Monitoring

For detailed CPU frequency information, darwin-metrics provides comprehensive access through both the `CPU` struct directly and the standalone `FrequencyMonitor`:

### Using the CPU Struct (Recommended)

```rust,no_run,ignore
use darwin_metrics::hardware::cpu::CPU;
use darwin_metrics::error::Result;

fn monitor_cpu_frequency() -> Result<()> {
    let cpu = CPU::new()?;

    // Get current frequency
    println!("Current frequency: {:.0} MHz", cpu.frequency_mhz());

    // Get detailed frequency metrics (min/max/steps)
    if let Some(min) = cpu.min_frequency_mhz() {
        println!("Min frequency: {:.0} MHz", min);
    }

    if let Some(max) = cpu.max_frequency_mhz() {
        println!("Max frequency: {:.0} MHz", max);
    }

    // Get all available frequency steps
    if let Some(steps) = cpu.available_frequencies() {
        println!("Available frequency steps: {:?} MHz", steps);
    }

    // Access the complete frequency metrics object
    if let Some(metrics) = cpu.frequency_metrics() {
        println!("Current: {:.0} MHz", metrics.current);
        println!("Min: {:.0} MHz", metrics.min);
        println!("Max: {:.0} MHz", metrics.max);
        println!("Available steps: {:?} MHz", metrics.available);
    }

    Ok(())
}
```

### Using the FrequencyMonitor Directly

```rust,no_run,ignore
use darwin_metrics::hardware::cpu::FrequencyMonitor;
use darwin_metrics::error::Result;

fn monitor_frequency() -> Result<()> {
    let monitor = FrequencyMonitor::new();
    let metrics = monitor.get_metrics()?;

    println!("Current: {:.0} MHz", metrics.current);
    println!("Min: {:.0} MHz", metrics.min);
    println!("Max: {:.0} MHz", metrics.max);
    println!("Available steps: {:?} MHz", metrics.available);

    Ok(())
}
```

## Platform-Specific Notes

### macOS Implementation Details

On macOS, the CPU module uses a combination of:

- **IOKit Framework**: For accessing low-level hardware information
- **AppleACPICPU Service**: For core count and CPU model information
- **System Management Controller (SMC)**: For temperature readings
- **sysctl**: For retrieving accurate CPU frequency information
  - `hw.cpufrequency` for current frequency
  - `hw.cpufrequency_min` for minimum frequency
  - `hw.cpufrequency_max` for maximum frequency

### Temperature Monitoring

Temperature readings are available on most modern Mac hardware, but may be unavailable on:

- Older Mac models
- Virtual machines
- Some Mac models with specific security restrictions

When temperature readings are not available, the `temperature()` method returns `None`.

## Performance Considerations

- The `CPU` struct caches metrics until `update()` is called
- For continuous monitoring, call `update()` periodically (e.g., every 1-2 seconds)
- Avoid calling `update()` too frequently as it involves system calls

## Error Handling

Most methods that interact with the system return a `Result` type that should be handled appropriately:

```rust,no_run,ignore
match CPU::new() {
    Ok(cpu) => {
        // Use the CPU instance
    },
    Err(e) => {
        eprintln!("Failed to initialize CPU monitoring: {}", e);
    }
}
```

## Advanced Usage

### Implementing the CpuMetrics Trait

You can implement the `CpuMetrics` trait for your own types to provide a consistent interface:

```rust,no_run,ignore
use darwin_metrics::hardware::cpu::CpuMetrics;

struct MyCpuMonitor {
    usage: f64,
    temperature: Option<f64>,
    frequency: f64,
}

impl CpuMetrics for MyCpuMonitor {
    fn get_cpu_usage(&self) -> f64 {
        self.usage // Return stored value
    }

    fn get_cpu_temperature(&self) -> Option<f64> {
        self.temperature // Return stored temperature if available
    }

    fn get_cpu_frequency(&self) -> f64 {
        self.frequency // Return stored frequency
    }
}
```

## Integration Examples

### Periodic Monitoring

```rust,no_run,ignore
use std::thread;
use std::time::Duration;
use darwin_metrics::hardware::cpu::CPU;
use darwin_metrics::error::Result;

fn monitor_cpu() -> Result<()> {
    let mut cpu = CPU::new()?;

    for _ in 0..10 {
        // Update metrics
        cpu.update()?;

        // Display current usage
        println!("CPU Usage: {:.1}%", cpu.get_cpu_usage() * 100.0);

        // Wait before next update
        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}
```

### Usage with Async

```rust,no_run,ignore
use tokio::time::{interval, Duration};
use darwin_metrics::hardware::cpu::CPU;
use darwin_metrics::error::Result;

async fn monitor_cpu_async() -> Result<()> {
    let mut cpu = CPU::new()?;
    let mut interval = interval(Duration::from_secs(1));

    for _ in 0..10 {
        interval.tick().await;

        cpu.update()?;
        println!("CPU: {:.1}%", cpu.get_cpu_usage() * 100.0);
    }

    Ok(())
}
```
