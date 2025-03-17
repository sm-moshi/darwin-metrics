mod frequency;
mod temperature;
mod utilization;

pub use frequency::*;
pub use temperature::*;
pub use utilization::*;

use crate::core::metrics::{HardwareMonitor, Metric, TemperatureMonitor, UtilizationMonitor};
use crate::core::types::{Percentage, Temperature};
use crate::error::Result;
use crate::hardware::iokit::IOKit;
use async_trait::async_trait;

/// CPU temperature monitor
///
/// Provides access to CPU temperature metrics through the `HardwareMonitor` and `TemperatureMonitor` traits.
/// Temperature readings are obtained from the system's IOKit interface.
pub struct CPUTemperatureMonitor<'a> {
    iokit: &'a Box<dyn IOKit>,
    device_id: String,
}

impl<'a> CPUTemperatureMonitor<'a> {
    pub fn new(iokit: &'a Box<dyn IOKit>, device_id: String) -> Self {
        Self { iokit, device_id }
    }
}

#[async_trait]
impl HardwareMonitor for CPUTemperatureMonitor<'_> {
    type MetricType = Temperature;

    async fn name(&self) -> Result<String> {
        Ok("CPU Temperature".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("CPU".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.device_id.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let temp = self.iokit.get_cpu_temperature("IOService")?;
        Ok(Metric::new(Temperature::new_celsius(temp)))
    }
}

#[async_trait]
impl TemperatureMonitor for CPUTemperatureMonitor<'_> {}

/// CPU utilization monitor
///
/// Provides access to CPU utilization metrics through the `HardwareMonitor` and `UtilizationMonitor` traits.
/// Utilization readings are obtained from the system's IOKit interface.
pub struct CPUUtilizationMonitor<'a> {
    iokit: &'a Box<dyn IOKit>,
    device_id: String,
}

impl<'a> CPUUtilizationMonitor<'a> {
    pub fn new(iokit: &'a Box<dyn IOKit>, device_id: String) -> Self {
        Self { iokit, device_id }
    }
}

#[async_trait]
impl HardwareMonitor for CPUUtilizationMonitor<'_> {
    type MetricType = Percentage;

    async fn name(&self) -> Result<String> {
        Ok("CPU Utilization".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("CPU".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.device_id.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let utilization = self.iokit.get_core_usage()?;

        // Calculate average utilization if it's a vector
        let avg_utilization = if utilization.is_empty() {
            0.0
        } else {
            let sum: f64 = utilization.iter().sum();
            sum / utilization.len() as f64
        };

        Ok(Metric::new(Percentage::new(avg_utilization).unwrap_or(Percentage::new(0.0).unwrap())))
    }
}

#[async_trait]
impl UtilizationMonitor for CPUUtilizationMonitor<'_> {}
