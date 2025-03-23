mod constants;
mod cpu_impl;
mod monitors;
mod types;

// Re-export all types and monitors
// Re-export constants
pub use constants::*;
pub use monitors::*;
pub use types::*;

use crate::core::metrics::Metric;
use crate::core::types::Percentage;
use crate::error::Result;
// Import IOKit from the hardware module
use crate::hardware::iokit::IOKit;
// Re-export core traits from the traits module
pub use crate::traits::{CpuMonitor, HardwareMonitor, TemperatureMonitor, UtilizationMonitor};
use std::sync::Arc;

/// # CPU Module
///
/// This module provides functionality for monitoring CPU metrics on macOS systems.
/// It includes traits and types for collecting CPU usage, temperature, and frequency data.
///
/// ## Features
///
/// * CPU core usage monitoring
/// * CPU temperature tracking
/// * CPU frequency measurement
/// * Support for multiple CPU cores
///
/// ## Example
///
/// ```rust
/// use darwin_metrics::cpu::{CpuMetrics, CpuMetricsData};
///
/// struct MyCpuMonitor;
///
/// impl CpuMetrics for MyCpuMonitor {
///     fn get_core_usage(&self) -> f64 { 50.0 }
///     fn get_cpu_temperature(&self) -> Option<f64> { Some(45.0) }
///     fn get_cpu_frequency(&self) -> f64 { 2400.0 }
/// }
/// ```

/// A trait for collecting CPU metrics on macOS systems.
///
/// This trait defines the interface for monitoring various CPU metrics including
/// core usage, temperature, and frequency. Implementations should provide accurate
/// readings from the system's hardware sensors and performance counters.
pub trait CpuMetrics {
    /// Gets the current CPU core usage as a percentage (0.0 to 100.0).
    ///
    /// This method returns the average CPU usage across all cores.
    fn get_core_usage(&self) -> f64;

    /// Gets the current CPU temperature in degrees Celsius.
    ///
    /// Returns `None` if the temperature sensor is not available or reading fails.
    fn get_cpu_temperature(&self) -> Option<f64>;

    /// Gets the current CPU frequency in MHz.
    ///
    /// This method returns the current operating frequency of the CPU.
    fn get_cpu_frequency(&self) -> f64;
}

/// A data structure containing CPU metrics at a point in time.
///
/// This struct holds a snapshot of CPU metrics including usage, temperature,
/// and frequency measurements.
pub struct CpuMetricsData {
    /// The CPU usage as a percentage (0.0 to 100.0)
    pub usage: f64,
    /// The CPU temperature in degrees Celsius (if available)
    pub temperature: Option<f64>,
    /// The CPU frequency in MHz
    pub frequency: f64,
}

/// Maximum number of CPU cores supported by the library
pub const MAX_CORES: u32 = 64;

/// Maximum CPU frequency in MHz that can be reported
pub const MAX_FREQUENCY_MHZ: f64 = 5000.0;

/// Configuration for CPU monitoring
#[derive(Debug, Clone)]
pub struct CpuConfig {
    /// Threshold for CPU temperature (in Celsius)
    pub temperature_threshold: f64,
    /// Threshold for CPU utilization (0-100)
    pub utilization_threshold: f64,
}

impl Default for CpuConfig {
    fn default() -> Self {
        Self {
            temperature_threshold: 90.0,
            utilization_threshold: 90.0,
        }
    }
}

/// CPU monitor implementation
///
/// This struct provides access to CPU monitoring capabilities through separate monitor instances
/// for temperature, utilization, and frequency metrics.
///
/// # Example
///
/// ```rust
/// use darwin_metrics::cpu::{CPU};
/// use darwin_metrics::hardware::iokit::IOKitImpl;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let iokit = Box::new(IOKitImpl::new()?);
///     let cpu = CPU::new(iokit);
///     
///     // Get temperature monitor
///     let temp_monitor = cpu.temperature_monitor();
///     let temp = temp_monitor.temperature().await?;
///     println!("CPU Temperature: {:.1}°C", temp);
///     
///     // Get utilization monitor
///     let util_monitor = cpu.utilization_monitor();
///     let usage = util_monitor.utilization().await?;
///     println!("CPU Usage: {:.1}%", usage);
///     
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct CPU {
    /// IOKit interface for hardware communication
    iokit: Box<dyn IOKit>,
}

impl CPU {
    /// Creates a new CPU monitor
    pub fn new(iokit: Arc<dyn IOKit>) -> Self {
        Self {
            iokit,
            config: CpuConfig::default(),
        }
    }

    /// Get a monitor for CPU temperature metrics
    ///
    /// Returns a monitor that implements both `HardwareMonitor` and `TemperatureMonitor` traits,
    /// providing access to CPU temperature readings.
    pub fn temperature_monitor(&self) -> CpuTemperatureMonitor {
        CpuTemperatureMonitor::new(self.clone(), "cpu0".to_string())
    }

    /// Get a monitor for CPU utilization metrics
    ///
    /// Returns a monitor that implements both `HardwareMonitor` and `UtilizationMonitor` traits,
    /// providing access to CPU usage readings.
    pub fn utilization_monitor(&self) -> CpuUtilizationMonitor {
        CpuUtilizationMonitor::new(self.clone(), "cpu0".to_string())
    }

    /// Get a monitor for CPU frequency metrics
    ///
    /// Returns a monitor that provides access to CPU frequency readings.
    pub fn frequency_monitor(&self) -> CpuFrequencyMonitor {
        CpuFrequencyMonitor::new(self.clone(), "cpu0".to_string())
    }

    /// Clone method that creates a new CPU instance with a cloned IOKit box
    pub fn clone(&self) -> Self {
        Self {
            iokit: self.iokit.clone_box(),
            config: self.config.clone(),
        }
    }

    /// Get the number of CPU cores
    pub async fn core_count(&self) -> Result<u32> {
        // This is a placeholder implementation
        // In a real implementation, you would get the actual core count
        //! TODO: Implement this
        Ok(4) // Default to 4 cores
    }

    /// Get CPU usage metrics
    async fn get_cpu_usage(&self) -> Result<Vec<f64>> {
        let usage = self.iokit.get_core_usage()?;
        Ok(usage)
    }

    /// Get CPU utilization metrics
    pub async fn utilization(&self) -> Result<Vec<f64>> {
        self.get_cpu_usage().await
    }

    /// Calculate average CPU utilization across all cores
    pub async fn average_utilization(&self) -> Result<f64> {
        let usage = self.utilization().await?;
        if usage.is_empty() {
            return Ok(0.0);
        }
        let sum: f64 = usage.iter().sum();
        Ok(sum / usage.len() as f64)
    }

    /// Get CPU utilization as a metric
    pub async fn utilization_metric(&self) -> Result<Metric<Percentage>> {
        let avg = self.average_utilization().await?;
        Ok(Metric::new(
            Percentage::new(avg).unwrap_or(Percentage::new(0.0).unwrap()),
        ))
    }

    /// Get CPU temperature
    pub fn temperature(&self) -> Option<f64> {
        self.iokit.get_cpu_temperature("IOService").ok()
    }

    /// Get CPU frequency in MHz
    pub fn frequency_mhz(&self) -> f64 {
        // Placeholder implementation
        2000.0
    }

    /// Get minimum CPU frequency in MHz
    pub fn min_frequency_mhz(&self) -> Option<f64> {
        // Placeholder implementation
        Some(1000.0)
    }

    /// Get maximum CPU frequency in MHz
    pub fn max_frequency_mhz(&self) -> Option<f64> {
        // Placeholder implementation
        Some(3000.0)
    }

    /// Get available CPU frequency steps in MHz
    pub fn available_frequencies(&self) -> Option<&[f64]> {
        // Placeholder implementation
        None
    }

    /// Get all CPU metrics in a single call
    pub fn metrics(&self) -> Result<CpuMetricsData> {
        let usage = self.iokit.get_core_usage()?.iter().sum::<f64>() / self.iokit.get_core_usage()?.len() as f64;

        let temperature = self.temperature();
        let frequency = self.frequency_mhz();

        Ok(CpuMetricsData {
            usage,
            temperature,
            frequency,
        })
    }
}

impl Clone for CPU {
    fn clone(&self) -> Self {
        Self {
            iokit: self.iokit.clone_box(),
        }
    }
}
