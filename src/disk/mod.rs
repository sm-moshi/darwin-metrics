use crate::{Error, Result};

/// Represents disk information and metrics
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
    /// Creates a new Disk instance with the given values
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

    /// Get current disk information
    pub fn get_info() -> Result<Self> {
        Err(Error::NotImplemented(
            "Disk info not yet implemented".to_string(),
        ))
    }

    /// Get information for all mounted disks
    ///
    /// # Returns
    /// Returns a `Result` containing a vector of disk information or an error if the information
    /// cannot be retrieved.
    ///
    /// # Examples
    /// ```no_run
    /// use darwin_metrics::disk::Disk;
    ///
    /// let disks = Disk::get_all().unwrap();
    /// for disk in disks {
    ///     println!("{}: {:.1}% used",
    ///         disk.mount_point,
    ///         disk.usage_percentage()
    ///     );
    /// }
    /// ```
    pub fn get_all() -> Result<Vec<Self>> {
        // TODO: Implement actual disk info retrieval
        Err(Error::not_implemented(
            "Disk info retrieval not yet implemented",
        ))
    }

    /// Returns disk usage as a percentage (0-100)
    pub fn usage_percentage(&self) -> f64 {
        (self.used as f64 / self.total as f64) * 100.0
    }

    /// Returns true if disk usage is above 90%
    pub fn is_nearly_full(&self) -> bool {
        self.usage_percentage() > 90.0
    }

    /// Returns available space in human-readable format
    pub fn available_display(&self) -> String {
        Self::format_bytes(self.available)
    }

    /// Format bytes into human-readable string
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
