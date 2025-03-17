//! Disk monitoring module
//!
//! This module provides disk metrics and monitoring for macOS systems.
//!
//! It includes functionality for gathering disk information, monitoring
//! disk performance, and tracking storage usage.

// Re-export disk types and constants
pub mod constants;
pub mod types;
mod disk_impl;

// Re-export monitors
pub mod monitors;
pub use monitors::*;

// Re-export types
pub use types::*;

// Re-export core traits from the traits module
pub use crate::traits::{
    ByteMetricsMonitor, DiskHealthMonitor, DiskMountMonitor, DiskPerformanceMonitor,
    HardwareMonitor, StorageMonitor, UtilizationMonitor,
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
        crate::disk::monitors::DiskStorageMonitor::get_root_info()
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
        crate::disk::monitors::DiskStorageMonitor::get_all_disks()
    }
} 