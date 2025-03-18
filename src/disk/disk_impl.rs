// Only define these implementations when testing to avoid duplicate definitions
#[cfg(any(test, feature = "testing"))]
impl Disk {
    /// Creates a new Disk instance with the given parameters
    pub fn new(device: String, mount_point: String, fs_type: String, total: u64, available: u64, used: u64) -> Self {
        Self {
            device,
            mount_point,
            fs_type,
            total,
            available,
            used,
            disk_type: DiskType::Unknown,
            name: String::new(),
            is_boot_volume: false,
            last_update: Instant::now(),
            prev_read_bytes: 0,
            prev_write_bytes: 0,
        }
    }

    /// Gets information about the current system disk
    pub fn get_info() -> Result<Self> {
        DiskStorageMonitor::get_root_info()
    }

    /// Gets all mounted disks on the system
    pub fn get_all() -> Result<Vec<Self>> {
        DiskStorageMonitor::get_all_disks()
    }

    /// Creates a disk health monitor for this disk
    pub fn health_monitor(&self) -> DiskHealthMonitorImpl {
        DiskHealthMonitorImpl::new(self.clone())
    }

    /// Creates a disk I/O monitor for this disk
    pub fn io_monitor(&self) -> DiskIOMonitorImpl {
        DiskIOMonitorImpl::new(self.clone())
    }

    /// Creates a disk mount monitor for this disk
    pub fn mount_monitor(&self) -> DiskMountMonitorImpl {
        DiskMountMonitorImpl::new(self.clone())
    }

    /// Creates a disk performance monitor for this disk
    pub fn performance_monitor(&self) -> DiskPerformanceMonitorImpl {
        DiskPerformanceMonitorImpl::new(self.clone())
    }

    /// Creates a disk storage monitor for this disk
    pub fn storage_monitor(&self) -> DiskStorageMonitorImpl {
        DiskStorageMonitorImpl::new(self.clone())
    }

    /// Creates a disk utilization monitor for this disk
    pub fn utilization_monitor(&self) -> DiskUtilizationMonitorImpl {
        DiskUtilizationMonitorImpl::new(self.clone())
    }
}
