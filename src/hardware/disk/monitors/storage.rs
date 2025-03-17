use std::{
    ffi::{CStr, CString},
    io,
    mem::{size_of, MaybeUninit},
};

use async_trait::async_trait;

use crate::{
    core::metrics::hardware::StorageMonitor,
    core::prelude::*,
    error::Error,
    error::Result,
    hardware::disk::types::Disk,
    hardware::disk::{
        constants::{FS_TYPE_APFS, FS_TYPE_NFS, FS_TYPE_RAMFS, FS_TYPE_SMB, FS_TYPE_TMPFS},
        types::{DiskConfig, DiskType},
    },
    utils::bindings::{getfsstat, statfs, Statfs, MNT_NOWAIT},
};

/// Monitor for disk storage metrics
#[derive(Debug)]
pub struct DiskStorageMonitor {
    disk: Disk,
}

impl DiskStorageMonitor {
    /// Creates a new DiskStorageMonitor for the given disk
    pub fn new(disk: Disk) -> Self {
        Self { disk }
    }

    /// Gets information about the root filesystem
    pub fn get_root_info() -> Result<Disk> {
        // Use a direct approach with statfs for the root filesystem
        let c_path = CString::new("/").expect("Failed to create CString for root path");
        let mut fs_stat = MaybeUninit::<Statfs>::uninit();

        let result = unsafe { statfs(c_path.as_ptr(), fs_stat.as_mut_ptr()) };

        if result != 0 {
            return Err(Error::system("Failed to get root filesystem information"));
        }

        // Extract data from statfs
        let stat = unsafe { fs_stat.assume_init() };

        // Extract filesystem type
        let fs_type = unsafe { CStr::from_ptr(stat.f_fstypename.as_ptr()).to_string_lossy().into_owned() };

        // Extract mount point (should be "/" for root)
        let mount_point = unsafe { CStr::from_ptr(stat.f_mntonname.as_ptr()).to_string_lossy().into_owned() };

        // Extract device name
        let device = unsafe { CStr::from_ptr(stat.f_mntfromname.as_ptr()).to_string_lossy().into_owned() };

        // Calculate disk space values
        let block_size = stat.f_bsize as u64;
        let total = stat.f_blocks * block_size;
        let available = stat.f_bavail * block_size;
        let used = total - (stat.f_bfree * block_size);

        // Create the Disk instance for root
        let config = DiskConfig {
            disk_type: DiskType::Unknown, // We'll skip the disk type detection for simplicity
            name: "Root".to_string(),
            is_boot_volume: true, // Root is always the boot volume
        };

        Ok(Disk::with_details(device, mount_point, fs_type, total, available, used, config))
    }

    /// Gets information about all mounted filesystems
    pub fn get_all_disks() -> Result<Vec<Disk>> {
        unsafe {
            let fs_count = getfsstat(std::ptr::null_mut(), 0, MNT_NOWAIT);
            if fs_count < 0 {
                return Err(Error::io_error("Failed to get filesystem count", io::Error::last_os_error()));
            }

            let mut stats = Vec::with_capacity(fs_count as usize);
            // Initialize with zeros since we'll write directly to this memory
            stats.resize(fs_count as usize, std::mem::zeroed());

            let result = getfsstat(stats.as_mut_ptr(), (size_of::<Statfs>() * fs_count as usize) as i32, MNT_NOWAIT);

            if result < 0 {
                return Err(Error::io_error("Failed to get filesystem stats", io::Error::last_os_error()));
            }

            // Truncate to actual number of entries returned
            stats.truncate(result as usize);

            Ok(stats
                .into_iter()
                .map(|stat| {
                    let device = CStr::from_ptr(stat.f_mntfromname.as_ptr()).to_string_lossy().into_owned();
                    let mount_point = CStr::from_ptr(stat.f_mntonname.as_ptr()).to_string_lossy().into_owned();
                    let fs_type = CStr::from_ptr(stat.f_fstypename.as_ptr()).to_string_lossy().into_owned();

                    // Determine disk type based on filesystem type
                    let disk_type = match fs_type.as_str() {
                        FS_TYPE_APFS => DiskType::SSD,
                        FS_TYPE_NFS | FS_TYPE_SMB => DiskType::Network,
                        FS_TYPE_TMPFS | FS_TYPE_RAMFS => DiskType::RAM,
                        _ => DiskType::Unknown,
                    };

                    // Check if it's the boot volume
                    let is_boot_volume = mount_point == "/";

                    // Calculate disk space values
                    let block_size = stat.f_bsize as u64;
                    let total = stat.f_blocks * block_size;
                    let available = stat.f_bavail * block_size;
                    let used = (stat.f_blocks - stat.f_bfree) * block_size;

                    // Create a disk config
                    let config = DiskConfig {
                        disk_type,
                        name: mount_point.split('/').next_back().unwrap_or("").to_string(),
                        is_boot_volume,
                    };

                    Disk::with_details(device, mount_point, fs_type, total, available, used, config)
                })
                .collect())
        }
    }

    fn get_disk_type(&self, fs_type: &str) -> DiskType {
        match fs_type {
            fs_type if fs_type == FS_TYPE_APFS => DiskType::SSD,
            fs_type if fs_type == FS_TYPE_NFS || fs_type == FS_TYPE_SMB => DiskType::Network,
            fs_type if fs_type == FS_TYPE_TMPFS || fs_type == FS_TYPE_RAMFS => DiskType::RAM,
            _ => DiskType::Unknown,
        }
    }

    async fn get_disk_config(&self) -> Result<DiskConfig> {
        Err(Error::not_implemented("Disk configuration retrieval"))
    }
}

#[async_trait]
impl HardwareMonitor for DiskStorageMonitor {
    type MetricType = DiskConfig;

    async fn name(&self) -> Result<String> {
        Ok("Disk Storage Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Disk".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.disk.device.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let config = self.get_disk_config().await?;
        Ok(Metric::new(config))
    }
}

#[async_trait::async_trait]
impl StorageMonitor for DiskStorageMonitor {
    async fn total_capacity(&self) -> Result<u64> {
        Ok(self.disk.total)
    }

    async fn available_capacity(&self) -> Result<u64> {
        Ok(self.disk.available)
    }

    async fn used_capacity(&self) -> Result<u64> {
        Ok(self.disk.used)
    }
}
