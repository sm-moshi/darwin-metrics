//! FFI bindings to macOS system APIs.
//!
//! This module centralizes all the FFI bindings for macOS system APIs used throughout the crate. It provides a clean
//! interface to low-level C functions from various macOS frameworks:
//!
//! - `sysctl` for system information
//! - `IOKit` for hardware access
//! - Mach host functions for memory statistics
//!
//! By centralizing these bindings, we improve maintainability and reduce redundancy across modules.

use std::{
    ffi::c_void as ffi_c_void,
    os::raw::{c_char, c_int, c_uint, c_void},
};

//------------------------------------------------------------------------------
// sysctl FFI bindings for macOS
//------------------------------------------------------------------------------

/// Constants for sysctl used in various information queries
pub mod sysctl_constants {
    use std::os::raw::c_int;

    // General categories
    pub const CTL_KERN: c_int = 1;
    pub const CTL_HW: c_int = 6;
    pub const CTL_VM: c_int = 2;

    // Kernel-related
    pub const KERN_PROC: c_int = 14;
    pub const KERN_PROC_ALL: c_int = 0;
    pub const KERN_PROC_PID: c_int = 1;
    pub const KERN_PROC_PGRP: c_int = 2;
    pub const KERN_PROC_TTY: c_int = 3;
    pub const KERN_PROC_UID: c_int = 4;
    pub const KERN_PROC_RUID: c_int = 5;

    // Hardware-related
    pub const HW_MACHINE: c_int = 1;
    pub const HW_MEMSIZE: c_int = 24;

    // VM-related
    pub const VM_SWAPUSAGE: c_int = 5;
}

/// Process information structure from sysctl/kern_proc.h
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct kinfo_proc {
    pub kp_proc: proc_info,
    pub kp_eproc: extern_proc,
}

/// Basic process information structure
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct proc_info {
    pub p_flag: c_int,
    pub p_pid: c_int,
    pub p_ppid: c_int,
    pub p_stat: c_int,
    // More fields exist but aren't needed for basic functionality
}

/// Extended process information structure
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct extern_proc {
    pub p_starttime: timeval,
    pub p_comm: [u8; 16], /* MAXCOMLEN
                           * More fields exist but aren't needed for basic functionality */
}

/// Time value structure used in BSD APIs
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct timeval {
    pub tv_sec: i64,
    pub tv_usec: i32,
}

//------------------------------------------------------------------------------
// External C functions (sysctl, Mach, IOKit)
//------------------------------------------------------------------------------

// sysctl function for system information
#[link(name = "System", kind = "framework")]
extern "C" {
    pub fn sysctl(
        name: *const c_int,
        namelen: c_uint,
        oldp: *mut c_void,
        oldlenp: *mut usize,
        newp: *const c_void,
        newlen: usize,
    ) -> c_int;
}

//------------------------------------------------------------------------------
// VM Statistics and Memory-related structures
//------------------------------------------------------------------------------

// Memory and VM constants
pub const KERN_SUCCESS: i32 = 0;
pub const HOST_VM_INFO64: i32 = 4;
pub const HOST_VM_INFO64_COUNT: u32 = 38;

pub type HostInfoT = *mut i32;
pub type MachPortT = u32;

#[repr(C)]
#[derive(Debug, Default)]
pub struct vm_statistics64 {
    pub free_count: u32,
    pub active_count: u32,
    pub inactive_count: u32,
    pub wire_count: u32,
    pub zero_fill_count: u64,
    pub reactivations: u64,
    pub pageins: u64,
    pub pageouts: u64,
    pub faults: u64,
    pub cow_faults: u64,
    pub lookups: u64,
    pub hits: u64,
    pub purges: u64,
    pub purgeable_count: u32,
    pub speculative_count: u32,
    pub decompressions: u64,
    pub compressions: u64,
    pub swapins: u64,
    pub swapouts: u64,
    pub compressor_page_count: u32,
    pub throttled_count: u32,
    pub external_page_count: u32,
    pub internal_page_count: u32,
    pub total_uncompressed_pages_in_compressor: u64,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct xsw_usage {
    pub xsu_total: u64,
    pub xsu_used: u64,
    pub xsu_avail: u64,
}

// Mach host functions
extern "C" {
    pub static vm_kernel_page_size: u32;

    pub fn host_statistics64(
        host_priv: MachPortT,
        flavor: i32,
        host_info_out: HostInfoT,
        host_info_outCnt: *mut u32,
    ) -> i32;

    pub fn mach_host_self() -> MachPortT;
}

//------------------------------------------------------------------------------
// IOKit Constants and Data Structures
//------------------------------------------------------------------------------

// IOKit constants
pub const KERNEL_INDEX_SMC: u32 = 2;
pub const SMC_CMD_READ_BYTES: u8 = 5;
pub const SMC_CMD_READ_KEYINFO: u8 = 9;
pub const IO_RETURN_SUCCESS: i32 = 0; // Renamed from kIOReturnSuccess to follow Rust naming convention

// IOKit basic types
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IOByteCount(pub usize);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IOOptionBits(pub u32);

// SMC key definitions for temperature sensors
pub const SMC_KEY_CPU_TEMP: [c_char; 4] =
    [b'T' as c_char, b'C' as c_char, b'0' as c_char, b'P' as c_char]; // CPU Temp (TC0P)

pub const SMC_KEY_GPU_TEMP: [c_char; 4] =
    [b'T' as c_char, b'G' as c_char, b'0' as c_char, b'P' as c_char]; // GPU Temp (TG0P)

// Fan speed keys
pub const SMC_KEY_FAN_NUM: [c_char; 4] =
    [b'F' as c_char, b'N' as c_char, b'u' as c_char, b'm' as c_char]; // Number of fans (FNum)

pub const SMC_KEY_FAN_SPEED: [c_char; 4] =
    [b'F' as c_char, b'0' as c_char, b'A' as c_char, b'c' as c_char]; // Fan 0 Speed (F0Ac)

pub const SMC_KEY_FAN1_SPEED: [c_char; 4] =
    [b'F' as c_char, b'1' as c_char, b'A' as c_char, b'c' as c_char]; // Fan 1 Speed (F1Ac)

// Fan speed min/max keys
pub const SMC_KEY_FAN0_MIN: [c_char; 4] =
    [b'F' as c_char, b'0' as c_char, b'M' as c_char, b'n' as c_char]; // Fan 0 Min Speed (F0Mn)

pub const SMC_KEY_FAN0_MAX: [c_char; 4] =
    [b'F' as c_char, b'0' as c_char, b'M' as c_char, b'x' as c_char]; // Fan 0 Max Speed (F0Mx)

// Additional thermal sensors
pub const SMC_KEY_HEATSINK_TEMP: [c_char; 4] =
    [b'T' as c_char, b'h' as c_char, b'0' as c_char, b'H' as c_char]; // Heatsink temp (Th0H)

pub const SMC_KEY_AMBIENT_TEMP: [c_char; 4] =
    [b'T' as c_char, b'A' as c_char, b'0' as c_char, b'P' as c_char]; // Ambient temp (TA0P)

pub const SMC_KEY_BATTERY_TEMP: [c_char; 4] =
    [b'T' as c_char, b'B' as c_char, b'0' as c_char, b'T' as c_char]; // Battery temp (TB0T)

// Power and thermal throttling keys
pub const SMC_KEY_CPU_POWER: [c_char; 4] =
    [b'P' as c_char, b'C' as c_char, b'P' as c_char, b'C' as c_char]; // CPU package power (PCPC)

pub const SMC_KEY_CPU_THROTTLE: [c_char; 4] =
    [b'P' as c_char, b'C' as c_char, b'T' as c_char, b'C' as c_char]; // CPU thermal throttling (PCTC)

// Apple Silicon specific power keys (based on macmon and NeoAsitop)
pub const SMC_KEY_PACKAGE_POWER: [c_char; 4] =
    [b'P' as c_char, b'M' as c_char, b'P' as c_char, b'0' as c_char]; // PMP0 - Package power (SoC)

pub const SMC_KEY_GPU_POWER: [c_char; 4] =
    [b'P' as c_char, b'G' as c_char, b'P' as c_char, b'G' as c_char]; // PGPG - GPU power

pub const SMC_KEY_DRAM_POWER: [c_char; 4] =
    [b'P' as c_char, b'D' as c_char, b'R' as c_char, b'P' as c_char]; // PDRP - DRAM/Memory power

pub const SMC_KEY_NEURAL_POWER: [c_char; 4] =
    [b'P' as c_char, b'N' as c_char, b'P' as c_char, b'0' as c_char]; // PNP0 - Neural Engine power

// SMC data structures
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SMCVersion {
    pub major: u8,
    pub minor: u8,
    pub build: u8,
    pub reserved: [u8; 1],
    pub release: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
#[allow(non_snake_case)]
pub struct SMCPLimitData {
    pub version: u16,
    pub length: u16,
    pub cpu_plimit: u32,
    pub gpu_plimit: u32,
    pub mem_plimit: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SMCKeyData_vers_t {
    pub version: SMCVersion,
    pub reserved: [u8; 16],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SMCKeyData_pLimitData_t {
    pub p_limit_data: SMCPLimitData,
    pub reserved: [u8; 10],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SMCKeyData_keyInfo_t {
    pub data_size: u32,
    pub data_type: [u8; 4],
    pub data_attributes: u8,
}

#[repr(C, packed)]
pub union SMCKeyData_t_data {
    pub bytes: [u8; 32],
    pub uint32: u32,
    pub float: f32,
    pub sint16: i16,
    pub vers: SMCKeyData_vers_t,
    pub p_limit: SMCKeyData_pLimitData_t,
    pub key_info: SMCKeyData_keyInfo_t,
}

// Manual implementations for union
impl Clone for SMCKeyData_t_data {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for SMCKeyData_t_data {}

#[repr(C, packed)]
pub struct SMCKeyData_t {
    pub key: u32,
    pub vers: u8,
    pub p_limit_data: u8,
    pub key_info: u8,
    pub padding: u8,
    pub result: u8,
    pub status: u8,
    pub data8: u8,
    pub data32: u8,
    pub bytes: [u8; 2],
    pub data: SMCKeyData_t_data,
}

// Manual implementations for struct containing union
impl Clone for SMCKeyData_t {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for SMCKeyData_t {}

impl Default for SMCKeyData_t {
    fn default() -> Self {
        unsafe {
            Self {
                key: 0,
                vers: 0,
                p_limit_data: 0,
                key_info: 0,
                padding: 0,
                result: 0,
                status: 0,
                data8: 0,
                data32: 0,
                bytes: [0; 2],
                data: std::mem::zeroed(),
            }
        }
    }
}

// IOKit function declarations
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    // IOService functions
    pub fn IOServiceGetMatchingService(masterPort: u32, matchingDict: *const ffi_c_void) -> u32;
    pub fn IOServiceMatching(serviceName: *const c_char) -> *mut ffi_c_void;
    pub fn IOServiceOpen(service: u32, owningTask: u32, type_: u32, handle: *mut u32) -> i32;
    pub fn IOServiceClose(handle: u32) -> i32;
    pub fn IORegistryEntryCreateCFProperties(
        entry: u32,
        properties: *mut *mut ffi_c_void,
        allocator: *mut ffi_c_void,
        options: u32,
    ) -> i32;

    // SMC specific functions
    pub fn IOConnectCallStructMethod(
        connection: u32,
        selector: u32,
        inputStruct: *const SMCKeyData_t,
        inputStructCnt: IOByteCount,
        outputStruct: *mut SMCKeyData_t,
        outputStructCnt: *mut IOByteCount,
    ) -> i32;
}

//------------------------------------------------------------------------------
// Process state constants
//------------------------------------------------------------------------------

/// Process state values from sys/proc.h
pub mod proc_state {
    pub const SIDL: u8 = 1; // Process being created by fork
    pub const SRUN: u8 = 2; // Running
    pub const SSLEEP: u8 = 3; // Sleeping on an address
    pub const SSTOP: u8 = 4; // Process debugging or suspension
    pub const SZOMB: u8 = 5; // Awaiting collection by parent
}

//------------------------------------------------------------------------------
// Filesystem related structures and bindings
//------------------------------------------------------------------------------

/// Filesystem statistics structure from sys/mount.h
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Statfs {
    pub f_bsize: u32,                  // Fundamental file system block size
    pub f_iosize: i32,                 // Optimal transfer block size
    pub f_blocks: u64,                 // Total data blocks in file system
    pub f_bfree: u64,                  // Free blocks in file system
    pub f_bavail: u64,                 // Free blocks available to non-superuser
    pub f_files: u64,                  // Total file nodes in file system
    pub f_ffree: u64,                  // Free nodes available
    pub f_fsid: [i32; 2],              // File system ID
    pub f_owner: u32,                  // User ID of mount owner
    pub f_type: u32,                   // Type of file system
    pub f_flags: u32,                  // Copy of mount flags
    pub f_fssubtype: u32,              // File system subtype
    pub f_fstypename: [c_char; 16],    // File system type name
    pub f_mntonname: [c_char; 1024],   // Mount point
    pub f_mntfromname: [c_char; 1024], // Mount source
    pub f_reserved: [u32; 8],          // Reserved for future use
}

/// Filesystem mount flags
pub const MNT_NOWAIT: c_int = 2; // Don't block for filesystem sync

// Filesystem functions
#[link(name = "System", kind = "framework")]
extern "C" {
    /// Get statistics about a mounted filesystem
    pub fn statfs(path: *const c_char, buf: *mut Statfs) -> c_int;

    /// Get statistics about all mounted filesystems
    pub fn getfsstat(buf: *mut Statfs, bufsize: c_int, flags: c_int) -> c_int;
}

//------------------------------------------------------------------------------
// Metal Framework Bindings for GPU Access
//------------------------------------------------------------------------------

/// Metal framework type for device access
pub type MTLDeviceRef = *mut c_void;

#[link(name = "Metal", kind = "framework")]
extern "C" {
    /// Creates and returns the default system Metal device Used to access GPU information including name and
    /// capabilities
    pub fn MTLCreateSystemDefaultDevice() -> MTLDeviceRef;
}

//------------------------------------------------------------------------------
// Process and System Info Functions
//------------------------------------------------------------------------------

/// Constants for proc_pidinfo
pub const PROC_PIDTASKINFO: c_int = 4;

extern "C" {
    /// Get system load averages for the past 1, 5, and 15 minutes
    pub fn getloadavg(loads: *mut f64, nelem: c_int) -> c_int;

    /// Get process information by PID
    pub fn proc_pidinfo(
        pid: c_int,
        flavor: c_int,
        arg: u64,
        buffer: *mut c_void,
        buffersize: c_int,
    ) -> c_int;
}

//------------------------------------------------------------------------------
// Network related data structures and bindings
//------------------------------------------------------------------------------

/// Network Address Family constants
pub mod address_family {
    pub const AF_UNSPEC: u8 = 0; // Unspecified
    pub const AF_INET: u8 = 2; // IPv4
    pub const AF_INET6: u8 = 30; // IPv6
    pub const AF_LINK: u8 = 18; // Link level interface
}

/// Interface Flags constants
pub mod if_flags {
    pub const IFF_UP: u32 = 0x1; // Interface is up
    pub const IFF_BROADCAST: u32 = 0x2; // Broadcast address valid
    pub const IFF_DEBUG: u32 = 0x4; // Turn on debugging
    pub const IFF_LOOPBACK: u32 = 0x8; // Is a loopback net
    pub const IFF_POINTOPOINT: u32 = 0x10; // Interface is point-to-point link
    pub const IFF_RUNNING: u32 = 0x40; // Resources allocated
    pub const IFF_NOARP: u32 = 0x80; // No address resolution protocol
    pub const IFF_PROMISC: u32 = 0x100; // Receive all packets
    pub const IFF_ALLMULTI: u32 = 0x200; // Receive all multicast packets
    pub const IFF_MULTICAST: u32 = 0x8000; // Supports multicast
    pub const IFF_WIRELESS: u32 = 0x20; // Wireless
}

#[repr(C)]
pub struct ifaddrs {
    pub ifa_next: *mut ifaddrs,
    pub ifa_name: *mut c_char,
    pub ifa_flags: u32,
    pub ifa_addr: *mut sockaddr,
    pub ifa_netmask: *mut sockaddr,
    pub ifa_dstaddr: *mut sockaddr,
    pub ifa_data: *mut c_void,
}

#[repr(C)]
pub struct sockaddr {
    pub sa_len: u8,
    pub sa_family: u8,
    pub sa_data: [c_char; 14],
}

#[repr(C)]
pub struct sockaddr_in {
    pub sin_len: u8,
    pub sin_family: u8,
    pub sin_port: u16,
    pub sin_addr: in_addr,
    pub sin_zero: [c_char; 8],
}

#[repr(C)]
pub struct in_addr {
    pub s_addr: u32,
}

#[repr(C)]
pub struct sockaddr_in6 {
    pub sin6_len: u8,
    pub sin6_family: u8,
    pub sin6_port: u16,
    pub sin6_flowinfo: u32,
    pub sin6_addr: in6_addr,
    pub sin6_scope_id: u32,
}

#[repr(C)]
pub struct in6_addr {
    pub s6_addr: [u8; 16],
}

#[repr(C)]
pub struct sockaddr_dl {
    pub sdl_len: u8,
    pub sdl_family: u8,
    pub sdl_index: u16,
    pub sdl_type: u8,
    pub sdl_nlen: u8,
    pub sdl_alen: u8,
    pub sdl_slen: u8,
    pub sdl_data: [c_char; 12],
}

/// Network interface data structure (from net/if_var.h) This structure contains all the traffic statistics and
/// interface properties
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct if_data {
    pub ifi_type: u8,            // Type of interface (ethernet, loopback, etc.)
    pub ifi_physical: u8,        // Physical port/connector type
    pub ifi_addrlen: u8,         // Media address length
    pub ifi_hdrlen: u8,          // Media header length
    pub ifi_recvquota: u8,       // Receive quota (obsolete)
    pub ifi_xmitquota: u8,       // Transmit quota (obsolete)
    pub ifi_unused1: u8,         // Unused padding
    pub ifi_mtu: u32,            // Maximum transmission unit
    pub ifi_metric: u32,         // Routing metric
    pub ifi_baudrate: u32,       // Linespeed
    pub ifi_ipackets: u32,       // Packets received on interface
    pub ifi_ierrors: u32,        // Input errors on interface
    pub ifi_opackets: u32,       // Packets sent on interface
    pub ifi_oerrors: u32,        // Output errors on interface
    pub ifi_collisions: u32,     // Collisions on csma interfaces
    pub ifi_ibytes: u32,         // Total number of bytes received
    pub ifi_obytes: u32,         // Total number of bytes sent
    pub ifi_imcasts: u32,        // Multicast packets received
    pub ifi_omcasts: u32,        // Multicast packets sent
    pub ifi_iqdrops: u32,        // Dropped on input, this interface
    pub ifi_noproto: u32,        // Destined for unsupported protocol
    pub ifi_recvtiming: u32,     // Receive timing offset (usec)
    pub ifi_xmittiming: u32,     // Transmit timing offset (usec)
    pub ifi_lastchange: timeval, // Time of last change
}

/// 64-bit version of if_data, available since macOS 10.8 This allows for tracking traffic stats beyond 4GB on modern
/// interfaces
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct if_data64 {
    pub ifi_type: u8,            // Type of interface (ethernet, loopback, etc.)
    pub ifi_physical: u8,        // Physical port/connector type
    pub ifi_addrlen: u8,         // Media address length
    pub ifi_hdrlen: u8,          // Media header length
    pub ifi_recvquota: u8,       // Receive quota (obsolete)
    pub ifi_xmitquota: u8,       // Transmit quota (obsolete)
    pub ifi_unused1: u8,         // Unused padding
    pub ifi_mtu: u32,            // Maximum transmission unit
    pub ifi_metric: u32,         // Routing metric
    pub ifi_baudrate: u64,       // Linespeed (64-bit)
    pub ifi_ipackets: u64,       // Packets received on interface (64-bit)
    pub ifi_ierrors: u64,        // Input errors on interface (64-bit)
    pub ifi_opackets: u64,       // Packets sent on interface (64-bit)
    pub ifi_oerrors: u64,        // Output errors on interface (64-bit)
    pub ifi_collisions: u64,     // Collisions on csma interfaces (64-bit)
    pub ifi_ibytes: u64,         // Total number of bytes received (64-bit)
    pub ifi_obytes: u64,         // Total number of bytes sent (64-bit)
    pub ifi_imcasts: u64,        // Multicast packets received (64-bit)
    pub ifi_omcasts: u64,        // Multicast packets sent (64-bit)
    pub ifi_iqdrops: u64,        // Dropped on input, this interface (64-bit)
    pub ifi_noproto: u64,        // Destined for unsupported protocol (64-bit)
    pub ifi_recvtiming: u32,     // Receive timing offset (usec)
    pub ifi_xmittiming: u32,     // Transmit timing offset (usec)
    pub ifi_lastchange: timeval, // Time of last change
}

// Network Reachability flags for SCNetworkReachabilityGetFlags These match Apple's constants, so we keep the naming
// convention
#[allow(non_upper_case_globals)]
pub mod reachability_flags {
    pub const kSCNetworkReachabilityFlagsTransientConnection: u32 = 1 << 0;
    pub const kSCNetworkReachabilityFlagsReachable: u32 = 1 << 1;
    pub const kSCNetworkReachabilityFlagsConnectionRequired: u32 = 1 << 2;
    pub const kSCNetworkReachabilityFlagsConnectionOnTraffic: u32 = 1 << 3;
    pub const kSCNetworkReachabilityFlagsInterventionRequired: u32 = 1 << 4;
    pub const kSCNetworkReachabilityFlagsConnectionOnDemand: u32 = 1 << 5;
    pub const kSCNetworkReachabilityFlagsIsLocalAddress: u32 = 1 << 16;
    pub const kSCNetworkReachabilityFlagsIsDirect: u32 = 1 << 17;
    pub const kSCNetworkReachabilityFlagsIsWWAN: u32 = 1 << 18;
}

// System native network API bindings
#[link(name = "System", kind = "framework")]
extern "C" {
    // Interface enumeration APIs
    pub fn getifaddrs(ifap: *mut *mut ifaddrs) -> c_int;
    pub fn freeifaddrs(ifp: *mut ifaddrs) -> c_void;

    // sysctl functions for network statistics
    pub fn sysctlbyname(
        name: *const c_char,
        oldp: *mut c_void,
        oldlenp: *mut usize,
        newp: *const c_void,
        newlen: usize,
    ) -> c_int;

    // IOKit registry entry parent function
    pub fn IORegistryEntryGetParentEntry(
        entry: c_uint,
        plane: *const c_char,
        parent: *mut c_uint,
    ) -> i32;
}

// SystemConfiguration framework bindings for network interface monitoring
#[link(name = "SystemConfiguration", kind = "framework")]
extern "C" {
    // Dynamic store functions for network configuration
    pub fn SCDynamicStoreCreate(
        allocator: *mut ffi_c_void,
        name: *const ffi_c_void,
        callout: *mut ffi_c_void,
        context: *mut ffi_c_void,
    ) -> *mut ffi_c_void;

    pub fn SCDynamicStoreCopyValue(
        store: *mut ffi_c_void,
        key: *const ffi_c_void,
        value: *mut *mut ffi_c_void,
    ) -> *mut ffi_c_void;

    // Network reachability functions
    pub fn SCNetworkReachabilityCreateWithAddress(
        allocator: *mut ffi_c_void,
        address: *const sockaddr,
    ) -> *mut ffi_c_void;

    pub fn SCNetworkReachabilityGetFlags(target: *mut ffi_c_void, flags: *mut u32) -> bool;

    pub fn CFRelease(cf: *mut ffi_c_void);
}

//------------------------------------------------------------------------------
// Helper methods for working with the bindings
//------------------------------------------------------------------------------

/// Extract the process name from a kinfo_proc structure
pub fn extract_proc_name(proc_info: &kinfo_proc) -> String {
    let raw_name = &proc_info.kp_eproc.p_comm;
    let end = raw_name.iter().position(|&c| c == 0).unwrap_or(raw_name.len());
    let name_slice = &raw_name[0..end];
    String::from_utf8_lossy(name_slice).to_string()
}

/// Check if a process is a system process based on PID and other heuristics
pub fn is_system_process(pid: u32, name: &str) -> bool {
    // On macOS, system processes typically:
    // 1. Have a PID < 1000
    // 2. Run as root (uid 0) - this would need additional privileges to check
    // 3. Are owned by system users
    // 4. Have names that start with "com.apple." or are well-known system process names

    pid < 1000
        || name.starts_with("com.apple.")
        || ["launchd", "kernel_task", "WindowServer", "systemstats", "logd", "syslogd"]
            .contains(&name)
}

/// Convert a char array to an SMC key integer
pub fn smc_key_from_chars(key: [c_char; 4]) -> u32 {
    let mut result: u32 = 0;
    for &k in &key {
        result = (result << 8) | (k as u8 as u32);
    }
    result
}

/// Get network interface statistics using sysctlbyname
///
/// This function retrieves the 64-bit network statistics for a given interface
/// using the sysctlbyname API, which provides direct access to kernel variables.
///
/// # Arguments
///
/// * `interface_name` - The name of the network interface (e.g., "en0", "lo0")
///
/// # Returns
///
/// On success, returns the interface data containing traffic statistics.
/// On failure, returns an error describing what went wrong.
///
/// # Example
///
/// ```no_run
/// use darwin_metrics::utils::bindings;
///
/// let stats = bindings::get_network_stats_native("en0").unwrap();
/// println!("Bytes received: {}", stats.ifi_ibytes);
/// println!("Bytes sent: {}", stats.ifi_obytes);
/// ```
pub fn get_network_stats_native(interface_name: &str) -> crate::error::Result<if_data64> {
    use crate::error::Error;
    use std::{ffi::CString, mem, ptr};

    // Format the sysctlbyname key
    let sysctl_key = format!("net.link.generic.system.ifdata.{}", interface_name);
    let c_sysctl_key = CString::new(sysctl_key).map_err(|_| {
        Error::Network(format!(
            "Failed to create sysctlbyname key for interface '{}'",
            interface_name
        ))
    })?;

    // Initialize output variables
    let mut if_data_64: if_data64 = unsafe { mem::zeroed() };
    let mut size = mem::size_of::<if_data64>();

    // Call sysctlbyname
    let result = unsafe {
        sysctlbyname(
            c_sysctl_key.as_ptr(),
            &mut if_data_64 as *mut _ as *mut c_void,
            &mut size,
            ptr::null(),
            0,
        )
    };

    if result != 0 {
        return Err(Error::Network(format!(
            "Failed to get network stats for interface '{}': errno={}",
            interface_name,
            std::io::Error::last_os_error()
        )));
    }

    Ok(if_data_64)
}
