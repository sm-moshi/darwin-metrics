//! Darwin Metrics - A Rust library for collecting macOS system metrics
//!
//! This crate provides a safe interface to various system metrics on macOS systems.
//! It uses direct FFI calls to system frameworks like IOKit, Metal, and CoreFoundation
//! to collect accurate system information with minimal overhead.
//!
//! # Features
//!
//! - **Battery Metrics**: Battery status, health, and power information
//! - **CPU Metrics**: CPU usage, frequency, and temperature
//! - **GPU Metrics**: GPU utilization, memory usage, and temperature via Metal framework
//! - **Memory Metrics**: System memory usage and pressure levels
//! - **Disk Metrics**: Storage space and I/O statistics
//! - **Temperature Sensors**: System temperature monitoring
//! - **Network Metrics**: Network interface statistics
//! - **Process Metrics**: Process resource usage tracking
//!
//! # Examples
//!
//! ```rust
//! use darwin_metrics::prelude::*;
//!
//! fn main() -> Result<()> {
//!     // Get battery information
//!     let battery = Battery::new()?;
//!     println!("Battery: {}%, {}", battery.percentage,
//!         if battery.is_charging { "Charging" } else { "Discharging" });
//!     
//!     // Get GPU metrics
//!     let gpu = GPU::new()?;
//!     let gpu_metrics = gpu.get_metrics()?;
//!     println!("GPU: {} - Utilization: {}%", gpu_metrics.name, gpu_metrics.utilization);
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Safety
//!
//! This crate makes extensive use of unsafe FFI calls to macOS system frameworks.
//! All unsafe operations are properly wrapped in safe abstractions and follow these principles:
//!
//! - Proper resource cleanup using Drop implementations
//! - Null pointer and error checking for all FFI calls
//! - Safe wrapping of unsafe system calls
//! - Thread-safe access to system resources
//!
//! # Error Handling
//!
//! The crate uses a custom Error type that covers various failure modes:
//!
//! ```rust
//! use darwin_metrics::{Error, Result};
//!
//! fn example() -> Result<()> {
//!     // Service not found error
//!     if true {
//!         return Err(Error::ServiceNotFound);
//!     }
//!     
//!     // Feature not available error
//!     if true {
//!         return Err(Error::NotAvailable("GPU metrics not available".to_string()));
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Thread Safety
//!
//! All public types in this crate implement Send and Sync where appropriate,
//! allowing safe usage in multi-threaded applications. Resource cleanup is
//! handled automatically through Drop implementations.

use thiserror::Error;

/// Error type for darwin-metrics operations
#[derive(Debug, Error)]
pub enum Error {
    #[error("Service not found")]
    ServiceNotFound,
    #[error("Feature not available: {0}")]
    NotAvailable(String),
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
    #[error("System error: {0}")]
    SystemError(String),
}

impl Error {
    pub(crate) fn not_available(msg: impl Into<String>) -> Self {
        Error::NotAvailable(msg.into())
    }

    pub(crate) fn not_implemented(msg: impl Into<String>) -> Self {
        Error::NotImplemented(msg.into())
    }

    #[allow(dead_code)]
    pub(crate) fn system_error(msg: impl Into<String>) -> Self {
        Error::SystemError(msg.into())
    }
}

/// Result type for darwin-metrics operations
pub type Result<T> = std::result::Result<T, Error>;

// Public modules
pub mod battery;
pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod iokit;
pub mod memory;
pub mod temperature;

// Private modules
mod utils;

/// Re-export common types for convenience
pub mod prelude {
    pub use crate::Error;
    pub use crate::Result;
    pub use crate::battery::Battery;
    pub use crate::cpu::CPU;
    pub use crate::disk::Disk;
    pub use crate::gpu::{GPU, GPUMemoryInfo, GPUMetrics};
    pub use crate::memory::Memory;
    pub use crate::temperature::Temperature;
    // Add other types as they're implemented
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_metrics() -> Result<()> {
        let gpu = gpu::GPU::new()?;
        let metrics = gpu.get_metrics()?;
        assert!(!metrics.name.is_empty());
        Ok(())
    }

    #[test]
    fn test_battery_metrics() -> Result<()> {
        let battery = battery::Battery::new()?;
        assert!(battery.percentage >= 0.0 && battery.percentage <= 100.0);
        Ok(())
    }
}
