#![doc(html_root_url = "https://docs.rs/darwin-metrics/0.2.0-alpha.1")]
#![cfg_attr(not(any(test, feature = "testing")), warn(missing_docs))]
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

#[cfg_attr(any(test, feature = "mock"), doc = "Error handling module exposed for testing")]
#[cfg_attr(not(any(test, feature = "mock")), doc = "Error handling module")]
pub mod error;

#[cfg_attr(any(test, feature = "mock"), doc = "Utility functions and helpers exposed for testing")]
#[cfg_attr(not(any(test, feature = "mock")), doc = "Utility functions and helpers")]
pub mod utils;

/// Core functionality for metrics and monitoring
pub mod core;

#[cfg_attr(any(test, feature = "mock"), doc = "Hardware monitoring functionality exposed for testing")]
#[cfg_attr(not(any(test, feature = "mock")), doc = "Hardware monitoring functionality")]
pub mod hardware;

/// Network related metrics and monitoring
pub mod network;

/// Power related metrics and monitoring
pub mod power;

/// Process related metrics and monitoring
pub mod process;

/// Resource monitoring and management
pub mod resource;

/// System information functionality
pub mod system;

/// Traits for hardware monitoring
pub mod traits;

// Re-export core functionality through the prelude
pub use core::prelude::*;

// Re-export error types
pub use error::{Error, Result};

// Re-export hardware monitoring types
pub use hardware::{
    battery::Battery,
    // CPU monitoring
    cpu::{CpuTemperatureMonitor, CpuUtilizationMonitor, CPU},
    // Disk monitoring
    disk::{Disk, DiskHealthMonitor, DiskMountMonitor, DiskPerformanceMonitor, DiskStorageMonitor},
    // GPU monitoring
    gpu::{Gpu, GpuCharacteristicsMonitor, GpuMemoryMonitor, GpuTemperatureMonitor, GpuUtilizationMonitor},
    // IOKit
    iokit::IOKitImpl,
    // Memory monitoring
    memory::{Memory, MemoryInfo, MemoryPressureMonitor, MemoryUsageMonitor},
    // Temperature monitoring
    temperature::{Fan, ThermalMetrics},
};

// Re-export system monitoring types

// Re-export network monitoring types
pub use network::{NetworkInfo, NetworkInterface, NetworkMonitor};

// Re-export power monitoring types
pub use power::{PowerInfo, PowerState};

// Re-export process monitoring types
pub use process::{
    ProcessInfo,
    // Use the correct path for ProcessIOMonitor
};

// Re-export resource monitoring types
pub use resource::{Cache, ResourceManager, ResourceMonitor, ResourceMonitoring, ResourcePool, ResourceUpdate};

/// Creates a new Battery instance
pub fn new_battery() -> Result<Battery> {
    let iokit = Box::new(IOKitImpl::new()?);
    Battery::new(iokit)
}

/// Creates a new CPU instance
pub fn new_cpu() -> Result<CPU> {
    let iokit = Box::new(IOKitImpl::new()?);
    Ok(CPU::new(iokit))
}

/// Creates a new GPU instance
pub fn new_gpu() -> Result<Gpu> {
    Gpu::new()
}

/// Creates a new Memory instance
pub fn new_memory() -> Result<Memory> {
    Memory::new()
}

/// Creates a new Temperature instance
pub fn new_temperature() -> Result<hardware::temperature::Temperature> {
    hardware::temperature::Temperature::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_monitors() -> Result<()> {
        let _battery = new_battery()?;
        let _cpu = new_cpu()?;
        let _gpu = new_gpu()?;
        let _memory = new_memory()?;
        let _temperature = new_temperature()?;
        Ok(())
    }

    use crate::utils::ffi::SmcKey;

    #[test]
    fn test_smc_key_from_chars() {
        let key = SmcKey::from_chars(['T', 'A', '0', 'P']);
        assert_eq!(key.to_string(), "TA0P");
    }
}

// Re-export types for convenience
pub use crate::{
    core::{
        metrics::{
            hardware::{CpuMonitor, GpuMonitor, HardwareMonitor, MemoryMonitor, NetworkInterfaceMonitor},
            Metric,
        },
        types::{Percentage, Temperature},
    },
    hardware::disk::{DiskHealth, DiskMount, DiskPerformance},
    hardware::gpu::{GpuMemory, GpuUtilization},
    network::Interface,
    process::Process,
};
