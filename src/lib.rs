#![doc(html_root_url = "https://docs.rs/darwin-metrics/0.2.0-alpha.1")]

//! # darwin-metrics
//!
//! `darwin-metrics` is a Rust library that provides native access to macOS system metrics through low-level system
//! APIs. This crate offers efficient, safe, and async-capable interfaces for monitoring system resources on macOS.
//!
//! ## Features
//!
//! - **CPU Monitoring**: Usage statistics, frequency information, model details
//! - **Memory Analysis**: RAM usage, swap space, memory pressure
//! - **GPU Information**: Model detection, utilization metrics, VRAM tracking
//! - **Storage Metrics**: Disk space, I/O performance, read/write speeds
//! - **Power Management**: Battery status, charging state, time estimation
//! - **Thermal Monitoring**: Fan speeds, temperature tracking, thermal status
//! - **Process Information**: Process enumeration, resource usage, system info
//! - **Network Monitoring**: Interface discovery, traffic statistics, bandwidth
//!
//! ## Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! darwin-metrics = "0.2.0-alpha.1"
//! ```
//!
//! ## Requirements
//!
//! - macOS 10.11 (El Capitan) or later
//! - Rust 1.75 or later
//! - Xcode Command Line Tools
//!
//! ## Quick Start
//!
//! ```ignore
//! // This example won't be run by doctests but serves as API usage documentation
//! use darwin_metrics::hardware::{cpu, gpu, temperature};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Get CPU information
//!     let cpu_obj = cpu::CPU::new();
//!     println!("CPU cores: {}", cpu_obj.cores());
//!     
//!     // Monitor temperature
//!     let mut temp_monitor = temperature::Temperature::new();
//!     let cpu_temp = temp_monitor.cpu_temperature()?;
//!     println!("CPU Temperature: {:.1}°C", cpu_temp);
//!     
//!     // Check thermal metrics
//!     let metrics = temp_monitor.get_thermal_metrics()?;
//!     println!("Is CPU throttling: {}", metrics.is_throttling);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Flags
//!
//! ### Core Features (Enabled by Default)
//!
//! - `battery` - Enable battery monitoring
//! - `cpu` - Enable CPU metrics
//! - `memory` - Enable memory statistics
//! - `gpu` - Enable GPU monitoring
//! - `disk` - Enable storage metrics
//! - `temperature` - Enable thermal monitoring
//! - `async` - Enable async support (requires tokio)
//!
//! ### Additional Features
//!
//! - `process_monitoring` - Enable detailed process monitoring
//! - `unstable-tests` - Enable tests that may be unstable in CI environments
//!
//! ## Module Structure
//!
//! - [`battery`] - Battery information and power metrics
//! - [`hardware`] - Hardware monitoring:
//!   - [`hardware::cpu`] - CPU usage, frequency, and core information
//!   - [`hardware::gpu`] - GPU metrics and memory usage
//!   - [`hardware::memory`] - System memory statistics
//!   - [`hardware::temperature`] - Temperature sensors and fan control
//! - [`network`] - Network interfaces and traffic statistics
//! - [`power`] - Power consumption and management
//! - [`process`] - Process monitoring and management
//! - [`system`] - Overall system information
//!
//! ## Error Handling
//!
//! The crate provides a centralized [`Error`] type that encompasses all possible error conditions and a convenient
//! [`Result`] type alias.
//!
//! ```
//! # fn foo() {
//! use darwin_metrics::Result;
//!
//! fn example() -> Result<()> {
//!     // Function implementation...
//!     Ok(())
//! }
//! # }
//! ```
//!
//! ## Async Support
//!
//! When the `async` feature is enabled, the crate provides async versions of monitoring functions that can be used with
//! the tokio runtime.
//!
//! ```ignore
//! use darwin_metrics::hardware::temperature::Temperature;
//!
//! async fn example() -> darwin_metrics::Result<()> {
//!     let mut temp = Temperature::new();
//!     let metrics = temp.get_thermal_metrics_async().await?;
//!     println!("CPU temperature: {:?}°C", metrics.cpu_temperature);
//!     Ok(())
//! }
//! ```

pub mod battery;
pub mod disk;
pub mod error;
pub mod hardware;
pub mod network;
pub mod power;
pub mod process;
pub mod system;
pub mod utils;

// Re-export the core error types for easier use
#[doc(inline)]
pub use error::{Error, Result};

// Re-export primary modules for direct access
#[doc(inline)]
pub use battery::Battery;

#[doc(inline)]
pub use disk::{Disk, DiskConfig, DiskType};

#[doc(inline)]
pub use hardware::cpu::{FrequencyMetrics, CPU};

#[doc(inline)]
pub use hardware::gpu::{Gpu, GpuMetrics};

#[doc(inline)]
pub use hardware::memory::{Memory, PageStates, PressureLevel, SwapUsage};

#[doc(inline)]
pub use hardware::temperature::{Fan, Temperature, ThermalMetrics};

#[doc(inline)]
pub use network::{Interface as NetworkInterface, TrafficData as NetworkTraffic};

#[doc(inline)]
pub use process::{Process, ProcessInfo};
