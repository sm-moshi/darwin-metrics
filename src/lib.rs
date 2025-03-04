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
//! use darwin_metrics::battery::PowerSource;
//! use darwin_metrics::gpu::{GPUMetrics, GPUMemoryInfo};
//!
//! fn main() -> Result<()> {
//!     // Create a battery with test values
//!     let battery = Battery::with_values(
//!         true,
//!         false,
//!         75.5,
//!         90,
//!         PowerSource::Battery,
//!         500,
//!         85.0,
//!         35.0,
//!     );
//!     println!("Battery: {}%, {}", battery.percentage,
//!         if battery.is_charging { "Charging" } else { "Discharging" });
//!     
//!     // Create a GPU with test values
//!     let metrics = GPUMetrics {
//!         utilization: 50.0,
//!         memory: GPUMemoryInfo {
//!             total: 4 * 1024 * 1024 * 1024,  // 4GB
//!             used: 2 * 1024 * 1024 * 1024,   // 2GB
//!             free: 2 * 1024 * 1024 * 1024,   // 2GB
//!         },
//!         temperature: Some(65.0),
//!         power_usage: Some(75.0),
//!         name: "Test GPU".to_string(),
//!     };
//!     println!("GPU: {} - Utilization: {}%", metrics.name, metrics.utilization);
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
pub mod memory;
pub mod network;
pub mod process;
pub mod temperature;
pub mod resource;

// Internal modules
pub mod iokit;
mod utils;

// Re-exports for convenience
pub use battery::Battery;
pub use cpu::CPU;
pub use disk::Disk;
pub use gpu::{GPU, GPUMetrics};
pub use memory::Memory;
pub use network::Network;
pub use process::Process;
pub use temperature::Temperature;
pub use resource::{ResourceManager, ResourcePool, Cache};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        Error,
        Result,
        Battery,
        CPU,
        Disk,
        GPU,
        GPUMetrics,
        Memory,
        Network,
        Process,
        Temperature,
        ResourceManager,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battery::PowerSource;
    use std::time::Duration;

    #[test]
    fn test_gpu_metrics() -> Result<()> {
        // Skip this test as it requires real hardware
        // We already have unit tests in the gpu module
        Ok(())
    }

    #[test]
    fn test_battery_metrics() -> Result<()> {
        // Create a battery with known test values instead of accessing hardware
        let battery = battery::Battery::with_values(
            true,
            false,
            75.5,
            90,
            PowerSource::Battery,
            500,
            85.0,
            35.0,
        );
        assert!(battery.percentage >= 0.0 && battery.percentage <= 100.0);
        Ok(())
    }
}
