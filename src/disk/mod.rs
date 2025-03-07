use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct Disk {
    pub device: String,
    pub mount_point: String,
    pub fs_type: String,
    pub total: u64,
    pub available: u64,
    pub used: u64,
}

impl Disk {
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

    pub fn get_info() -> Result<Self> {
        Err(Error::NotImplemented(
            "Disk info not yet implemented".to_string(),
        ))
    }

    pub fn get_all() -> Result<Vec<Self>> {
        Err(Error::not_implemented(
            "Disk info retrieval not yet implemented",
        ))
    }

    pub fn usage_percentage(&self) -> f64 {
        (self.used as f64 / self.total as f64) * 100.0
    }

    pub fn is_nearly_full(&self) -> bool {
        self.usage_percentage() > 90.0
    }

    pub fn available_display(&self) -> String {
        Self::format_bytes(self.available)
    }

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
            total: 500 * 1024 * 1024 * 1024, 
            available: 100 * 1024 * 1024 * 1024, 
            used: 400 * 1024 * 1024 * 1024, 
        };

        assert_eq!(disk.usage_percentage(), 80.0);
        assert!(!disk.is_nearly_full());
        assert_eq!(disk.available_display(), "100.0 GB");
    }
}
