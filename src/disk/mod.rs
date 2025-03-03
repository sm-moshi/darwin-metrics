//! Disk metrics and storage information for macOS systems.
//!
//! This module provides functionality to gather disk-related metrics and information
//! on macOS systems. It supports monitoring of mounted volumes, including:
//! - Storage capacity and usage
//! - Mount points and device paths
//! - File system types
//! - Space availability warnings
//!
//! # Examples
//!
//! ```no_run
//! use darwin_metrics::disk::Disk;
//!
//! fn main() -> darwin_metrics::Result<()> {
//!     // Get information for all mounted disks
//!     let disks = Disk::get_all()?;
//!     
//!     for disk in disks {
//!         println!("Mount: {}", disk.mount_point);
//!         println!("  Type: {}", disk.fs_type);
//!         println!("  Available: {}", disk.available_display());
//!         println!("  Usage: {:.1}%", disk.usage_percentage());
//!         
//!         if disk.is_nearly_full() {
//!             println!("  Warning: Disk is nearly full!");
//!         }
//!     }
//!     Ok(())
//! }
//! ```

use crate::{Error, Result};

/// Represents disk information and metrics for a mounted volume.
///
/// This struct provides access to various disk metrics and information, including:
/// - Device path and mount point
/// - File system type
/// - Storage capacity and usage statistics
///
/// The struct provides methods for calculating usage percentages and formatting
/// storage values in human-readable format.
///
/// # Examples
///
/// ```no_run
/// use darwin_metrics::disk::Disk;
///
/// let disk = Disk::new(
///     "/dev/disk1s1".to_string(),
///     "/".to_string(),
///     "apfs".to_string(),
///     500 * 1024 * 1024 * 1024, // 500GB total
///     100 * 1024 * 1024 * 1024, // 100GB available
///     400 * 1024 * 1024 * 1024, // 400GB used
/// );
///
/// println!("Disk at {} ({}):", disk.mount_point, disk.fs_type);
/// println!("  Available: {}", disk.available_display());
/// println!("  Usage: {:.1}%", disk.usage_percentage());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Disk {
    /// Device name (e.g., "/dev/disk1s1")
    pub device: String,
    /// Mount point (e.g., "/")
    pub mount_point: String,
    /// File system type (e.g., "apfs")
    pub fs_type: String,
    /// Total size in bytes
    pub total: u64,
    /// Available space in bytes
    pub available: u64,
    /// Used space in bytes
    pub used: u64,
}

impl Disk {
    /// Creates a new Disk instance with the given values.
    ///
    /// # Arguments
    ///
    /// * `device` - Device path (e.g., "/dev/disk1s1")
    /// * `mount_point` - Mount point path (e.g., "/")
    /// * `fs_type` - File system type (e.g., "apfs")
    /// * `total` - Total size in bytes
    /// * `available` - Available space in bytes
    /// * `used` - Used space in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use darwin_metrics::disk::Disk;
    ///
    /// let disk = Disk::new(
    ///     "/dev/disk1s1".to_string(),
    ///     "/".to_string(),
    ///     "apfs".to_string(),
    ///     1024 * 1024 * 1024,     // 1GB total
    ///     512 * 1024 * 1024,      // 512MB available
    ///     512 * 1024 * 1024,      // 512MB used
    /// );
    /// ```
    pub fn new(
        device: String,
        mount_point: String,
        fs_type: String,
        total: u64,
        available: u64,
        used: u64,
    ) -> Self {
        Self {
            device,
            mount_point,
            fs_type,
            total,
            available,
            used,
        }
    }

    /// Get current disk information.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Disk>` which is:
    /// - `Ok(Disk)` containing the disk information
    /// - `Err(Error)` if the information cannot be retrieved
    ///
    /// # Note
    ///
    /// This method is currently not implemented and will return a
    /// `NotImplemented` error.
    pub fn get_info() -> Result<Self> {
        Err(Error::NotImplemented(
            "Disk info not yet implemented".to_string(),
        ))
    }

    /// Get information for all mounted disks.
    ///
    /// This method retrieves information about all mounted disk volumes
    /// in the system, including removable media.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Vec<Disk>>` which is:
    /// - `Ok(Vec<Disk>)` containing information for all mounted disks
    /// - `Err(Error)` if the information cannot be retrieved
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use darwin_metrics::disk::Disk;
    ///
    /// let disks = Disk::get_all()?;
    /// for disk in disks {
    ///     println!("{}: {:.1}% used",
    ///         disk.mount_point,
    ///         disk.usage_percentage()
    ///     );
    /// }
    /// # Ok::<(), darwin_metrics::Error>(())
    /// ```
    ///
    /// # Note
    ///
    /// This method is currently not implemented and will return a
    /// `NotImplemented` error.
    pub fn get_all() -> Result<Vec<Self>> {
        // TODO: Implement actual disk info retrieval
        Err(Error::not_implemented(
            "Disk info retrieval not yet implemented",
        ))
    }

    /// Returns disk usage as a percentage.
    ///
    /// Calculates the percentage of used space relative to total capacity.
    /// The result is always between 0.0 and 100.0.
    ///
    /// # Returns
    ///
    /// Returns a `f64` representing the percentage of disk space used.
    pub fn usage_percentage(&self) -> f64 {
        (self.used as f64 / self.total as f64) * 100.0
    }

    /// Returns true if disk usage is above 90%.
    ///
    /// This method can be used to monitor for low disk space conditions.
    ///
    /// # Returns
    ///
    /// Returns `true` if more than 90% of the disk space is used,
    /// `false` otherwise.
    pub fn is_nearly_full(&self) -> bool {
        self.usage_percentage() > 90.0
    }

    /// Returns available space in human-readable format.
    ///
    /// Formats the available space using appropriate units (TB, GB, MB, KB, bytes).
    ///
    /// # Returns
    ///
    /// Returns a `String` containing the formatted size (e.g., "1.5 GB").
    ///
    /// # Examples
    ///
    /// ```
    /// use darwin_metrics::disk::Disk;
    ///
    /// let disk = Disk::new(
    ///     "/dev/disk1".to_string(),
    ///     "/".to_string(),
    ///     "apfs".to_string(),
    ///     1024 * 1024 * 1024,     // 1GB total
    ///     512 * 1024 * 1024,      // 512MB available
    ///     512 * 1024 * 1024,      // 512MB used
    /// );
    /// assert_eq!(disk.available_display(), "512.0 MB");
    /// ```
    pub fn available_display(&self) -> String {
        Self::format_bytes(self.available)
    }

    /// Format bytes into human-readable string.
    ///
    /// Internal helper method that converts a byte count into a human-readable
    /// string with appropriate units.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The number of bytes to format
    ///
    /// # Returns
    ///
    /// Returns a `String` containing the formatted size with units.
    fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        if bytes >= TB {
            format!("{:.1} TB", bytes as f64 / TB as f64)
        } else if bytes >= GB {
            format!("{:.1} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.1} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} bytes", bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_calculations() {
        let disk = Disk {
            device: "/dev/disk1s1".to_string(),
            mount_point: "/".to_string(),
            fs_type: "apfs".to_string(),
            total: 500 * 1024 * 1024 * 1024,     // 500GB
            available: 100 * 1024 * 1024 * 1024, // 100GB
            used: 400 * 1024 * 1024 * 1024,      // 400GB
        };

        assert_eq!(disk.usage_percentage(), 80.0);
        assert!(!disk.is_nearly_full());
        assert_eq!(disk.available_display(), "100.0 GB");
    }
}
