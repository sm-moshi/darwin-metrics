#![doc(html_root_url = "https://docs.rs/darwin-metrics/0.1.6")]
//#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! A library for monitoring system metrics on macOS/Darwin systems.
//!
//! This crate provides a high-level interface for accessing various system metrics including CPU, GPU, memory, battery,
//! and thermal information.
//!
//! This library provides a comprehensive set of tools for monitoring various system metrics on macOS systems. It
//! includes support for:
//!
//! - Battery monitoring
//! - Disk usage and I/O statistics
//! - Hardware information (CPU, GPU, Memory)
//! - Power management
//! - Process monitoring
//! - System information
//! - Network monitoring
//!
//! The library uses native macOS APIs through IOKit and other system frameworks to provide accurate and efficient
//! monitoring capabilities.
//!
//! # Examples
//!
//! ```rust
//! use darwin_metrics::hardware::iokit::IOKitImpl;
//! use darwin_metrics::Result;
//!
//! fn main() -> Result<()> {
//!     let iokit = IOKitImpl::new();
//!     
//!     // Get CPU temperature
//!     let temp = iokit.get_cpu_temperature()?;
//!     println!("CPU Temperature: {:.1}°C", temp);
//!     
//!     // Get battery status
//!     let battery = darwin_metrics::battery::Battery::new(Box::new(IOKitImpl::new()))?;
//!     println!("Battery level: {}%", battery.percentage());
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Features
//!
//! - `serde`: Enables serialization/deserialization support for metric types
//! - `async`: Enables async versions of metric collection methods
//! - `unstable`: Enables experimental features that may change in future versions
//!
//! # Safety and Platform Compatibility
//!
//! This crate uses macOS-specific APIs through FFI and is only compatible with macOS systems. All FFI calls are wrapped
//! in safe abstractions, with thorough error handling and resource cleanup.
//!
//! # Error Handling
//!
//! Operations that can fail return a `Result<T, Error>` where `Error` is this crate's error type. The error type
//! provides detailed information about what went wrong, including:
//!
//! - I/O errors
//! - System API errors
//! - Permission errors
//! - Resource unavailability
//!
//! # Thread Safety
//!
//! All types in this crate are thread-safe and can be shared across threads using standard synchronization primitives.
//! Many types implement `Send` and `Sync` where appropriate.

pub mod battery;
pub mod disk;
pub mod error;
pub mod hardware;
pub mod network;
pub mod power;
pub mod process;
pub mod system;
pub mod utils;

pub use battery::Battery;
pub use error::{Error, Result};
pub use hardware::{
    cpu::CPU,
    gpu::Gpu,
    iokit::{IOKit, IOKitImpl},
    memory::Memory,
    temperature::Temperature,
};

// Re-export primary modules for direct access
#[doc(inline)]
pub use disk::{Disk, DiskConfig, DiskType};

#[doc(inline)]
pub use hardware::cpu::FrequencyMetrics;

#[doc(inline)]
pub use hardware::gpu::GpuMetrics;

#[doc(inline)]
pub use hardware::memory::{PageStates, PressureLevel, SwapUsage};

#[doc(inline)]
pub use hardware::temperature::{Fan, ThermalMetrics};

#[doc(inline)]
pub use network::{Interface as NetworkInterface, TrafficData as NetworkTraffic};

#[doc(inline)]
pub use process::{Process, ProcessInfo};

/// Creates a new instance of the Battery monitor.
pub fn new_battery() -> Result<Battery> {
    Battery::new(Box::new(IOKitImpl::new()))
}

/// Creates a new instance of the CPU monitor.
pub fn new_cpu() -> Result<CPU> {
    CPU::new()
}

/// Creates a new instance of the GPU monitor.
pub fn new_gpu() -> Result<Gpu> {
    Gpu::new()
}

/// Creates a new instance of the Memory monitor.
pub fn new_memory() -> Result<Memory> {
    Memory::new()
}

/// Creates a new instance of the Temperature monitor.
pub fn new_temperature() -> Result<Temperature<IOKitImpl>> {
    Ok(Temperature::new())
}

#[cfg(test)]
mod tests;
