use async_trait::async_trait;
use std::time::SystemTime;

use crate::{
    core::metrics::hardware::{DiskPerformanceMonitor as DiskPerformanceMonitorTrait, HardwareMonitor},
    core::metrics::Metric,
    core::types::ByteSize,
    error::Result,
    hardware::disk::types::{Disk, DiskPerformanceMetrics},
};

/// Monitor for disk performance metrics
#[derive(Debug)]
pub struct DiskPerformanceMonitor {
    disk: Disk,
}

impl DiskPerformanceMonitor {
    /// Creates a new DiskPerformanceMonitor for the given disk
    pub fn new(disk: Disk) -> Self {
        Self { disk }
    }
}

#[async_trait]
impl HardwareMonitor for DiskPerformanceMonitor {
    type MetricType = DiskPerformanceMetrics;

    async fn name(&self) -> Result<String> {
        Ok("Disk Performance".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Disk".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(format!("disk_performance_{}", self.disk.device))
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let metrics = DiskPerformanceMetrics {
            bytes_read: ByteSize::new(0), // TODO: Implement actual metrics collection
            bytes_written: ByteSize::new(0),
            read_ops: 0,
            write_ops: 0,
            read_latency_ms: self.read_latency_ms().await?,
            write_latency_ms: self.write_latency_ms().await?,
            timestamp: SystemTime::now(),
        };
        Ok(Metric::new(metrics))
    }
}

#[async_trait]
impl DiskPerformanceMonitorTrait for DiskPerformanceMonitor {
    async fn read_ops_per_second(&self) -> Result<f64> {
        // TODO: Implement actual performance metrics collection
        Ok(0.0)
    }

    async fn write_ops_per_second(&self) -> Result<f64> {
        // TODO: Implement actual performance metrics collection
        Ok(0.0)
    }

    async fn read_latency_ms(&self) -> Result<f64> {
        // TODO: Implement actual performance metrics collection
        Ok(0.0)
    }

    async fn write_latency_ms(&self) -> Result<f64> {
        // TODO: Implement actual performance metrics collection
        Ok(0.0)
    }

    async fn queue_depth(&self) -> Result<f64> {
        // TODO: Implement actual performance metrics collection
        Ok(0.0)
    }
}
