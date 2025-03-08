//! # CPU Module
//! 
//! The CPU module provides access to macOS CPU metrics including usage statistics,
//! temperature data, and frequency information.
//! 
//! This module interfaces with the macOS IOKit framework to retrieve detailed CPU
//! information from the AppleACPICPU service and other system sources. It offers
//! a simple API to monitor CPU performance metrics on macOS systems.
//! 
//! ## Features
//! 
//! - CPU usage statistics per core and aggregated
//! - Physical and logical core count detection
//! - CPU frequency monitoring (current, min, max)
//! - CPU temperature readings (when available)
//! - CPU model name identification
//! 
//! ## Example
//! 
//! ```rust
//! use darwin_metrics::hardware::cpu::CPU;
//! use darwin_metrics::hardware::cpu::CpuMetrics;
//! 
//! fn main() -> darwin_metrics::error::Result<()> {
//!     let cpu = CPU::new()?;
//!     
//!     // Get basic CPU information
//!     println!("CPU Model: {}", cpu.model_name());
//!     println!("Physical cores: {}", cpu.physical_cores());
//!     println!("Logical cores: {}", cpu.logical_cores());
//!     
//!     // Get current CPU metrics
//!     println!("CPU Usage: {:.2}%", cpu.get_cpu_usage() * 100.0);
//!     println!("CPU Frequency: {:.2} MHz", cpu.get_cpu_frequency());
//!     
//!     if let Some(temp) = cpu.get_cpu_temperature() {
//!         println!("CPU Temperature: {:.1}°C", temp);
//!     } else {
//!         println!("CPU Temperature: Not available");
//!     }
//!     
//!     // Access per-core usage
//!     for (i, usage) in cpu.core_usage().iter().enumerate() {
//!         println!("Core {}: {:.2}%", i, usage * 100.0);
//!     }
//!     
//!     Ok(())
//! }
//! ```

mod cpu;
mod frequency;

pub use cpu::CPU;
pub use frequency::FrequencyMetrics;

/// Maximum number of CPU cores supported by the library.
pub const MAX_CORES: u32 = 64;

/// Maximum CPU frequency in MHz supported by the library.
pub const MAX_FREQUENCY_MHZ: f64 = 5000.0;

/// Trait defining the standard interface for accessing CPU metrics.
///
/// This trait provides a consistent API for retrieving common CPU metrics
/// regardless of the underlying CPU architecture or implementation details.
pub trait CpuMetrics {
    /// Returns the average CPU usage across all cores as a value between 0.0 (0%) and 1.0 (100%).
    fn get_cpu_usage(&self) -> f64;
    
    /// Returns the CPU temperature in degrees Celsius, if available.
    ///
    /// On macOS, temperature readings may not be available on all hardware, particularly
    /// on older systems or in virtualized environments. Returns `None` if temperature
    /// data cannot be retrieved.
    fn get_cpu_temperature(&self) -> Option<f64>;
    
    /// Returns the current CPU frequency in MHz.
    fn get_cpu_frequency(&self) -> f64;
}
