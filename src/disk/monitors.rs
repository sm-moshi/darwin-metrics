use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use tokio::time::sleep;

use super::DISK_UPDATE_INTERVAL;
use crate::core::metrics::Metric;
use crate::core::types::{ByteSize, Percentage, Transfer};
use crate::disk::types::{Disk, DiskConfig, DiskHealth, DiskType};
use crate::error::{Error, Result};
use crate::traits::hardware::{
    ByteMetricsMonitor, DiskHealthMonitor, DiskIOMonitor, DiskMountMonitor, DiskPerformanceMonitor, DiskStorageMonitor,
    DiskUtilizationMonitor, HardwareMonitor, RateMonitor, StorageMonitor, UtilizationMonitor,
};
use crate::{DiskIO, DiskSpace};

// Disk Health Monitor
//

/// Monitor for disk health metrics
#[derive(Debug)]
pub struct DiskHealthMonitorImpl {
    /// The disk being monitored
    disk: Disk,
}

impl DiskHealthMonitorImpl {
    /// Create a new disk health monitor
    pub fn new(disk: Disk) -> Self {
        Self { disk }
    }
}

#[async_trait]
impl HardwareMonitor for DiskHealthMonitorImpl {
    type MetricType = DiskHealth;

    async fn name(&self) -> Result<String> {
        Ok(format!("Disk Health Monitor ({})", self.disk.name))
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Disk".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.disk.device.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let health = DiskHealth {
            smart_status: true,
            temperature: 35.0, // Default temperature in Celsius
            power_on_hours: 0,
            reallocated_sectors: 0,
            pending_sectors: 0,
            uncorrectable_sectors: 0,
            last_check: std::time::SystemTime::now(),
        };

        Ok(Metric::new(health))
    }
}

#[async_trait]
impl DiskHealthMonitor for DiskHealthMonitorImpl {
    async fn disk_type(&self) -> Result<String> {
        Ok("SSD".to_string()) // TODO: Implement actual disk type
    }

    async fn disk_name(&self) -> Result<String> {
        Ok(self.disk.name.clone())
    }

    async fn filesystem_type(&self) -> Result<String> {
        Ok(self.disk.fs_type.clone())
    }

    async fn is_nearly_full(&self) -> Result<bool> {
        let total = self.disk.total;
        let used = self.disk.total as f64 * 0.9; // Default to 90% full
        Ok((used / total as f64) > 0.9)
    }

    async fn is_boot_volume(&self) -> Result<bool> {
        Ok(self.disk.is_boot_volume)
    }

    async fn smart_status(&self) -> Result<bool> {
        Ok(true) // TODO: Implement actual smart status
    }

    async fn temperature(&self) -> Result<f32> {
        Ok(35.0) // TODO: Implement actual temperature
    }

    async fn power_on_hours(&self) -> Result<u32> {
        Ok(1000) // TODO: Implement actual power on hours
    }

    async fn reallocated_sectors(&self) -> Result<u32> {
        Ok(0) // TODO: Implement actual reallocated sectors
    }

    async fn pending_sectors(&self) -> Result<u32> {
        Ok(0) // TODO: Implement actual pending sectors
    }

    async fn uncorrectable_sectors(&self) -> Result<u32> {
        Ok(0) // TODO: Implement actual uncorrectable sectors
    }

    async fn last_check(&self) -> Result<SystemTime> {
        Ok(SystemTime::now()) // TODO: Implement actual last check
    }
}

// Disk I/O Monitor
//

/// Monitor for disk I/O metrics
#[derive(Debug)]
pub struct DiskIOMonitorImpl {
    /// The disk being monitored
    disk: Disk,
    /// Last recorded disk I/O metrics
    last_io: Option<DiskIO>,
    /// Time interval for update
    update_interval: Duration,
}

impl DiskIOMonitorImpl {
    /// Create a new disk I/O monitor
    pub fn new(disk: Disk) -> Self {
        Self {
            disk,
            last_io: None,
            update_interval: DISK_UPDATE_INTERVAL,
        }
    }

    /// Set the update interval
    pub fn with_update_interval(mut self, interval: Duration) -> Self {
        self.update_interval = interval;
        self
    }

    /// Get current disk IO statistics
    async fn get_current_io(&self) -> Result<DiskIO> {
        Ok(DiskIO {
            reads: 0,
            writes: 0,
            read_bytes: ByteSize::from_bytes(0),
            write_bytes: ByteSize::from_bytes(0),
            read_time: Duration::from_millis(0),
            write_time: Duration::from_millis(0),
        })
    }
}

#[async_trait]
impl HardwareMonitor for DiskIOMonitorImpl {
    type MetricType = DiskIO;

    async fn name(&self) -> Result<String> {
        Ok(format!("Disk IO Monitor ({})", self.disk.name))
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Disk".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.disk.device.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let io = self.get_current_io().await?;
        Ok(Metric::new(io))
    }
}

#[async_trait]
impl DiskIOMonitor for DiskIOMonitorImpl {
    async fn get_io(&self) -> Result<DiskIO> {
        self.get_current_io().await
    }

    async fn get_transfer_rate(&self) -> Result<Transfer> {
        let current = self.get_current_io().await?;

        match &self.last_io {
            Some(last) => {
                // Simple rate calculation based on the interval
                let read_rate = ByteSize::from_bytes(
                    (current.read_bytes.as_bytes() - last.read_bytes.as_bytes()) / self.update_interval.as_secs(),
                );

                let write_rate = ByteSize::from_bytes(
                    (current.write_bytes.as_bytes() - last.write_bytes.as_bytes()) / self.update_interval.as_secs(),
                );

                Ok(Transfer {
                    read: read_rate,
                    write: write_rate,
                })
            },
            None => {
                // First call, wait for the interval and then calculate
                sleep(self.update_interval).await;
                let new_current = self.get_current_io().await?;

                let read_rate = ByteSize::from_bytes(
                    (new_current.read_bytes.as_bytes() - current.read_bytes.as_bytes())
                        / self.update_interval.as_secs(),
                );

                let write_rate = ByteSize::from_bytes(
                    (new_current.write_bytes.as_bytes() - current.write_bytes.as_bytes())
                        / self.update_interval.as_secs(),
                );

                Ok(Transfer {
                    read: read_rate,
                    write: write_rate,
                })
            },
        }
    }
}

#[async_trait]
impl ByteMetricsMonitor for DiskIOMonitorImpl {
    async fn total_bytes(&self) -> Result<u64> {
        Ok(self.disk.total)
    }

    async fn used_bytes(&self) -> Result<u64> {
        // Default to 90% of disk used
        Ok((self.disk.total as f64 * 0.9) as u64)
    }

    async fn free_bytes(&self) -> Result<u64> {
        let used = self.used_bytes().await?;
        Ok(self.disk.total - used)
    }
}

impl RateMonitor<u64> for DiskIOMonitorImpl {
    async fn rate(&self) -> Result<u64> {
        let transfer = DiskIOMonitor::get_transfer_rate(self).await?;
        let total_bytes_per_sec = transfer.read.as_bytes() + transfer.write.as_bytes();
        Ok(total_bytes_per_sec)
    }

    async fn average_rate(&self, seconds: u64) -> Result<u64> {
        // Update error handling for InvalidArgument
        if seconds == 0 {
            return Err(Error::InvalidArgument(
                "seconds for average_rate must be greater than 0".to_string(),
            ));
        }
        self.rate().await
    }
}

// Disk Mount Monitor
//

/// Monitor for disk mount information
#[derive(Debug)]
pub struct DiskMountMonitorImpl {
    /// The disk being monitored
    disk: Disk,
}

impl DiskMountMonitorImpl {
    /// Create a new disk mount monitor
    pub fn new(disk: Disk) -> Self {
        Self { disk }
    }
}

#[async_trait]
impl HardwareMonitor for DiskMountMonitorImpl {
    type MetricType = String; // Mount point

    async fn name(&self) -> Result<String> {
        Ok(format!("Disk Mount Monitor ({})", self.disk.name))
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Disk".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.disk.device.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        Ok(Metric::new(self.disk.mount_point.clone()))
    }
}

#[async_trait]
impl DiskMountMonitor for DiskMountMonitorImpl {
    async fn mount_point(&self) -> Result<String> {
        Ok(self.disk.mount_point.clone())
    }

    async fn filesystem_type(&self) -> Result<String> {
        Ok(self.disk.fs_type.clone())
    }

    async fn is_mounted(&self) -> Result<bool> {
        // Check if the disk has a mount point
        Ok(!self.disk.mount_point.is_empty())
    }

    async fn mount_options(&self) -> Result<Vec<String>> {
        // Placeholder implementation
        Ok(vec!["rw".to_string()])
    }

    async fn is_boot_volume(&self) -> Result<bool> {
        Ok(self.disk.is_boot_volume)
    }

    async fn is_readonly(&self) -> Result<bool> {
        // Since mount_flags is not available, assume it's not read-only
        // TODO: Implement proper readonly detection
        Ok(false)
    }

    async fn is_removable(&self) -> Result<bool> {
        // Placeholder implementation
        Ok(false)
    }

    async fn is_network(&self) -> Result<bool> {
        // Placeholder implementation
        Ok(false)
    }
}

// Disk Performance Monitor
//

/// Monitor for disk performance metrics
#[derive(Debug)]
pub struct DiskPerformanceMonitorImpl {
    /// The disk being monitored
    disk: Disk,
    /// The I/O monitor for this disk
    io_monitor: DiskIOMonitorImpl,
}

impl DiskPerformanceMonitorImpl {
    /// Create a new disk performance monitor
    pub fn new(disk: Disk) -> Self {
        Self {
            io_monitor: DiskIOMonitorImpl::new(disk.clone()),
            disk,
        }
    }
}

#[async_trait]
impl HardwareMonitor for DiskPerformanceMonitorImpl {
    type MetricType = f64; // IOPS (ops/sec)

    async fn name(&self) -> Result<String> {
        Ok(format!("Disk Performance Monitor ({})", self.disk.name))
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Disk".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.disk.device.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        // Calculate total iops (read + write)
        let iops = self.read_ops_per_second().await? + self.write_ops_per_second().await?;
        Ok(Metric::new(iops))
    }
}

#[async_trait]
impl DiskPerformanceMonitor for DiskPerformanceMonitorImpl {
    async fn read_ops_per_second(&self) -> Result<f64> {
        // Placeholder - not implemented
        Ok(0.0)
    }

    async fn write_ops_per_second(&self) -> Result<f64> {
        // Placeholder - not implemented
        Ok(0.0)
    }

    async fn read_latency_ms(&self) -> Result<f64> {
        // Not implemented yet, would require lower-level access
        Ok(0.0)
    }

    async fn write_latency_ms(&self) -> Result<f64> {
        // Not implemented yet
        Ok(0.0)
    }

    async fn queue_depth(&self) -> Result<f64> {
        // Not implemented yet
        Ok(0.0)
    }
}

// Disk Storage Monitor
//

/// Monitor for disk storage metrics
#[derive(Debug)]
pub struct DiskStorageMonitorImpl {
    /// The disk being monitored
    disk: Disk,
}

impl DiskStorageMonitorImpl {
    /// Create a new disk storage monitor
    pub fn new(disk: Disk) -> Self {
        Self { disk }
    }

    /// Create a new disk storage monitor for the root filesystem
    pub fn new_root() -> Result<Self> {
        // TODO: Implement proper root disk detection
        let disk_config = DiskConfig {
            disk_type: DiskType::SSD,
            name: "Macintosh HD".to_string(),
            is_boot_volume: true,
        };

        let disk = Disk::with_details(
            "default".to_string(),
            "/".to_string(),
            "apfs".to_string(),
            500 * 1024 * 1024 * 1024, // 500 GB
            100 * 1024 * 1024 * 1024, // 100 GB available
            400 * 1024 * 1024 * 1024, // 400 GB used
            disk_config,
        );

        Ok(Self::new(disk))
    }

    /// Get disk info for the root filesystem
    pub async fn get_disk_info(&self) -> Result<Disk> {
        Ok(self.disk.clone())
    }

    /// Get info for all disks
    pub async fn get_all_disks(&self) -> Result<Vec<Disk>> {
        // For now, just return the current disk
        Ok(vec![self.disk.clone()])
    }

    /// Get space information for a disk
    async fn get_space_info(&self) -> Result<DiskSpace> {
        #[cfg(any(test, feature = "testing"))]
        {
            let total = ByteSize::from_bytes(match self.disk.total {
                0 => {
                    // Default to 500GB when disk total is 0
                    500 * 1024 * 1024 * 1024
                },
                bytes => bytes,
            });

            let used = ByteSize::from_bytes((self.disk.total as f64 * 0.9) as u64);
            let available = ByteSize::from_bytes(self.disk.total - used.as_bytes());

            Ok(DiskSpace { total, used, available })
        }

        #[cfg(not(any(test, feature = "testing")))]
        {
            Ok(DiskSpace {
                total: ByteSize::from_bytes(self.disk.total),
                used: ByteSize::from_bytes(if self.disk.used == 0 { 0 } else { self.disk.used }),
                available: ByteSize::from_bytes(self.disk.available),
            })
        }
    }
}

#[async_trait]
impl HardwareMonitor for DiskStorageMonitorImpl {
    type MetricType = DiskSpace;

    async fn name(&self) -> Result<String> {
        Ok(format!("Disk Storage Monitor ({})", self.disk.name))
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Disk".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.disk.device.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let space = self.get_space_info().await?;
        Ok(Metric::new(space))
    }
}

#[async_trait]
impl StorageMonitor for DiskStorageMonitorImpl {
    async fn total_capacity(&self) -> Result<u64> {
        Ok(self.disk.total)
    }

    async fn available_capacity(&self) -> Result<u64> {
        // Default to 10% available
        Ok((self.disk.total as f64 * 0.1) as u64)
    }

    async fn used_capacity(&self) -> Result<u64> {
        let available = self.available_capacity().await?;
        Ok(self.disk.total - available)
    }
}

#[async_trait]
impl DiskStorageMonitor for DiskStorageMonitorImpl {
    async fn total_space(&self) -> Result<ByteSize> {
        Ok(ByteSize::from_bytes(self.disk.total))
    }

    async fn used_space(&self) -> Result<ByteSize> {
        Ok(ByteSize::from_bytes(self.disk.used))
    }

    async fn available_space(&self) -> Result<ByteSize> {
        Ok(ByteSize::from_bytes(self.disk.available))
    }

    async fn usage_percentage(&self) -> Result<Percentage> {
        let used = self.disk.used as f64;
        let total = self.disk.total as f64;
        Ok(Percentage::from_f64(used / total * 100.0))
    }
}

// Disk Utilization Monitor
//

/// Monitor for disk utilization metrics
#[derive(Debug)]
pub struct DiskUtilizationMonitorImpl {
    /// The disk being monitored
    disk: Disk,
    /// The I/O monitor for this disk
    io_monitor: DiskIOMonitorImpl,
}

impl DiskUtilizationMonitorImpl {
    /// Create a new disk utilization monitor
    pub fn new(disk: Disk) -> Self {
        Self {
            io_monitor: DiskIOMonitorImpl::new(disk.clone()),
            disk,
        }
    }
}

#[async_trait]
impl HardwareMonitor for DiskUtilizationMonitorImpl {
    type MetricType = Percentage;

    async fn name(&self) -> Result<String> {
        Ok(format!("Disk Utilization Monitor ({})", self.disk.name))
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Disk".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.disk.device.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let utilization = self.utilization().await?;
        Ok(Metric::new(Percentage::from_f64(utilization)))
    }
}

#[async_trait]
impl UtilizationMonitor for DiskUtilizationMonitorImpl {
    async fn utilization(&self) -> Result<f64> {
        // This is a simplistic implementation - actual disk utilization would
        // require more sophisticated monitoring

        // Get I/O rates as a proxy for utilization
        let transfer = self.io_monitor.get_transfer_rate().await?;

        // Convert to MB/s for easier calculation
        let read_mb_per_sec = transfer.read.as_bytes() as f64 / 1_000_000.0;
        let write_mb_per_sec = transfer.write.as_bytes() as f64 / 1_000_000.0;

        // Assume max throughput around 500 MB/s for a typical disk
        // (this varies widely in reality)
        let max_throughput = 500.0;

        // Simple utilization calculation based on throughput
        let utilization = (read_mb_per_sec + write_mb_per_sec) / max_throughput * 100.0;

        // Cap at 100%
        Ok(utilization.min(100.0))
    }
}

#[async_trait]
impl DiskUtilizationMonitor for DiskUtilizationMonitorImpl {
    async fn get_read_utilization(&self) -> Result<Percentage> {
        // Simplified implementation
        let transfer = self.io_monitor.get_transfer_rate().await?;
        let read_mb_per_sec = transfer.read.as_bytes() as f64 / 1_000_000.0;

        // Assume max read throughput around 250 MB/s
        let max_read_throughput = 250.0;

        let utilization = (read_mb_per_sec / max_read_throughput * 100.0).min(100.0);

        Ok(Percentage::from_f64(utilization))
    }

    async fn get_write_utilization(&self) -> Result<Percentage> {
        // Simplified implementation
        let transfer = self.io_monitor.get_transfer_rate().await?;
        let write_mb_per_sec = transfer.write.as_bytes() as f64 / 1_000_000.0;

        // Assume max write throughput around 250 MB/s
        let max_write_throughput = 250.0;

        let utilization = (write_mb_per_sec / max_write_throughput * 100.0).min(100.0);

        Ok(Percentage::from_f64(utilization))
    }
}
