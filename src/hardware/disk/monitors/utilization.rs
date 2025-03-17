use async_trait::async_trait;

use crate::{
    core::{
        metrics::{hardware::UtilizationMonitor, HardwareMonitor, Metric},
        types::Percentage,
    },
    error::Result,
    hardware::disk::types::Disk,
};

/// Monitor for disk utilization metrics
#[derive(Debug)]
pub struct DiskUtilizationMonitor {
    disk: Disk,
}

impl DiskUtilizationMonitor {
    /// Creates a new DiskUtilizationMonitor for the given disk
    pub fn new(disk: Disk) -> Self {
        Self { disk }
    }
}

#[async_trait]
impl HardwareMonitor for DiskUtilizationMonitor {
    type MetricType = Percentage;

    async fn name(&self) -> Result<String> {
        Ok("Disk Utilization Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Disk".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(format!("disk_utilization_{}", self.disk.device))
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let percentage = self.utilization().await?;
        Ok(Metric::new(Percentage::from_f64(percentage)))
    }
}

#[async_trait]
impl UtilizationMonitor for DiskUtilizationMonitor {
    async fn utilization(&self) -> Result<f64> {
        // Calculate disk utilization based on I/O activity
        let total_space = self.disk.total as f64;
        let used_space = self.disk.used as f64;
        let utilization = (used_space / total_space) * 100.0;
        Ok(utilization)
    }
}
