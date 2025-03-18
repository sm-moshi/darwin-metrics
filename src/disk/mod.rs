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
    DiskHealthMonitor, DiskMountMonitor, DiskIOMonitor, 
    DiskPerformanceMonitor, DiskStorageMonitor, DiskUtilizationMonitor
};

// Re-export core traits from the traits module
pub use crate::traits::{
    ByteMetricsMonitor, HardwareMonitor, StorageMonitor, UtilizationMonitor,
};

// Import IOKit for disk monitoring
use crate::error::Result;

/// Get information about the current system disk
///
/// Returns information about the root filesystem (/) including total, available, and used space.
pub fn get_info() -> Result<Disk> {
    #[cfg(any(test, feature = "testing"))]
    {
        Disk::get_info()
    }
    
    #[cfg(not(any(test, feature = "testing")))]
    {
        crate::traits::hardware::DiskStorageMonitor::get_root_info()
    }
}

/// Get information about all mounted filesystems
///
/// Returns a list of all mounted filesystems including total, available, and used space for each.
pub fn get_all_disks() -> Result<Vec<Disk>> {
    #[cfg(any(test, feature = "testing"))]
    {
        Disk::get_all()
    }
    
    #[cfg(not(any(test, feature = "testing")))]
    {
        crate::traits::hardware::DiskStorageMonitor::get_all_disks()
    }
} 