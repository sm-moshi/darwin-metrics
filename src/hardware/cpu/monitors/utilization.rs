use crate::hardware::cpu::{CpuUtilization, CPU};
use crate::{
    core::metrics::hardware::{HardwareMonitor, UtilizationMonitor},
    core::metrics::Metric,
    core::types::Percentage,
    error::Result,
};
use async_trait::async_trait;

/// Monitor for CPU utilization metrics
pub struct CpuUtilizationMonitor {
    cpu: CPU,
    device_id: String,
    last_utilization: Option<CpuUtilization>,
}

impl CpuUtilizationMonitor {
    /// Creates a new CpuUtilizationMonitor with the provided CPU and device ID
    pub fn new(cpu: CPU, device_id: String) -> Self {
        Self { cpu, device_id, last_utilization: None }
    }
}

#[async_trait]
impl HardwareMonitor for CpuUtilizationMonitor {
    type MetricType = Percentage;

    async fn name(&self) -> Result<String> {
        Ok("CPU Utilization Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("CPU".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.device_id.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let percentage_value = self.utilization().await?;
        let percentage = Percentage::new(percentage_value).unwrap_or(Percentage::new(0.0).unwrap());
        Ok(Metric::new(percentage))
    }
}

#[async_trait]
impl UtilizationMonitor for CpuUtilizationMonitor {
    async fn utilization(&self) -> Result<f64> {
        self.cpu.average_utilization().await
    }
}
