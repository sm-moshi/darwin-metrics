/// Common filesystem types
pub const FS_TYPE_APFS: &str = "apfs";
/// HFS (Hierarchical File System) filesystem type identifier
pub const FS_TYPE_HFS: &str = "hfs";
/// ExFAT (Extended File Allocation Table) filesystem type identifier
pub const FS_TYPE_EXFAT: &str = "exfat";
/// FAT/MSDOS filesystem type identifier
pub const FS_TYPE_FAT: &str = "msdos";
/// NTFS (New Technology File System) filesystem type identifier
pub const FS_TYPE_NTFS: &str = "ntfs";
/// NFS (Network File System) filesystem type identifier
pub const FS_TYPE_NFS: &str = "nfs";
/// SMB (Server Message Block) filesystem type identifier
pub const FS_TYPE_SMB: &str = "smbfs";
/// tmpfs (Temporary File System) identifier
pub const FS_TYPE_TMPFS: &str = "tmpfs";
/// RAM filesystem type identifier
pub const FS_TYPE_RAMFS: &str = "ramfs";

/// Mount flags
pub const MNT_RDONLY: u32 = 0x00000001;
/// Mount flag: Updates are performed synchronously
pub const MNT_SYNCHRONOUS: u32 = 0x00000002;
/// Mount flag: Executables are not permitted on this filesystem
pub const MNT_NOEXEC: u32 = 0x00000004;
/// Mount flag: Setuid/setgid bits are ignored by exec
pub const MNT_NOSUID: u32 = 0x00000008;
/// Mount flag: No device-special files permitted on this filesystem
pub const MNT_NODEV: u32 = 0x00000010;
/// Mount flag: Union with underlying filesystem
pub const MNT_UNION: u32 = 0x00000020;
/// Mount flag: Updates are performed asynchronously
pub const MNT_ASYNC: u32 = 0x00000040;
/// Mount flag: Content protection enabled
pub const MNT_CPROTECT: u32 = 0x00000080;

/// Default update interval in milliseconds
pub const DEFAULT_UPDATE_INTERVAL_MS: u64 = 1000;

/// Default disk update interval
pub const DISK_UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(DEFAULT_UPDATE_INTERVAL_MS);

/// Default buffer sizes
pub const DEFAULT_BUFFER_SIZE: usize = 4096;
/// Maximum length for device names
pub const MAX_DEVICE_NAME_LEN: usize = 128;
/// Maximum length for mount paths
pub const MAX_MOUNT_PATH_LEN: usize = 1024;
