mod constants;
mod cpu_impl;
mod monitors;
mod types;

pub use monitors::*;
pub use types::*;

// Re-export core types and monitors
pub use crate::core::metrics::hardware::{HardwareMonitor, TemperatureMonitor, UtilizationMonitor};

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
/// for temperature and utilization metrics.
///
/// # Example
///
/// ```rust
/// use darwin_metrics::hardware::{CPU, IOKitImpl};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let iokit = Box::new(IOKitImpl::new());
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
    pub fn temperature_monitor(&self) -> CPUTemperatureMonitor {
        CPUTemperatureMonitor { cpu: self }
    }

    /// Get a monitor for CPU utilization metrics
    ///
    /// Returns a monitor that implements both `HardwareMonitor` and `UtilizationMonitor` traits,
    /// providing access to CPU usage readings.
    pub fn utilization_monitor(&self) -> CPUUtilizationMonitor {
        CPUUtilizationMonitor { cpu: self }
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
        // This is a placeholder implementation
        // In a real implementation, you would get the actual CPU usage
        //! TODO: Implement this
        let cores = self.core_count().await?;
        let mut usage = Vec::with_capacity(cores as usize);
        for _ in 0..cores {
            usage.push(0.0); // Placeholder value
        }
        Ok(usage)
    }

    /// Get CPU utilization metrics
    pub async fn utilization(&self) -> Result<Vec<f64>> {
        let usage = self.get_cpu_usage().await?;
        Ok(usage)
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
}

/// CPU temperature monitor
///
/// Provides access to CPU temperature metrics through the `HardwareMonitor` and `TemperatureMonitor` traits.
/// Temperature readings are obtained from the system's IOKit interface.
pub struct CPUTemperatureMonitor<'a> {
    cpu: &'a CPU,
}

#[async_trait]
impl HardwareMonitor for CPUTemperatureMonitor<'_> {
    type MetricType = Temperature;

    async fn name(&self) -> Result<String> {
        Ok("CPU Temperature Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("CPU".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("cpu0".to_string())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let temp = self.cpu.iokit.get_cpu_temperature("IOService")?;
        Ok(Metric::new(Temperature(temp)))
    }
}

#[async_trait]
impl TemperatureMonitor for CPUTemperatureMonitor<'_> {}

/// CPU utilization monitor
///
/// Provides access to CPU utilization metrics through the `HardwareMonitor` and `UtilizationMonitor` traits.
/// Utilization readings are obtained from the system's IOKit interface.
pub struct CPUUtilizationMonitor<'a> {
    cpu: &'a CPU,
}

#[async_trait]
impl HardwareMonitor for CPUUtilizationMonitor<'_> {
    type MetricType = Percentage;

    async fn name(&self) -> Result<String> {
        Ok("CPU Utilization Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("CPU".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("cpu0".to_string())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let usage = self.cpu.iokit.get_core_usage()?;

        // Calculate average utilization if it's a vector
        let avg_usage = if usage.is_empty() {
            0.0
        } else {
            let sum: f64 = usage.iter().sum();
            sum / usage.len() as f64
        };

        Ok(Metric::new(Percentage::new(avg_usage).unwrap_or(Percentage::new(0.0).unwrap())))
    }
}

#[async_trait]
impl UtilizationMonitor for CPUUtilizationMonitor<'_> {}
