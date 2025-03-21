#![doc(html_root_url = "https://docs.rs/darwin-metrics/0.2.0-alpha.1")]
#![cfg_attr(not(any(test, feature = "testing")), warn(missing_docs))]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # darwin-metrics
//!
//! A library for monitoring system metrics on macOS/Darwin systems.
//!
//! This crate provides a high-level interface for accessing various system metrics including CPU, GPU, memory, battery,
//! and thermal information.
//!
//! ## Features
//!
//! This library provides a comprehensive set of tools for monitoring various system metrics on macOS systems:
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
//! ## Async Support
//!
//! Many functions in this crate provide both synchronous and asynchronous versions. The synchronous versions
//! (in the crate root) create a Tokio runtime under the hood. If you're already in an async context, use the
//! async versions directly from their respective modules.
//!
//! ## Examples
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
//! ## Feature Flags
//!
//! - `serde`: Enables serialization/deserialization support for metric types
//! - `async`: Enables async versions of metric collection methods
//! - `unstable`: Enables experimental features that may change in future versions
//!
//! ## Safety and Platform Compatibility
//!
//! This crate uses macOS-specific APIs through FFI and is only compatible with macOS systems. All FFI calls are wrapped
//! in safe abstractions, with thorough error handling and resource cleanup.
//!
//! ## Error Handling
//!
//! Operations that can fail return a `Result<T, Error>` where `Error` is this crate's error type. The error type
//! provides detailed information about what went wrong, including:
//!
//! - I/O errors
//! - System API errors
//! - Permission errors
//! - Resource unavailability
//!
//! ## Thread Safety
//!
//! All types in this crate are thread-safe and can be shared across threads using standard synchronization primitives.
//! Many types implement `Send` and `Sync` where appropriate.

use std::sync::Arc;

use crate::hardware::iokit::IOKitImpl;

// ===== Module Declarations =====

/// Error handling module.
#[cfg_attr(any(test, feature = "mock"), doc = "Error handling module exposed for testing")]
#[cfg_attr(not(any(test, feature = "mock")), doc = "Error handling module")]
pub mod error;

/// Utility functions and helpers.
#[cfg_attr(
    any(test, feature = "mock"),
    doc = "Utility functions and helpers exposed for testing"
)]
#[cfg_attr(not(any(test, feature = "mock")), doc = "Utility functions and helpers")]
pub mod utils;

/// Core functionality for metrics and monitoring.
pub mod core;

/// CPU monitoring functionality.
pub mod cpu;

/// GPU monitoring module.
pub mod gpu;

/// Hardware monitoring functionality.
#[cfg_attr(
    any(test, feature = "mock"),
    doc = "Hardware monitoring functionality exposed for testing"
)]
#[cfg_attr(not(any(test, feature = "mock")), doc = "Hardware monitoring functionality")]
pub mod hardware;

/// Network related metrics and monitoring.
pub mod network;

/// Power related metrics and monitoring.
pub mod power;

/// Process related metrics and monitoring.
pub mod process;

/// Resource monitoring and management.
pub mod resource;

/// System information functionality.
pub mod system;

/// Temperature monitoring functionality.
pub mod temperature;

/// Traits for hardware monitoring.
pub mod traits;

/// Battery monitoring module.
pub mod battery;

/// Disk monitoring module.
pub mod disk;

/// Memory monitoring module.
pub mod memory;

// ===== Re-exports =====

// Re-export error types
// Re-export core types
pub use core::metrics::Metric;
pub use core::types::{ByteSize, DiskHealth, DiskIO, DiskSpace, Percentage, Temperature, Transfer};

// Re-export battery module
pub use battery::Battery;
// Re-export CPU monitoring types
pub use cpu::{
    CPU, CpuFrequencyMonitor, CpuTemperatureMonitor, CpuUtilizationMonitor, FrequencyMetrics, FrequencyMonitor,
};
// Re-export disk module types
pub use disk::{Disk, DiskConfig, DiskMount, DiskPerformance, DiskType};
pub use error::{Error, Result};
// Re-export GPU monitoring types
pub use gpu::{Gpu, GpuMemoryMonitor, GpuUtilizationMonitor};
// Re-export hardware monitoring types
pub use hardware::iokit::IOKit;
// Re-export memory monitoring types
pub use memory::{
    Memory, MemoryInfo, MemoryPressureMonitor, MemoryUsageMonitor, PageStates, PressureLevel, SwapMonitor, SwapUsage,
};
// Re-export network monitoring types
pub use network::{Interface, NetworkInfo, NetworkInterface, NetworkMonitor};
// Re-export power monitoring types
pub use power::{PowerInfo, PowerState};
// Re-export process monitoring types
pub use process::{Process, ProcessInfo};
// Re-export resource monitoring types
pub use resource::{Cache, ResourceManager, ResourceMonitor, ResourceMonitoring, ResourcePool, ResourceUpdate};
pub use temperature::TemperatureFactory as NewTemperature;
pub use temperature::monitors::{
    AmbientTemperatureMonitor, BatteryTemperatureMonitor, CpuTemperatureMonitor as NewCpuTemperatureMonitor,
    GpuTemperatureMonitor,
};
pub use temperature::types::{Fan, ThermalMetrics};
// Re-export trait types
pub use traits::{
    ByteMetricsMonitor, CpuMonitor, DiskHealthMonitor, DiskMountMonitor, DiskPerformanceMonitor, GpuMonitor,
    HardwareMonitor, MemoryMonitor, NetworkInterfaceMonitor, RateMonitor, StorageMonitor, TemperatureMonitor,
    UtilizationMonitor,
};

// Also re-export disk-specific traits via prelude
pub use crate::traits::hardware::{DiskIOMonitor, DiskStorageMonitor, DiskUtilizationMonitor};

// ===== Sync API Helpers =====

/// Creates a new Battery instance.
pub fn new_battery() -> Result<Battery> {
    let iokit = Arc::new(IOKitImpl::new()?);
    Ok(Battery::new(iokit))
}

/// Creates a new CPU instance.
pub fn new_cpu() -> Result<cpu::CPU> {
    let iokit = Box::new(IOKitImpl::new()?);
    Ok(cpu::CPU::new(iokit))
}

/// Creates a new GPU instance.
pub fn new_gpu() -> Result<Gpu> {
    Gpu::new()
}

/// Creates a new Memory instance.
pub fn new_memory() -> Result<Memory> {
    Memory::new()
}

/// Create a new Temperature instance.
pub fn new_temperature() -> Result<temperature::TemperatureFactory> {
    Ok(temperature::TemperatureFactory::new()?)
}

/// Get information about the root filesystem (/).
///
/// This function creates a new tokio runtime to execute the async operation.
/// If you're already in an async context, use [`disk::get_root_disk()`] directly.
pub fn get_root_disk() -> Result<disk::Disk> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(disk::get_root_disk())
}

/// Get information about all mounted disks.
///
/// This function creates a new tokio runtime to execute the async operation.
/// If you're already in an async context, use [`disk::get_all_disks()`] directly.
pub fn get_all_disks() -> Result<Vec<disk::Disk>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(disk::get_all_disks())
}

// ===== Prelude Module =====

/// Commonly used types and traits re-exported for convenience.
///
/// This module follows the pattern of the standard library's prelude,
/// providing the most commonly used components for convenient importing.
pub mod prelude {
    // Core types
    // Hardware components
    pub use crate::battery::Battery;
    pub use crate::core::metrics::Metric;
    pub use crate::core::types::{ByteSize, DiskHealth, DiskIO, DiskSpace, Percentage, Temperature, Transfer};
    pub use crate::cpu::CPU;
    pub use crate::disk::{
        Disk, DiskConfig, DiskHealthMonitor, DiskIOMonitor, DiskMount, DiskMountMonitor, DiskPerformance,
        DiskPerformanceMonitor, DiskStorageMonitor, DiskType, DiskUtilizationMonitor,
    };
    // Result and Error types
    pub use crate::error::{Error, Result};
    pub use crate::gpu::{Gpu, GpuMemory, GpuUtilization};
    pub use crate::memory::{Memory, MemoryInfo, PageStates, PressureLevel, SwapUsage};
    pub use crate::network::{Interface, NetworkInfo, NetworkInterface};
    // Common monitoring traits
    pub use crate::traits::{
        ByteMetricsMonitor, CpuMonitor, GpuMonitor, HardwareMonitor, MemoryMonitor, NetworkInterfaceMonitor,
        RateMonitor, StorageMonitor, TemperatureMonitor, UtilizationMonitor,
    };
}

/// FFI bindings and C-compatible interfaces.
pub mod ffi {
    // Re-export FFI types for C interop
    pub use crate::utils::ffi::*;
}

// ===== Tests =====

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

// Type aliases for common name clashes
/// A type alias for the network manager to avoid name clashes
pub type NetworkManager = crate::network::interface::NetworkManager;

// Re-export System for examples
pub use crate::system::System;
