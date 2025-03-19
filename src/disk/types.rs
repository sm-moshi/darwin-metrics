use std::time::{Instant, SystemTime};

use crate::core::types::ByteSize;

/// The type of disk storage device
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiskType {
    /// Hard Disk Drive
    HDD,
    /// Solid State Drive
    SSD,
    /// Apple Fusion Drive (hybrid)
    Fusion,
    /// External drive
    External,
    /// Network mount
    Network,
    /// RAM Disk
    RAM,
    /// Virtual drive
    Virtual,
    /// Unknown drive type
    Unknown,
}

impl Default for DiskType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Configuration struct for creating a Disk with detailed options
#[derive(Debug, Clone, Default)]
pub struct DiskConfig {
    /// Disk type (SSD, HDD, etc)
    pub disk_type: DiskType,
    /// Volume name (e.g., "Macintosh HD")
    pub name: String,
    /// Whether this is the boot volume
    pub is_boot_volume: bool,
}

/// Basic disk volume information
#[derive(Debug, Clone, PartialEq)]
pub struct Disk {
    /// Device identifier (e.g., /dev/disk1s1)
    pub device: String,
    /// Mount point path (e.g., /)
    pub mount_point: String,
    /// Filesystem type (e.g., apfs, hfs+)
    pub fs_type: String,
    /// Total capacity in bytes
    pub total: u64,
    /// Available space in bytes
    pub available: u64,
    /// Used space in bytes
    pub used: u64,
    /// Disk type (SSD, HDD, etc)
    pub disk_type: DiskType,
    /// Volume name (e.g., "Macintosh HD")
    pub name: String,
    /// Whether this is the boot volume
    pub is_boot_volume: bool,
    /// Last update time
    pub(crate) last_update: Instant,
    /// Previous read bytes
    pub(crate) prev_read_bytes: u64,
    /// Previous written bytes
    pub(crate) prev_write_bytes: u64,
}

impl Disk {
    /// Create a new Disk with detailed information
    pub fn with_details(
        device: String,
        mount_point: String,
        fs_type: String,
        total: u64,
        available: u64,
        used: u64,
        config: DiskConfig,
    ) -> Self {
        Self {
            device,
            mount_point,
            fs_type,
            total,
            available,
            used,
            disk_type: config.disk_type,
            name: config.name,
            is_boot_volume: config.is_boot_volume,
            last_update: Instant::now(),
            prev_read_bytes: 0,
            prev_write_bytes: 0,
        }
    }
}

/// Detailed I/O performance metrics for a disk
#[derive(Debug, Clone, Default)]
pub struct DiskPerformance {
    /// Device identifier
    pub device: String,
    /// Read operations per second
    pub reads_per_second: f64,
    /// Write operations per second
    pub writes_per_second: f64,
    /// Bytes read per second
    pub bytes_read_per_second: u64,
    /// Bytes written per second
    pub bytes_written_per_second: u64,
    /// Average read latency in milliseconds
    pub read_latency_ms: f64,
    /// Average write latency in milliseconds
    pub write_latency_ms: f64,
    /// Disk utilization percentage (0-100)
    pub utilization: f64,
    /// Queue depth (number of pending I/O operations)
    pub queue_depth: f64,
}

/// Holds raw disk I/O statistics for performance calculations
#[derive(Debug, Clone)]
pub(crate) struct DiskStats {
    /// Total read operations
    pub read_ops: u64,
    /// Total write operations
    pub write_ops: u64,
    /// Total bytes read
    pub bytes_read: u64,
    /// Total bytes written
    pub bytes_written: u64,
    /// Total read time in nanoseconds
    pub read_time_ns: u64,
    /// Total write time in nanoseconds
    pub write_time_ns: u64,
    /// Timestamp when stats were collected
    pub timestamp: Instant,
}

impl Default for DiskStats {
    fn default() -> Self {
        Self {
            read_ops: 0,
            write_ops: 0,
            bytes_read: 0,
            bytes_written: 0,
            read_time_ns: 0,
            write_time_ns: 0,
            timestamp: Instant::now(),
        }
    }
}

/// Disk mount information
#[derive(Debug, Clone)]
pub struct DiskMount {
    /// The mount point path of the disk
    pub mount_point: String,
    /// The filesystem type of the disk
    pub filesystem_type: String,
    /// Whether this disk is the boot volume
    pub is_boot_volume: bool,
    /// Whether this disk is mounted as read-only
    pub is_readonly: bool,
    /// Whether this disk is a removable device
    pub is_removable: bool,
    /// Whether this disk is a network mount
    pub is_network: bool,
}

/// Disk health information
#[derive(Debug, Clone)]
pub struct DiskHealth {
    /// The SMART status of the disk (true = healthy)
    pub smart_status: bool,
    /// The temperature of the disk in Celsius
    pub temperature: f32,
    /// The number of hours the disk has been powered on
    pub power_on_hours: u32,
    /// The number of reallocated sectors on the disk
    pub reallocated_sectors: u32,
    /// The number of pending sectors on the disk
    pub pending_sectors: u32,
    /// The number of uncorrectable sectors on the disk
    pub uncorrectable_sectors: u32,
    /// The timestamp of the last health check
    pub last_check: SystemTime,
}

/// Disk performance metrics
#[derive(Debug, Clone)]
pub struct DiskPerformanceMetrics {
    /// Total bytes read since last measurement
    pub bytes_read: ByteSize,
    /// Total bytes written since last measurement
    pub bytes_written: ByteSize,
    /// Number of read operations since last measurement
    pub read_ops: u64,
    /// Number of write operations since last measurement
    pub write_ops: u64,
    /// Average read latency in milliseconds
    pub read_latency_ms: f64,
    /// Average write latency in milliseconds
    pub write_latency_ms: f64,
    /// Timestamp of the measurement
    pub timestamp: SystemTime,
}

impl DiskPerformanceMetrics {
    pub fn new(
        bytes_read: ByteSize,
        bytes_written: ByteSize,
        read_ops: u64,
        write_ops: u64,
        read_latency_ms: f64,
        write_latency_ms: f64,
    ) -> Self {
        Self {
            bytes_read,
            bytes_written,
            read_ops,
            write_ops,
            read_latency_ms,
            write_latency_ms,
            timestamp: SystemTime::now(),
        }
    }
}

/// Represents byte metrics for disk I/O
#[derive(Debug, Clone)]
pub struct ByteMetrics {
    /// Total bytes read
    pub bytes_read: u64,
    /// Total bytes written
    pub bytes_written: u64,
    /// Total bytes transferred
    pub total_bytes: u64,
}

impl ByteMetrics {
    pub fn new(bytes_read: u64, bytes_written: u64) -> Self {
        Self {
            bytes_read,
            bytes_written,
            total_bytes: bytes_read + bytes_written,
        }
    }
}
