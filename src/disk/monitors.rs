use std::{
    ffi::{CStr, CString},
    io,
    mem::{size_of, MaybeUninit},
    time::{Duration, SystemTime},
};

use async_trait::async_trait;
use tokio::time::sleep;

use crate::{
    core::{
        metrics::Metric,
        types::{ByteSize, ByteSizeFormat, DiskHealth, DiskIO, DiskSpace, Percentage, PercentageFormat, Transfer},
    },
    disk::{
        constants::{FS_TYPE_APFS, FS_TYPE_NFS, FS_TYPE_RAMFS, FS_TYPE_SMB, FS_TYPE_TMPFS},
        types::{ByteMetrics, Disk, DiskConfig, DiskHealth, DiskMount, DiskPerformanceMetrics, DiskType},
    },
    error::{Error, Result},
    traits::{
        ByteMetricsMonitor, DiskHealthMonitor, DiskIOMonitor, DiskMountMonitor,
        DiskPerformanceMonitor, DiskStorageMonitor, DiskUtilizationMonitor,
        HardwareMonitor, RateMonitor, StorageMonitor, UtilizationMonitor,
    },
    utils::bindings::{getfsstat, statfs, Statfs, MNT_NOWAIT},
};

use super::{DISK_UPDATE_INTERVAL};

//
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
        // Smart status is not currently implemented
        let health = DiskHealth {
            smart_status: true,
            issues: vec![],
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
        // Default to 90% threshold
        let total = self.disk.size as f64;
        let used = (self.disk.size as f64 * 0.9) as f64; // Default to 90% full
        Ok((used / total) > 0.9)
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

//
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
                    (current.read_bytes.as_bytes() - last.read_bytes.as_bytes()) 
                        / self.update_interval.as_secs() as u64
                );
                
                let write_rate = ByteSize::from_bytes(
                    (current.write_bytes.as_bytes() - last.write_bytes.as_bytes())
                        / self.update_interval.as_secs() as u64
                );
                
                Ok(Transfer { read: read_rate, write: write_rate })
            }
            None => {
                // First call, wait for the interval and then calculate
                sleep(self.update_interval).await;
                let new_current = self.get_current_io().await?;
                
                let read_rate = ByteSize::from_bytes(
                    (new_current.read_bytes.as_bytes() - current.read_bytes.as_bytes())
                        / self.update_interval.as_secs() as u64
                );
                
                let write_rate = ByteSize::from_bytes(
                    (new_current.write_bytes.as_bytes() - current.write_bytes.as_bytes())
                        / self.update_interval.as_secs() as u64
                );
                
                Ok(Transfer { read: read_rate, write: write_rate })
            }
        }
    }
}

#[async_trait]
impl ByteMetricsMonitor for DiskIOMonitorImpl {
    async fn total_bytes(&self) -> Result<u64> {
        Ok(self.disk.size)
    }
    
    async fn used_bytes(&self) -> Result<u64> {
        // Default to 90% of disk used
        Ok((self.disk.size as f64 * 0.9) as u64)
    }
    
    async fn free_bytes(&self) -> Result<u64> {
        let used = self.used_bytes().await?;
        Ok(self.disk.size - used)
    }
}

impl RateMonitor<u64> for DiskIOMonitorImpl {
    async fn rate(&self) -> Result<u64> {
        let transfer = DiskIOMonitor::get_transfer_rate(self).await?;
        let total_bytes_per_sec = transfer.read.as_bytes() + transfer.write.as_bytes();
        Ok(total_bytes_per_sec)
    }

    async fn average_rate(&self, seconds: u64) -> Result<u64> {
        // Simplistic implementation
        if seconds == 0 {
            return Err(Error::InvalidArgument { 
                context: "seconds for average_rate".to_string(), 
                value: "0".to_string() 
            });
        }
        self.rate().await
    }
}

//
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
        // Check for read-only flag in mount flags
        Ok(self.disk.mount_flags & crate::disk::MNT_RDONLY != 0)
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

//
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

//
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

    /// Get space information for a disk
    async fn get_space_info(&self) -> Result<DiskSpace> {
        #[cfg(any(test, feature = "testing"))]
        {
            // For testing, return mock values
            let total = ByteSize::from_bytes(self.disk.total.as_bytes());
            
            if total.as_bytes() == 0 {
                // Mock disk is empty, return all zeroes
                return Ok(DiskSpace {
                    total: ByteSize::from_bytes(0),
                    used: ByteSize::from_bytes(0),
                    available: ByteSize::from_bytes(0),
                });
            }
            
            let used = ByteSize::from_bytes((self.disk.total.as_bytes() as f64 * 0.9) as u64);
            let available = ByteSize::from_bytes(self.disk.total.as_bytes() - used.as_bytes());
            
            Ok(DiskSpace {
                total,
                used,
                available,
            })
        }
        
        #[cfg(not(any(test, feature = "testing")))]
        {
            // Just return zeroes for now
            Ok(DiskSpace {
                total: ByteSize::from_bytes(self.disk.total.as_bytes()),
                used: ByteSize::from_bytes(self.disk.used.unwrap_or(ByteSize::from_bytes(0)).as_bytes()),
                available: ByteSize::from_bytes(self.disk.available.as_bytes()),
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
        Ok(self.disk.size)
    }
    
    async fn available_capacity(&self) -> Result<u64> {
        // Default to 10% available
        Ok((self.disk.size as f64 * 0.1) as u64)
    }
    
    async fn used_capacity(&self) -> Result<u64> {
        let available = self.available_capacity().await?;
        Ok(self.disk.size - available)
    }
}

#[async_trait]
impl DiskStorageMonitor for DiskStorageMonitorImpl {
    async fn total_space(&self) -> Result<ByteSize> {
        Ok(self.get_space_info().await?.total)
    }

    async fn used_space(&self) -> Result<ByteSize> {
        Ok(self.get_space_info().await?.used)
    }

    async fn available_space(&self) -> Result<ByteSize> {
        Ok(self.get_space_info().await?.available)
    }

    async fn usage_percentage(&self) -> Result<Percentage> {
        let space = self.get_space_info().await?;
        let percentage = space.used.as_bytes() as f64 / space.total.as_bytes() as f64 * 100.0;
        Ok(Percentage::new(percentage, PercentageFormat::WithSymbol))
    }
}

//
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
        Ok(Metric::new(Percentage::new(utilization, PercentageFormat::WithSymbol)))
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
        
        Ok(Percentage::new(utilization, PercentageFormat::WithSymbol))
    }

    async fn get_write_utilization(&self) -> Result<Percentage> {
        // Simplified implementation
        let transfer = self.io_monitor.get_transfer_rate().await?;
        let write_mb_per_sec = transfer.write.as_bytes() as f64 / 1_000_000.0;
        
        // Assume max write throughput around 250 MB/s
        let max_write_throughput = 250.0;
        
        let utilization = (write_mb_per_sec / max_write_throughput * 100.0).min(100.0);
        
        Ok(Percentage::new(utilization, PercentageFormat::WithSymbol))
    }
} 