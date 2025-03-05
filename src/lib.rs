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
    
    pub(crate) fn invalid_value(msg: impl Into<String>) -> Self {
        Error::SystemError(msg.into())
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
pub mod utils;
#[cfg(test)]
pub(crate) mod testing {
    use crate::battery::{Battery, PowerSource};
    use crate::cpu::CPU;
    use crate::iokit::MockIOKit;
    use objc2::runtime::AnyObject;
    use objc2::rc::Retained;
    use objc2::{msg_send, class};
    
    /// Creates a safe test dictionary for tests
    pub(crate) fn create_safe_dictionary() -> Retained<objc2_foundation::NSDictionary<objc2_foundation::NSString, objc2_foundation::NSObject>> {
        unsafe {
            let dict: *mut AnyObject = msg_send![class!(NSDictionary), dictionary];
            Retained::from_raw(dict.cast()).unwrap()
        }
    }

    /// Creates a safe test object for tests
    pub(crate) fn create_safe_object() -> Retained<AnyObject> {
        unsafe {
            let obj: *mut AnyObject = msg_send![class!(NSObject), new];
            Retained::from_raw(obj).unwrap()
        }
    }

    /// Creates a safe mock IOKit for tests
    pub(crate) fn create_safe_mock_iokit() -> MockIOKit {
        let mut mock = MockIOKit::new();
        mock.expect_io_service_matching()
            .returning(|_| create_safe_dictionary());
        mock.expect_io_service_get_matching_service()
            .returning(|_| None);
        mock
    }

    /// Creates a test battery with safe values
    pub(crate) fn create_test_battery() -> Battery {
        Battery::with_values(
            true,                 // is_present
            false,                // is_charging
            75.5,                 // percentage
            90,                   // time_remaining
            PowerSource::Battery, // power_source
            500,                 // cycle_count
            85.0,                // health_percentage
            35.0                 // temperature
        )
    }

    /// Creates a test CPU with safe values
    pub(crate) fn create_test_cpu() -> CPU {
        CPU {
            physical_cores: 4,
            logical_cores: 8,
            frequency_mhz: 2400.0,
            core_usage: vec![50.0, 75.0, 25.0, 100.0],
            model_name: "Test CPU".to_string(),
            temperature: Some(45.0),
            iokit: Box::new(create_safe_mock_iokit()),
        }
    }

    /// Sets up test environment for better diagnostics
    pub(crate) fn setup_test_environment() {
        std::panic::set_hook(Box::new(|panic_info| {
            eprintln!("Test panic: {}", panic_info);
            if let Some(location) = panic_info.location() {
                eprintln!("Panic occurred in file '{}' at line {}", location.file(), location.line());
            }
        }));
    }
}

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
    use std::panic;
    
    /// Setup crash handlers for tests
    fn setup_test_environment() {
        // Set up a panic hook to get better information on segfaults
        panic::set_hook(Box::new(|panic_info| {
            eprintln!("Test panic: {}", panic_info);
        }));
    }
    
    #[test]
    fn test_gpu_metrics() -> Result<()> {
        setup_test_environment();
        // Skip this test as it requires real hardware
        Ok(())
    }
    
    #[test]
    fn test_battery_metrics() -> Result<()> {
        setup_test_environment();
        // Create a battery with known test values
        let battery = battery::Battery::with_values(
            true, false, 75.5, 90,
            PowerSource::Battery, 500, 85.0, 35.0
        );
        assert!(battery.percentage >= 0.0 && battery.percentage <= 100.0);
        Ok(())
    }

    #[test]
    fn test_error_types() {
        fn get_service_not_found() -> Result<()> {
            Err(Error::ServiceNotFound)
        }
        
        assert!(matches!(get_service_not_found(), Err(Error::ServiceNotFound)));
        assert!(matches!(
            Error::not_available("test"), 
            Error::NotAvailable(_)
        ));
    }

    #[test]
    fn test_public_api() {
        use crate::battery::PowerSource;
        
        let battery = battery::Battery::with_values(
            true, false, 75.5, 90,
            PowerSource::Battery, 500, 85.0, 35.0
        );
        
        assert_eq!(battery.power_source, PowerSource::Battery);
        assert!(!battery.is_critical());
        assert_eq!(battery.time_remaining_display(), "1 hours 30 minutes");
    }
}
