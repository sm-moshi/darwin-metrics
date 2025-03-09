# Getting Started

This guide will help you get started with using `darwin-metrics` in your Rust projects.

## Installation

Add `darwin-metrics` to your `Cargo.toml`:

```toml
[dependencies]
darwin-metrics = "0.1.3"
```

### Requirements

- macOS Sonoma (14.x) or later
- Rust 1.85 or later
- Xcode Command Line Tools (for macOS system headers)

## Basic Usage

Here's a simple example showing how to retrieve basic system metrics:

```rust,no_run,ignore
// Basic example of darwin-metrics usage
use darwin_metrics::{hardware, process};
use std::time::Duration;

fn main() -> darwin_metrics::Result<()> {
    // Get CPU information
    let cpu_info = hardware::cpu::Cpu::get_metrics()?;
    println!("CPU Usage: {}%", cpu_info.usage);

    // Get memory information
    let memory = hardware::memory::Memory::get_info()?;
    println!("Memory Used: {} bytes", memory.used);
    println!("Memory Available: {} bytes", memory.available);

    // Get process information
    let processes = process::Process::get_all().unwrap();
    println!("Running processes: {}", processes.len());

    Ok(())
}
```

## Using Feature Flags

`darwin-metrics` is organized with feature flags to allow you to include only the modules you need:

```toml
[dependencies]
darwin-metrics = { version = "0.1.3", features = ["cpu", "memory"] }
```

Available features include:

- `cpu` - CPU monitoring
- `memory` - Memory statistics
- `gpu` - GPU monitoring
- `process` - Process information
- `thermal` - Temperature sensors
- `power` - Power and battery information

## Async Support

For non-blocking operations, `darwin-metrics` provides async functionality:

```rust,no_run,ignore
// Async example with Tokio
use darwin_metrics::{hardware, process};
use std::time::Duration;
use futures::stream::StreamExt; // For the StreamExt trait

#[tokio::main]
async fn main() -> darwin_metrics::Result<()> {
    // Monitor process metrics over time
    let pid = std::process::id();
    let mut stream = process::Process::monitor_metrics(pid, Duration::from_secs(1));

    // Process the metrics stream
    while let Some(process) = stream.next().await {
        if let Ok(proc) = process {
            println!("Process: {} - CPU: {}%, Memory: {}",
                proc.name, proc.cpu_usage, proc.memory_usage);
        }
    }

    Ok(())
}
```

## Error Handling

`darwin-metrics` provides a robust error handling system that helps you understand what went wrong:

```rust,no_run,ignore
// Error handling example
use darwin_metrics::{Error, Result};

// This is just a hypothetical function to demonstrate error handling
fn potentially_failing_function() -> Result<String> {
    Ok(String::from("success"))
}

fn example() -> Result<()> {
    let result = potentially_failing_function();

    match result {
        Ok(value) => println!("Success: {}", value),
        Err(Error::NotAvailable(msg)) => println!("Resource not available: {}", msg),
        Err(Error::System(msg)) => println!("System error: {}", msg),
        Err(Error::Process(msg)) => println!("Process error: {}", msg),
        Err(err) => println!("Other error: {}", err),
    }

    Ok(())
}
```

## Next Steps

Explore the documentation for each module to learn more about available metrics and functionality:

- [CPU Monitoring](./modules/cpu.md)
- [Memory Analysis](./modules/memory.md)
- [Process Monitoring](./modules/process.md)
- [GPU Information](./modules/gpu.md)
