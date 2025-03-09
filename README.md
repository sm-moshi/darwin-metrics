# 🦀 darwin-metrics

A Rust library that provides native access to macOS system metrics through low-level system APIs. This crate offers efficient, safe, and async-capable interfaces for monitoring system resources on macOS.

[![Crates.io](https://img.shields.io/crates/v/darwin-metrics.svg)](https://crates.io/crates/darwin-metrics)
[![Documentation](https://docs.rs/darwin-metrics/badge.svg)](https://docs.rs/darwin-metrics)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## ✨ Features

### 🔄 System Monitoring

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

### 💾 Resource Tracking

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

---

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
darwin-metrics = "0.1.0"
```

### 🔧 Requirements

- macOS 14 or later
- Rust 1.75 or later
- Xcode Command Line Tools

---

## 🚀 Usage

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

---

## 🎯 Feature Flags

- `battery` - Enable battery monitoring
- `cpu` - Enable CPU metrics
- `memory` - Enable memory statistics
- `gpu` - Enable GPU monitoring
- `disk` - Enable storage metrics
- `temperature` - Enable thermal monitoring
- `async` - Enable async support (requires tokio)
- `metrics-export` - Enable metrics export functionality
- `cached-metrics` - Enable caching for expensive calls

---

## 📈 Development Status

Currently in active development. See our [roadmap](docs/ROADMAP.md) for detailed development plans.

### ✅ Completed

- [x] Initial project setup
- [x] Basic crate structure

### 🚧 In Progress

- [ ] Core metric collection implementations
- [ ] Async support integration
- [ ] Documentation improvements
- [ ] Performance optimizations

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### 🛠️ Development Setup

1. Clone the repository:

```bash
git clone https://github.com/sm-moshi/darwin-metrics.git
cd darwin-metrics
```

1. Install dependencies:

```bash
xcode-select --install  # Install Xcode Command Line Tools
```

1. Build the project:

```bash
cargo build --all-features
```

1. Run tests:

```bash
cargo test --all-features
```

---

## 📝 How to Cite

If you use darwin-metrics in your projects, please include one of the following attributions:

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

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- Apple's [IOKit](https://developer.apple.com/documentation/iokit) documentation
- Apple's [Foundation](https://developer.apple.com/documentation/foundation) documentation
- Apple's [Core Foundation](https://developer.apple.com/documentation/corefoundation) documentation
- Apple's [Core Graphics](https://developer.apple.com/documentation/coregraphics) documentation
- Apple's [Metal](https://developer.apple.com/documentation/metal) documentation
- Mads Marquart's [objc2](https://github.com/mattn/objc2) crate
- The Rust and Swift communities
- Contributors to the core dependencies
