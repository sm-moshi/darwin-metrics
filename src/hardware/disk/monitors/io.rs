use async_trait::async_trait;

use crate::{
    core::metrics::{hardware::ByteMetricsMonitor, HardwareMonitor, Metric},
    error::{Error, Result},
    hardware::disk::types::{ByteMetrics, Disk},
};

/// Monitor for disk I/O metrics
#[derive(Debug)]
pub struct DiskIOMonitor {
    disk: Disk,
}

impl DiskIOMonitor {
    /// Creates a new DiskIOMonitor for the given disk
    pub fn new(disk: Disk) -> Self {
        Self { disk }
    }
}

#[async_trait]
impl HardwareMonitor for DiskIOMonitor {
    type MetricType = ByteMetrics;

    async fn name(&self) -> Result<String> {
        Ok("Disk I/O Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Disk".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(format!("disk_io_{}", self.disk.device))
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let bytes_read = Err(Error::NotImplemented { feature: "Disk bytes read retrieval".to_string() })?;
        let bytes_written = Err(Error::NotImplemented { feature: "Disk bytes written retrieval".to_string() })?;
        let total = self.total_bytes().await?;
        Ok(Metric::new(ByteMetrics { bytes_read, bytes_written, total_bytes: total }))
    }
}

#[async_trait::async_trait]
impl ByteMetricsMonitor for DiskIOMonitor {
    async fn total_bytes(&self) -> Result<u64> {
        // Implementation...
        Err(Error::NotImplemented { feature: "Disk total bytes retrieval".to_string() })
    }

    async fn used_bytes(&self) -> Result<u64> {
        // Implementation...
        Err(Error::NotImplemented { feature: "Disk used bytes retrieval".to_string() })
    }

    async fn free_bytes(&self) -> Result<u64> {
        // Implementation...
        Err(Error::NotImplemented { feature: "Disk free bytes retrieval".to_string() })
    }
}
