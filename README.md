# 🦀 darwin-metrics

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/darwin-metrics.svg)](https://crates.io/crates/darwin-metrics)
[![Documentation](https://docs.rs/darwin-metrics/badge.svg)](https://docs.rs/darwin-metrics)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Build Status](https://img.shields.io/github/actions/workflow/status/sm-moshi/darwin-metrics/ci.yml?branch=main)
![Crates.io Downloads](https://img.shields.io/crates/d/darwin-metrics)

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
  - [ ] Swap space monitoring
  - [x] Memory pressure levels

- **GPU Information**
  - [ ] Active GPU model detection
  - [ ] GPU utilization metrics
  - [ ] VRAM consumption tracking

</td>
<td>

### 📊 Resource Tracking

- **Storage Metrics**

  - [x] Disk space utilization
  - [ ] I/O performance monitoring
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
  - [x] System uptime and version info

- **Network Monitoring**
  - [x] Interface discovery and state tracking
  - [x] Traffic statistics (bytes/packets sent/received)
  - [x] Bandwidth calculations

</td>
</tr>
</table>

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
darwin-metrics = "0.1.0"
```

### 🔧 Requirements

- macOS 10.11 (El Capitan) or later
- Rust 1.75 or later
- Xcode Command Line Tools

## 🚀 Quick Start

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

## 🎯 Feature Flags

| Flag             | Description                           |
| ---------------- | ------------------------------------- |
| `battery`        | Enable battery monitoring             |
| `cpu`            | Enable CPU metrics                    |
| `memory`         | Enable memory statistics              |
| `gpu`            | Enable GPU monitoring                 |
| `disk`           | Enable storage metrics                |
| `temperature`    | Enable thermal monitoring             |
| `async`          | Enable async support (requires tokio) |
| `metrics-export` | Enable metrics export functionality   |
| `cached-metrics` | Enable caching for expensive calls    |

## 📈 Development Status

Currently in active development. See our [roadmap](docs/ROADMAP.md) for detailed development plans.

<details>
<summary><b>Development Progress</b></summary>

### ✅ Completed

- [x] Initial project setup
- [x] Basic crate structure
- [x] CPU monitoring module implementation
- [x] Network monitoring module implementation

### 🚧 In Progress

- [ ] Memory analysis module implementation
- [ ] GPU monitoring refinement
- [ ] Documentation improvements
- [ ] Performance optimizations

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
