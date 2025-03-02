# darwin-metrics

A Rust library that provides native access to macOS system metrics through Swift and low-level system APIs. This crate offers efficient, safe, and async-capable interfaces for monitoring system resources on macOS.

[![Crates.io](https://img.shields.io/crates/v/darwin-metrics.svg)](https://crates.io/crates/darwin-metrics)
[![Documentation](https://docs.rs/darwin-metrics/badge.svg)](https://docs.rs/darwin-metrics)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **CPU Monitoring**
  - Per-core usage statistics
  - CPU model and frequency information
  - System load metrics (user, system, idle)

- **Memory Analysis**
  - RAM usage and availability
  - Swap space monitoring
  - Memory pressure levels

- **GPU Information**
  - Active GPU model detection
  - GPU utilization metrics
  - VRAM consumption tracking

- **Storage Metrics**
  - Disk space utilization
  - I/O performance monitoring
  - Read/write speed tracking

- **Power Management**
  - Battery status and health
  - Charging state detection
  - Remaining battery time estimation

- **Thermal Monitoring**
  - Fan speed readings
  - CPU and GPU temperature tracking
  - System-wide thermal status

- **Process Information**
  - Running process enumeration
  - Per-process resource usage
  - System uptime and version info

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
darwin-metrics = "0.1.0"
```

### Requirements

- macOS 15 or later
- Rust 1.75 or later
- Xcode Command Line Tools (for Swift compilation)

## Usage

```rust
use darwin_metrics::{cpu, memory, gpu};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get CPU usage
    let cpu_info = cpu::get_usage().await?;
    println!("CPU Usage: {}%", cpu_info.total_usage);

    // Monitor memory
    let mem_info = memory::get_stats()?;
    println!("Memory Used: {} GB", mem_info.used_gb);

    // Check GPU status
    let gpu_info = gpu::get_info()?;
    println!("Active GPU: {}", gpu_info.model);

    Ok(())
}
```

## Feature Flags

- `battery` - Enable battery monitoring
- `cpu` - Enable CPU metrics
- `memory` - Enable memory statistics
- `gpu` - Enable GPU monitoring
- `disk` - Enable storage metrics
- `temperature` - Enable thermal monitoring
- `async` - Enable async support (requires tokio)
- `metrics-export` - Enable metrics export functionality
- `cached-metrics` - Enable caching for expensive calls

## Development Status

Currently in active development. See our [roadmap](docs/ROADMAP.md) for detailed development plans.

### Completed

- [x] Initial project setup
- [x] Basic crate structure
- [x] Swift bridge integration planning

### In Progress

- [ ] Core metric collection implementations
- [ ] Swift API bindings
- [ ] Async support integration
- [ ] Documentation

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

1. Clone the repository:

```bash
git clone https://github.com/sm-moshi/darwin-metrics.git
cd darwin-metrics
```

2. Install dependencies:

```bash
xcode-select --install  # Install Xcode Command Line Tools
```

3. Build the project:

```bash
cargo build --all-features
```

4. Run tests:

```bash
cargo test --all-features
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Apple's [IOKit](https://developer.apple.com/documentation/iokit) documentation
- The Rust and Swift communities
- Contributors to the core dependencies
