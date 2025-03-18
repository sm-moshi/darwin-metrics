//! This module provides disk metrics for macOS.
//!
//! It includes monitors for disk IO, mount points, storage, utilization, and health.

mod constants;
mod types;
pub use constants::*;
pub use types::*;

mod disk_impl;
pub use disk_impl::*;

// Consolidated monitors
mod monitors;
pub use monitors::*;

// Re-exports for disk-specific traits
pub use crate::traits::hardware::{
    DiskHealthMonitor, DiskIOMonitor, DiskMountMonitor, DiskPerformanceMonitor, DiskStorageMonitor,
    DiskUtilizationMonitor,
};

// Re-export core traits from the traits module
pub use crate::traits::{ByteMetricsMonitor, HardwareMonitor, StorageMonitor, UtilizationMonitor};

// Import IOKit for disk monitoring
use crate::error::Result;

/// Get information about the root filesystem
///
/// Returns details about the root filesystem including total, available, and used space.
pub async fn get_root_disk() -> Result<Disk> {
    #[cfg(any(test, feature = "testing"))]
    {
        Disk::get_info()
    }

    #[cfg(not(any(test, feature = "testing")))]
    {
        // Create a temporary monitor and call the instance method
        let monitor = monitors::DiskStorageMonitorImpl::new_root()?;
        monitor.get_disk_info().await
    }
}

/// Get information about all mounted filesystems
///
/// Returns a list of all mounted filesystems including total, available, and used space for each.
pub async fn get_all_disks() -> Result<Vec<Disk>> {
    #[cfg(any(test, feature = "testing"))]
    {
        Disk::get_all()
    }

    #[cfg(not(any(test, feature = "testing")))]
    {
        // Create a temporary monitor and call the instance method
        let monitor = monitors::DiskStorageMonitorImpl::new_root()?;
        monitor.get_all_disks().await
    }
}
