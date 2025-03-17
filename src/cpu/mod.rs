mod constants;
mod monitors;
mod types;
mod cpu_impl;

// Re-export all types and monitors
pub use monitors::*;
pub use types::*;
pub use cpu_impl::*;

// Re-export constants
pub use constants::*;

// Re-export core traits from the traits module
pub use crate::traits::{CpuMonitor, HardwareMonitor, TemperatureMonitor, UtilizationMonitor};

// Import IOKit from the hardware module
use crate::hardware::iokit::IOKit;
use crate::{
    core::metrics::Metric,
    core::types::{Percentage, Temperature},
    error::Result,
};

use async_trait::async_trait;

pub trait CpuMetrics {
    fn get_core_usage(&self) -> f64;
    fn get_cpu_temperature(&self) -> Option<f64>;
    fn get_cpu_frequency(&self) -> f64;
}

#[derive(Debug, Clone)]
pub struct CpuMetricsData {
    pub usage: f64,
    pub temperature: Option<f64>,
    pub frequency: f64,
}

pub const MAX_CORES: u32 = 64;
pub const MAX_FREQUENCY_MHZ: f64 = 5000.0;

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
    iokit: Box<dyn IOKit>,
}

impl CPU {
    /// Create a new CPU monitor with the provided IOKit implementation
    pub fn new(iokit: Box<dyn IOKit>) -> Self {
        Self { iokit }
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
        Ok(Metric::new(Percentage::new(avg).unwrap_or(Percentage::new(0.0).unwrap())))
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
        let usage = self.iokit.get_core_usage()?
            .iter()
            .sum::<f64>() / 
            self.iokit.get_core_usage()?.len() as f64;
        
        let temperature = self.temperature();
        let frequency = self.frequency_mhz();
        
        Ok(CpuMetricsData {
            usage,
            temperature,
            frequency,
        })
    }
} 