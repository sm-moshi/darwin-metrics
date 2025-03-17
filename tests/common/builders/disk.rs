use darwin_metrics::disk::{Disk, DiskConfig, DiskType};

/// Creates a test disk with the specified parameters
pub fn create_test_disk(
    device: &str,
    mount_point: &str,
    fs_type: &str,
    total: u64,
    available: u64,
    used: u64,
) -> Disk {
    Disk::new(
        device.to_string(),
        mount_point.to_string(),
        fs_type.to_string(),
        total,
        available,
        used,
    )
}

/// Creates a test disk with detailed configuration
pub fn create_test_disk_with_details(
    device: &str,
    mount_point: &str,
    fs_type: &str,
    total: u64,
    available: u64,
    used: u64,
    disk_type: DiskType,
    name: &str,
    is_boot_volume: bool,
) -> Disk {
    Disk::with_details(
        device.to_string(),
        mount_point.to_string(),
        fs_type.to_string(),
        total,
        available,
        used,
        DiskConfig {
            disk_type,
            name: name.to_string(),
            is_boot_volume,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_new() {
        let disk = create_test_disk("/dev/disk1s1", "/", "apfs", 1000, 200, 800);

        assert_eq!(disk.device, "/dev/disk1s1");
        assert_eq!(disk.mount_point, "/");
        assert_eq!(disk.fs_type, "apfs");
        assert_eq!(disk.total, 1000);
        assert_eq!(disk.available, 200);
        assert_eq!(disk.used, 800);
        assert_eq!(disk.disk_type, DiskType::Unknown);
        assert_eq!(disk.name, "");
        assert!(!disk.is_boot_volume);
    }

    #[test]
    fn test_disk_with_details() {
        let disk = create_test_disk_with_details(
            "/dev/disk1s1",
            "/",
            "apfs",
            1000,
            200,
            800,
            DiskType::SSD,
            "Macintosh HD",
            true,
        );

        assert_eq!(disk.device, "/dev/disk1s1");
        assert_eq!(disk.mount_point, "/");
        assert_eq!(disk.fs_type, "apfs");
        assert_eq!(disk.total, 1000);
        assert_eq!(disk.available, 200);
        assert_eq!(disk.used, 800);
        assert_eq!(disk.disk_type, DiskType::SSD);
        assert_eq!(disk.name, "Macintosh HD");
        assert!(disk.is_boot_volume);
    }
} 