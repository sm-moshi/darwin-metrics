# 🦀 darwin-metrics

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/darwin-metrics.svg)](https://crates.io/crates/darwin-metrics)
[![Documentation](https://docs.rs/darwin-metrics/badge.svg)](https://docs.rs/darwin-metrics)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/sm-moshi/darwin-metrics/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/sm-moshi/darwin-metrics/actions/workflows/ci.yml)
[![Crates.io Downloads](https://img.shields.io/crates/d/darwin-metrics)](https://crates.io/crates/darwin-metrics)

</div>

A Rust library providing native access to macOS system metrics through low-level system APIs. This crate offers efficient, safe, and async-capable interfaces for monitoring system resources on macOS.

## 🌟 Features

<table>
<tr>
<td>

### 🖥️ System Monitoring

- **CPU Monitoring**

  - [x] Per-core usage statistics
  - [x] CPU model and frequency information
  - [x] System load metrics (user, system, idle)

- **Memory Analysis**

  - [x] RAM usage and availability
  - [x] Swap space monitoring
  - [x] Memory pressure levels

- **GPU Information**
  - [x] Active GPU model detection
  - [x] GPU utilization metrics
  - [x] VRAM consumption tracking

</td>
<td>

### 📊 Resource Tracking

- **Storage Metrics**

  - [x] Disk space utilization
  - [x] I/O performance monitoring
  - [x] Read/write speed tracking

- **Power Management**

  - [x] Battery status and health
  - [x] Charging state detection
  - [x] Remaining battery time estimation

- **Thermal Monitoring**
  - [x] Fan speed readings
  - [x] CPU and GPU temperature tracking
  - [x] System-wide thermal status

</td>
</tr>
<tr>
<td colspan="2">

### 🔌 Additional Features

- **Process Information**

  - [x] Running process enumeration
  - [x] Per-process resource usage
  - [x] Parent-child process relationship tracking
  - [x] Process tree visualization

- **Network Monitoring**
  - [x] Interface discovery and state tracking
  - [x] Traffic statistics (bytes/packets sent/received)
  - [x] Bandwidth calculations
  - [x] Async network monitoring

</td>
</tr>
</table>

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
darwin-metrics = "0.2.0-alpha.1"  # Development version

# Or for latest development features:
# darwin-metrics = { git = "https://github.com/sm-moshi/darwin-metrics", branch = "0.2.x" }
```

### 🔧 Requirements

- macOS 14.5 (Ventura) or later
- Rust 1.85 or later
- Xcode Command Line Tools

## 🚀 Quick Start

```rust
use darwin_metrics::{CPU, Memory, Gpu};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get CPU information
    let cpu = CPU::new();
    println!("CPU cores: {}", cpu.cores());
    println!("CPU usage: {}%", cpu.usage()?);
    
    // Monitor memory
    let mut memory = Memory::new()?;
    memory.update()?;
    println!("Memory used: {:.2} GB", memory.used as f64 / 1_073_741_824.0);
    println!("Memory pressure: {:.1}%", memory.pressure_percentage());
    
    // Check GPU status
    let gpu = Gpu::new()?;
    println!("GPU name: {}", gpu.name()?);
    
    Ok(())
}
```

## 🎯 Feature Flags

All features are enabled by default, but you can selectively enable only what you need:

| Flag                | Description                               |
| ------------------- | ----------------------------------------- |
| `battery`           | Enable battery monitoring                 |
| `cpu`               | Enable CPU metrics                        |
| `memory`            | Enable memory statistics                  |
| `gpu`               | Enable GPU monitoring                     |
| `disk`              | Enable storage metrics                    |
| `temperature`       | Enable thermal monitoring                 |
| `async`             | Enable async support (requires tokio)     |
| `process_monitoring`| Enable detailed process monitoring        |
| `unstable-tests`    | Enable tests that may be unstable in CI   |

## 📈 Development Status

Currently in active development. See our [roadmap](docs/ROADMAP.md) for detailed development plans.

<details>
<summary><b>Development Progress</b></summary>

### ✅ Completed (v0.1.0)

- [x] Initial project setup
- [x] Core architecture and error handling
- [x] CPU monitoring module with frequency data
- [x] Memory monitoring with pressure levels
- [x] GPU information and metrics
- [x] Network interface discovery and traffic stats
- [x] Disk space monitoring
- [x] Process monitoring and hierarchy tracking
- [x] Temperature sensors and fan speed tracking

### 🚧 In Progress (v0.2.0)

- [x] Enhanced async support throughout
- [ ] Metal API integration for improved GPU monitoring
- [ ] Memory management optimizations for IOKit interfaces
- [ ] Cross-platform abstractions (Linux/Windows)
- [ ] Metrics export to Prometheus/InfluxDB
- [ ] Performance optimizations
- [ ] Event-based monitoring

</details>

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

<details>
<summary><b>Development Setup</b></summary>

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

</details>

## 📝 How to Cite

If you use darwin-metrics in your projects, please include one of the following attributions:

<details>
<summary><b>Citation Formats</b></summary>

### 💻 For Software Projects

```markdown
This project uses darwin-metrics (https://github.com/sm-moshi/darwin-metrics) by Stuart Meya.
```

### 📚 For Documentation or Technical Writing

```markdown
darwin-metrics: A Rust library for native macOS system metrics, developed by Stuart Meya.
GitHub repository: https://github.com/sm-moshi/darwin-metrics
```

### 🎓 For Academic or Research Use

```markdown
Meya, S. (2025). darwin-metrics: A Rust library for native macOS system metrics.
GitHub repository: https://github.com/sm-moshi/darwin-metrics
```

For more detailed attribution requirements, please see the [NOTICE](NOTICE) file.

</details>

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Apple's [IOKit](https://developer.apple.com/documentation/iokit), [Foundation](https://developer.apple.com/documentation/foundation), [Core Foundation](https://developer.apple.com/documentation/corefoundation), and [Metal](https://developer.apple.com/documentation/metal) documentation
- [objc2](https://github.com/mattn/objc2) crate by Mads Marquart
- The Rust and Swift communities
- Contributors to the core dependencies
