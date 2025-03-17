use async_trait::async_trait;

use crate::{
    core::metrics::{
        hardware::{DiskMountMonitor as DiskMountMonitorTrait, HardwareMonitor},
        Metric,
    },
    error::Result,
    hardware::disk::types::DiskMount,
};

/// Monitor for disk mount metrics
pub struct DiskMountMonitor {
    device_id: String,
}

impl DiskMountMonitor {
    /// Creates a new DiskMountMonitor for the given disk
    pub fn new(device_id: String) -> Self {
        Self { device_id }
    }
}

#[async_trait]
impl HardwareMonitor for DiskMountMonitor {
    type MetricType = DiskMount;

    async fn name(&self) -> Result<String> {
        Ok("Disk Mount".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Disk".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.device_id.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let mount_info = DiskMount {
            mount_point: "/".to_string(),
            filesystem_type: "apfs".to_string(),
            is_boot_volume: true,
            is_readonly: false,
            is_removable: false,
            is_network: false,
        };

        Ok(Metric::new(mount_info))
    }
}

#[async_trait]
impl DiskMountMonitorTrait for DiskMountMonitor {
    async fn mount_point(&self) -> Result<String> {
        Ok("/".to_string())
    }

    async fn filesystem_type(&self) -> Result<String> {
        Ok("apfs".to_string())
    }

    async fn is_boot_volume(&self) -> Result<bool> {
        Ok(true)
    }

    async fn is_readonly(&self) -> Result<bool> {
        Ok(false)
    }

    async fn is_removable(&self) -> Result<bool> {
        Ok(false)
    }

    async fn is_network(&self) -> Result<bool> {
        Ok(false)
    }

    async fn is_mounted(&self) -> Result<bool> {
        Ok(true)
    }

    async fn mount_options(&self) -> Result<Vec<String>> {
        Ok(vec!["rw".to_string(), "noatime".to_string()])
    }
}
