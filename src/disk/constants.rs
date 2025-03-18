/// Common filesystem types
pub const FS_TYPE_APFS: &str = "apfs";
pub const FS_TYPE_HFS: &str = "hfs";
pub const FS_TYPE_EXFAT: &str = "exfat";
pub const FS_TYPE_FAT: &str = "msdos";
pub const FS_TYPE_NTFS: &str = "ntfs";
pub const FS_TYPE_NFS: &str = "nfs";
pub const FS_TYPE_SMB: &str = "smbfs";
pub const FS_TYPE_TMPFS: &str = "tmpfs";
pub const FS_TYPE_RAMFS: &str = "ramfs";

/// Mount flags
pub const MNT_RDONLY: u32 = 0x00000001;
pub const MNT_SYNCHRONOUS: u32 = 0x00000002;
pub const MNT_NOEXEC: u32 = 0x00000004;
pub const MNT_NOSUID: u32 = 0x00000008;
pub const MNT_NODEV: u32 = 0x00000010;
pub const MNT_UNION: u32 = 0x00000020;
pub const MNT_ASYNC: u32 = 0x00000040;
pub const MNT_CPROTECT: u32 = 0x00000080;

/// Default update interval in milliseconds
pub const DEFAULT_UPDATE_INTERVAL_MS: u64 = 1000;

/// Default disk update interval
pub const DISK_UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(DEFAULT_UPDATE_INTERVAL_MS);

/// Default buffer sizes
pub const DEFAULT_BUFFER_SIZE: usize = 4096;
pub const MAX_DEVICE_NAME_LEN: usize = 128;
pub const MAX_MOUNT_PATH_LEN: usize = 1024;
