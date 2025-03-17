//! FFI bindings to macOS system APIs.
//!
//! This module centralizes all the FFI bindings for macOS system APIs used throughout the crate.

#![allow(non_camel_case_types)]

use libc::mach_port_t;
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

    /// Kernel-related sysctl category
    pub const CTL_KERN: c_int = 1;
    /// Hardware-related sysctl category
    pub const CTL_HW: c_int = 6;
    /// Virtual memory sysctl category
    pub const CTL_VM: c_int = 2;

    /// Kernel process information
    pub const KERN_PROC: c_int = 14;
    /// System hostname
    pub const KERN_HOSTNAME: c_int = 10;
    /// All processes
    pub const KERN_PROC_ALL: c_int = 0;
    /// Process by PID
    pub const KERN_PROC_PID: c_int = 1;
    /// Process by process group
    pub const KERN_PROC_PGRP: c_int = 2;
    /// Process by TTY
    pub const KERN_PROC_TTY: c_int = 3;
    /// Process by UID
    pub const KERN_PROC_UID: c_int = 4;
    /// Process by RUID
    pub const KERN_PROC_RUID: c_int = 5;

    /// Machine hardware type
    pub const HW_MACHINE: c_int = 1;
    /// Physical memory size
    pub const HW_MEMSIZE: c_int = 24;
    /// Number of logical CPUs
    pub const HW_NCPU: c_int = 3;
    /// Number of logical CPUs
    pub const HW_LOGICALCPU: c_int = 103;
    /// Number of physical CPUs
    pub const HW_PHYSICALCPU: c_int = 104;

    /// System boot time
    pub const KERN_BOOTTIME: c_int = 21;
    /// OS release
    pub const KERN_OSRELEASE: c_int = 2;
    /// OS version
    pub const KERN_OSVERSION: c_int = 65;

    /// Swap usage information
    pub const VM_SWAPUSAGE: c_int = 5;
}

/// Process information structure from sysctl/kern_proc.h
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct kinfo_proc {
    /// Basic process information
    pub kp_proc: proc_info,
    /// Extended process information
    pub kp_eproc: extern_proc,
}

/// Basic process information structure
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct proc_info {
    /// Process flags
    pub p_flag: c_int,
    /// Process ID
    pub p_pid: c_int,
    /// Parent process ID
    pub p_ppid: c_int,
    /// Process state
    pub p_stat: c_int,
}

/// Extended process information structure
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct extern_proc {
    /// Process start time
    pub p_starttime: timeval,
    /// Process name (MAXCOMLEN)
    pub p_comm: [u8; 16],
}

/// Time value structure used in BSD APIs
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct timeval {
    /// Seconds
    pub tv_sec: i64,
    /// Microseconds
    pub tv_usec: i32,
}

//------------------------------------------------------------------------------
// External C functions (sysctl, Mach, IOKit)
//------------------------------------------------------------------------------

// sysctl function for system information
#[link(name = "System", kind = "framework")]
extern "C" {
    /// Get or set system information
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

/// Successful Mach operation
pub const KERN_SUCCESS: i32 = 0;
/// Host VM info version 64
pub const HOST_VM_INFO64: i32 = 4;
/// Count of VM info fields
pub const HOST_VM_INFO64_COUNT: u32 = 38;

/// Host info type
pub type HostInfoT = *mut i32;
/// Mach port type
pub type MachPortT = u32;

/// Process state constants
pub mod process_state {
    /// Process being created by fork
    pub const P_SIDL: u8 = 1;
    /// Running
    pub const P_SRUN: u8 = 2;
    /// Sleeping on an address
    pub const P_SSLEEP: u8 = 3;
    /// Process debugging or suspension
    pub const P_SSTOP: u8 = 4;
    /// Awaiting collection by parent
    pub const P_SZOMB: u8 = 5;
}

/// Filesystem statistics structure
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Statfs {
    /// Fundamental file system block size
    pub f_bsize: u32,
    /// Optimal transfer block size
    pub f_iosize: i32,
    /// Total data blocks in file system
    pub f_blocks: u64,
    /// Free blocks in file system
    pub f_bfree: u64,
    /// Free blocks available to non-superuser
    pub f_bavail: u64,
    /// Total file nodes in file system
    pub f_files: u64,
    /// Free nodes available
    pub f_ffree: u64,
    /// File system ID
    pub f_fsid: [i32; 2],
    /// User ID of mount owner
    pub f_owner: u32,
    /// Type of file system
    pub f_type: u32,
    /// Copy of mount flags
    pub f_flags: u32,
    /// File system subtype
    pub f_fssubtype: u32,
    /// File system type name
    pub f_fstypename: [c_char; 16],
    /// Mount point
    pub f_mntonname: [c_char; 1024],
    /// Mount source
    pub f_mntfromname: [c_char; 1024],
    /// Reserved for future use
    pub f_reserved: [u32; 8],
}

/// Network interface address structure
#[repr(C)]
pub struct ifaddrs {
    /// Next interface in list
    pub ifa_next: *mut ifaddrs,
    /// Interface name
    pub ifa_name: *mut c_char,
    /// Interface flags
    pub ifa_flags: u32,
    /// Interface address
    pub ifa_addr: *mut sockaddr,
    /// Interface netmask
    pub ifa_netmask: *mut sockaddr,
    /// Point-to-point destination address
    pub ifa_dstaddr: *mut sockaddr,
    /// Interface-specific data
    pub ifa_data: *mut c_void,
}

/// Socket address structure
#[repr(C)]
pub struct sockaddr {
    /// Length of structure
    pub sa_len: u8,
    /// Address family
    pub sa_family: u8,
    /// Protocol-specific address
    pub sa_data: [c_char; 14],
}

/// IPv4 socket address structure
#[repr(C)]
pub struct sockaddr_in {
    /// Length of structure
    pub sin_len: u8,
    /// Address family (AF_INET)
    pub sin_family: u8,
    /// Port number
    pub sin_port: u16,
    /// IPv4 address
    pub sin_addr: in_addr,
    /// Padding
    pub sin_zero: [c_char; 8],
}

/// IPv4 address structure
#[repr(C)]
pub struct in_addr {
    /// IPv4 address in network byte order
    pub s_addr: u32,
}

/// IPv6 socket address structure
#[repr(C)]
pub struct sockaddr_in6 {
    /// Length of structure
    pub sin6_len: u8,
    /// Address family (AF_INET6)
    pub sin6_family: u8,
    /// Port number
    pub sin6_port: u16,
    /// Flow information
    pub sin6_flowinfo: u32,
    /// IPv6 address
    pub sin6_addr: in6_addr,
    /// Scope ID
    pub sin6_scope_id: u32,
}

/// IPv6 address structure
#[repr(C)]
pub struct in6_addr {
    /// IPv6 address
    pub s6_addr: [u8; 16],
}

/// Link-layer socket address structure
#[repr(C)]
pub struct sockaddr_dl {
    /// Length of structure
    pub sdl_len: u8,
    /// Address family (AF_LINK)
    pub sdl_family: u8,
    /// Link-layer interface index
    pub sdl_index: u16,
    /// Interface type
    pub sdl_type: u8,
    /// Name length
    pub sdl_nlen: u8,
    /// Address length
    pub sdl_alen: u8,
    /// Selector length
    pub sdl_slen: u8,
    /// Link-layer address and selector
    pub sdl_data: [c_char; 12],
}

/// Network interface statistics structure (32-bit)
#[repr(C)]
pub struct if_data {
    /// Type of interface (ethernet, loopback, etc.)
    pub ifi_type: u8,
    /// Physical port/connector type
    pub ifi_physical: u8,
    /// Media address length
    pub ifi_addrlen: u8,
    /// Media header length
    pub ifi_hdrlen: u8,
    /// Receive quota (obsolete)
    pub ifi_recvquota: u8,
    /// Transmit quota (obsolete)
    pub ifi_xmitquota: u8,
    /// Unused padding
    pub ifi_unused1: u8,
    /// Maximum transmission unit
    pub ifi_mtu: u32,
    /// Routing metric
    pub ifi_metric: u32,
    /// Linespeed
    pub ifi_baudrate: u32,
    /// Packets received on interface
    pub ifi_ipackets: u32,
    /// Input errors on interface
    pub ifi_ierrors: u32,
    /// Packets sent on interface
    pub ifi_opackets: u32,
    /// Output errors on interface
    pub ifi_oerrors: u32,
    /// Collisions on csma interfaces
    pub ifi_collisions: u32,
    /// Total number of bytes received
    pub ifi_ibytes: u32,
    /// Total number of bytes sent
    pub ifi_obytes: u32,
    /// Multicast packets received
    pub ifi_imcasts: u32,
    /// Multicast packets sent
    pub ifi_omcasts: u32,
    /// Dropped on input, this interface
    pub ifi_iqdrops: u32,
    /// Destined for unsupported protocol
    pub ifi_noproto: u32,
    /// Receive timing offset (usec)
    pub ifi_recvtiming: u32,
    /// Transmit timing offset (usec)
    pub ifi_xmittiming: u32,
    /// Time of last change
    pub ifi_lastchange: timeval,
}

/// Interface data structure
#[derive(Debug)]
pub struct if_data64 {
    pub ifi_type: u8,
    pub ifi_typelen: u8,
    pub ifi_physical: u8,
    pub ifi_addrlen: u8,
    pub ifi_hdrlen: u8,
    pub ifi_recvquota: u8,
    pub ifi_xmitquota: u8,
    pub ifi_unused1: u8,
    pub ifi_mtu: u32,
    pub ifi_metric: u32,
    pub ifi_baudrate: u64,
    pub ifi_ipackets: u64,
    pub ifi_ierrors: u64,
    pub ifi_opackets: u64,
    pub ifi_oerrors: u64,
    pub ifi_collisions: u64,
    pub ifi_ibytes: u64,
    pub ifi_obytes: u64,
    pub ifi_imcasts: u64,
    pub ifi_omcasts: u64,
    pub ifi_iqdrops: u64,
    pub ifi_noproto: u64,
    pub ifi_recvtiming: u32,
    pub ifi_xmittiming: u32,
    pub ifi_lastchange: timeval,
}

/// Network reachability flags
pub mod reachability_flags {
    /// Connection is transient
    pub const K_SC_NETWORK_REACHABILITY_FLAGS_TRANSIENT_CONNECTION: u32 = 1 << 0;
    /// Target is reachable
    pub const K_SC_NETWORK_REACHABILITY_FLAGS_REACHABLE: u32 = 1 << 1;
    /// Connection is required
    pub const K_SC_NETWORK_REACHABILITY_FLAGS_CONNECTION_REQUIRED: u32 = 1 << 2;
    /// Connection needed when there is traffic
    pub const K_SC_NETWORK_REACHABILITY_FLAGS_CONNECTION_ON_TRAFFIC: u32 = 1 << 3;
    /// User intervention is required
    pub const K_SC_NETWORK_REACHABILITY_FLAGS_INTERVENTION_REQUIRED: u32 = 1 << 4;
    /// Connection needed on demand
    pub const K_SC_NETWORK_REACHABILITY_FLAGS_CONNECTION_ON_DEMAND: u32 = 1 << 5;
    /// Target is a local address
    pub const K_SC_NETWORK_REACHABILITY_FLAGS_IS_LOCAL_ADDRESS: u32 = 1 << 16;
    /// Connection is direct
    pub const K_SC_NETWORK_REACHABILITY_FLAGS_IS_DIRECT: u32 = 1 << 17;
    /// Connection is through cellular network
    pub const K_SC_NETWORK_REACHABILITY_FLAGS_IS_WWAN: u32 = 1 << 18;
}

extern "C" {
    /// Get network interface addresses
    pub fn getifaddrs(ifap: *mut *mut ifaddrs) -> c_int;
    /// Free network interface addresses
    pub fn freeifaddrs(ifp: *mut ifaddrs) -> c_void;

    /// Get system control information by name
    pub fn sysctlbyname(
        name: *const c_char,
        oldp: *mut c_void,
        oldlenp: *mut usize,
        newp: *const c_void,
        newlen: usize,
    ) -> c_int;

    /// Get parent entry in IORegistry
    pub fn IORegistryEntryGetParentEntry(entry: c_uint, plane: *const c_char, parent: *mut c_uint) -> i32;

    /// Create a dynamic store session
    pub fn SCDynamicStoreCreate(
        allocator: *mut ffi_c_void,
        name: *const ffi_c_void,
        callout: *mut ffi_c_void,
        context: *mut ffi_c_void,
    ) -> *mut ffi_c_void;

    /// Copy a value from the dynamic store
    pub fn SCDynamicStoreCopyValue(
        store: *mut ffi_c_void,
        key: *const ffi_c_void,
        value: *mut *mut ffi_c_void,
    ) -> *mut ffi_c_void;

    /// Create a network reachability object for an address
    pub fn SCNetworkReachabilityCreateWithAddress(
        allocator: *mut ffi_c_void,
        address: *const sockaddr,
    ) -> *mut ffi_c_void;

    /// Get network reachability flags
    pub fn SCNetworkReachabilityGetFlags(target: *mut ffi_c_void, flags: *mut u32) -> bool;

    /// Release a Core Foundation object
    pub fn CFRelease(cf: *mut ffi_c_void);
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
    pub fn proc_pidinfo(pid: c_int, flavor: c_int, arg: u64, buffer: *mut c_void, buffersize: c_int) -> c_int;
}

//------------------------------------------------------------------------------
// Network related data structures and bindings
//------------------------------------------------------------------------------

/// Network Address Family constants
pub mod address_family {
    /// Unspecified address family
    pub const AF_UNSPEC: u8 = 0;

    /// IPv4 address family
    pub const AF_INET: u8 = 2;

    /// IPv6 address family
    pub const AF_INET6: u8 = 30;

    /// Link level interface address family
    pub const AF_LINK: u8 = 18;
}

/// Interface Flags constants
pub mod if_flags {
    /// Interface is up
    pub const IFF_UP: u32 = 0x1;

    /// Broadcast address is valid
    pub const IFF_BROADCAST: u32 = 0x2;

    /// Turn on debugging
    pub const IFF_DEBUG: u32 = 0x4;

    /// Is a loopback network
    pub const IFF_LOOPBACK: u32 = 0x8;

    /// Interface is point-to-point link
    pub const IFF_POINTOPOINT: u32 = 0x10;

    /// Resources are allocated
    pub const IFF_RUNNING: u32 = 0x40;

    /// No address resolution protocol
    pub const IFF_NOARP: u32 = 0x80;

    /// Receive all packets
    pub const IFF_PROMISC: u32 = 0x100;

    /// Receive all multicast packets
    pub const IFF_ALLMULTI: u32 = 0x200;

    /// Supports multicast
    pub const IFF_MULTICAST: u32 = 0x8000;

    /// Is a wireless interface
    pub const IFF_WIRELESS: u32 = 0x20;
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
        || ["launchd", "kernel_task", "WindowServer", "systemstats", "logd", "syslogd"].contains(&name)
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
/// This function retrieves the 64-bit network statistics for a given interface using the sysctlbyname API, which
/// provides direct access to kernel variables.
///
/// # Arguments
///
/// * `interface_name` - The name of the network interface (e.g., "en0", "lo0")
///
/// # Returns
///
/// On success, returns the interface data containing traffic statistics. On failure, returns an error describing what
/// went wrong.
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

    // Check for empty interface name
    if interface_name.is_empty() {
        return Err(Error::network_error(
            "get_stats",
            "Failed to get interface data: interface name cannot be empty".to_string(),
        ));
    }

    // Format the sysctlbyname key
    let sysctl_key = format!("net.link.generic.system.ifdata.{}", interface_name);
    let c_sysctl_key = CString::new(sysctl_key).map_err(|e| {
        Error::network_error(
            "sysctlbyname",
            format!("Failed to create sysctlbyname key for interface '{}': {}", interface_name, e),
        )
    })?;

    // Initialize output variables
    let mut if_data_64: if_data64 = unsafe { mem::zeroed() };
    let mut size = mem::size_of::<if_data64>();

    // Call sysctlbyname
    let result = unsafe {
        sysctlbyname(c_sysctl_key.as_ptr(), &mut if_data_64 as *mut _ as *mut c_void, &mut size, ptr::null(), 0)
    };

    if result != 0 {
        return Err(Error::network_error(
            "get_stats",
            format!(
                "Failed to get network stats for interface '{}': errno={}",
                interface_name,
                std::io::Error::last_os_error()
            ),
        ));
    }

    Ok(if_data_64)
}

//------------------------------------------------------------------------------
// Memory and VM structures
//------------------------------------------------------------------------------

/// Virtual memory statistics structure
#[repr(C)]
#[derive(Debug, Default)]
pub struct vm_statistics64 {
    /// Number of free pages
    pub free_count: u32,
    /// Number of active pages
    pub active_count: u32,
    /// Number of inactive pages
    pub inactive_count: u32,
    /// Number of wired pages
    pub wire_count: u32,
    /// Number of zero fill pages
    pub zero_fill_count: u64,
    /// Number of reactivations
    pub reactivations: u64,
    /// Number of pageins
    pub pageins: u64,
    /// Number of pageouts
    pub pageouts: u64,
    /// Number of faults
    pub faults: u64,
    /// Number of copy-on-write faults
    pub cow_faults: u64,
    /// Number of lookups
    pub lookups: u64,
    /// Number of hits
    pub hits: u64,
    /// Number of purges
    pub purges: u64,
    /// Number of purgeable pages
    pub purgeable_count: u32,
    /// Number of speculative pages
    pub speculative_count: u32,
    /// Number of decompressions
    pub decompressions: u64,
    /// Number of compressions
    pub compressions: u64,
    /// Number of swapins
    pub swapins: u64,
    /// Number of swapouts
    pub swapouts: u64,
    /// Number of compressor pages
    pub compressor_page_count: u32,
    /// Number of throttled pages
    pub throttled_count: u32,
    /// Number of external pages
    pub external_page_count: u32,
    /// Number of internal pages
    pub internal_page_count: u32,
    /// Total uncompressed pages in compressor
    pub total_uncompressed_pages_in_compressor: u64,
}

/// Swap usage statistics structure
#[repr(C)]
#[derive(Debug, Default)]
pub struct xsw_usage {
    /// Total swap space
    pub xsu_total: u64,
    /// Used swap space
    pub xsu_used: u64,
    /// Available swap space
    pub xsu_avail: u64,
}

//------------------------------------------------------------------------------
// IOKit Constants and Data Structures
//------------------------------------------------------------------------------

/// IOKit constants for SMC access
pub const KERNEL_INDEX_SMC: u32 = 2;
/// SMC command to read bytes
pub const SMC_CMD_READ_BYTES: u8 = 5;
/// SMC command to read key info
pub const SMC_CMD_READ_KEYINFO: u8 = 9;
/// Successful IOKit operation
pub const IO_RETURN_SUCCESS: i32 = 0;

/// IOKit byte count type
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IOByteCount(pub usize);

/// IOKit option bits type
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IOOptionBits(pub u32);

/// SMC key for CPU temperature
pub const SMC_KEY_CPU_TEMP: [c_char; 4] = [b'T' as c_char, b'C' as c_char, b'0' as c_char, b'P' as c_char];
/// SMC key for GPU temperature
pub const SMC_KEY_GPU_TEMP: [c_char; 4] = [b'T' as c_char, b'G' as c_char, b'0' as c_char, b'P' as c_char];
/// SMC key for number of fans
pub const SMC_KEY_FAN_NUM: [c_char; 4] = [b'F' as c_char, b'N' as c_char, b'u' as c_char, b'm' as c_char];
/// SMC key for fan 0 speed
pub const SMC_KEY_FAN_SPEED: [c_char; 4] = [b'F' as c_char, b'0' as c_char, b'A' as c_char, b'c' as c_char];
/// SMC key for fan 1 speed
pub const SMC_KEY_FAN1_SPEED: [c_char; 4] = [b'F' as c_char, b'1' as c_char, b'A' as c_char, b'c' as c_char];
/// SMC key for fan 0 minimum speed
pub const SMC_KEY_FAN0_MIN: [c_char; 4] = [b'F' as c_char, b'0' as c_char, b'M' as c_char, b'n' as c_char];
/// SMC key for fan 0 maximum speed
pub const SMC_KEY_FAN0_MAX: [c_char; 4] = [b'F' as c_char, b'0' as c_char, b'M' as c_char, b'x' as c_char];
/// SMC key for heatsink temperature
pub const SMC_KEY_HEATSINK_TEMP: [c_char; 4] = [b'T' as c_char, b'h' as c_char, b'0' as c_char, b'H' as c_char];
/// SMC key for ambient temperature
pub const SMC_KEY_AMBIENT_TEMP: [c_char; 4] = [b'T' as c_char, b'A' as c_char, b'0' as c_char, b'P' as c_char];
/// SMC key for battery temperature
pub const SMC_KEY_BATTERY_TEMP: [c_char; 4] = [b'T' as c_char, b'B' as c_char, b'0' as c_char, b'T' as c_char];
/// SMC key for CPU power
pub const SMC_KEY_CPU_POWER: [c_char; 4] = [b'P' as c_char, b'C' as c_char, b'P' as c_char, b'C' as c_char];
/// SMC key for CPU thermal throttling
pub const SMC_KEY_CPU_THROTTLE: [c_char; 4] = [b'P' as c_char, b'C' as c_char, b'T' as c_char, b'C' as c_char];
/// SMC key for package power (SoC)
pub const SMC_KEY_PACKAGE_POWER: [c_char; 4] = [b'P' as c_char, b'M' as c_char, b'P' as c_char, b'0' as c_char];
/// SMC key for GPU power
pub const SMC_KEY_GPU_POWER: [c_char; 4] = [b'P' as c_char, b'G' as c_char, b'P' as c_char, b'G' as c_char];
/// SMC key for DRAM/Memory power
pub const SMC_KEY_DRAM_POWER: [c_char; 4] = [b'P' as c_char, b'D' as c_char, b'R' as c_char, b'P' as c_char];
/// SMC key for Neural Engine power
pub const SMC_KEY_NEURAL_POWER: [c_char; 4] = [b'P' as c_char, b'N' as c_char, b'P' as c_char, b'0' as c_char];

/// SMC version information
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SMCVersion {
    /// Major version
    pub major: u8,
    /// Minor version
    pub minor: u8,
    /// Build version
    pub build: u8,
    /// Reserved byte
    pub reserved: [u8; 1],
    /// Release version
    pub release: u16,
}

/// SMC power limit data
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
#[allow(non_snake_case)]
pub struct SMCPLimitData {
    /// Version
    pub version: u16,
    /// Length
    pub length: u16,
    /// CPU power limit
    pub cpu_plimit: u32,
    /// GPU power limit
    pub gpu_plimit: u32,
    /// Memory power limit
    pub mem_plimit: u32,
}

/// SMC key version data
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SMCKeyData_vers_t {
    /// Version information
    pub version: SMCVersion,
    /// Reserved bytes
    pub reserved: [u8; 16],
}

/// SMC power limit data structure
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SMCKeyData_pLimitData_t {
    /// Power limit data
    pub p_limit_data: SMCPLimitData,
    /// Reserved bytes
    pub reserved: [u8; 10],
}

/// SMC key information
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SMCKeyData_keyInfo_t {
    /// Data size
    pub data_size: u32,
    /// Data type
    pub data_type: [u8; 4],
    /// Data attributes
    pub data_attributes: u8,
}

/// SMC key data union
#[repr(C, packed)]
pub union SMCKeyData_t_data {
    /// Raw bytes
    pub bytes: [u8; 32],
    /// 32-bit unsigned integer
    pub uint32: u32,
    /// 32-bit float
    pub float: f32,
    /// 16-bit signed integer
    pub sint16: i16,
    /// Version data
    pub vers: SMCKeyData_vers_t,
    /// Power limit data
    pub p_limit: SMCKeyData_pLimitData_t,
    /// Key information
    pub key_info: SMCKeyData_keyInfo_t,
}

impl Clone for SMCKeyData_t_data {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for SMCKeyData_t_data {}

/// SMC key data structure
#[repr(C, packed)]
pub struct SMCKeyData_t {
    /// Key
    pub key: u32,
    /// Version
    pub vers: u8,
    /// Power limit data
    pub p_limit_data: u8,
    /// Key info
    pub key_info: u8,
    /// Padding
    pub padding: u8,
    /// Result
    pub result: u8,
    /// Status
    pub status: u8,
    /// 8-bit data
    pub data8: u8,
    /// 32-bit data
    pub data32: u8,
    /// Bytes
    pub bytes: [u8; 2],
    /// Data union
    pub data: SMCKeyData_t_data,
}

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

//------------------------------------------------------------------------------
// Mach host functions
//------------------------------------------------------------------------------

extern "C" {
    /// Kernel page size
    pub static vm_kernel_page_size: u32;

    /// Get host statistics
    pub fn host_statistics64(
        host_priv: MachPortT,
        flavor: i32,
        host_info_out: HostInfoT,
        host_info_outCnt: *mut u32,
    ) -> i32;

    /// Get the current host port
    pub fn mach_host_self() -> MachPortT;
}

//------------------------------------------------------------------------------
// IOKit function declarations
//------------------------------------------------------------------------------

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    /// Get matching service from IOKit registry
    pub fn IOServiceGetMatchingService(masterPort: u32, matchingDictionary: *mut ffi_c_void) -> u32;
    /// Create matching dictionary for IOKit service
    pub fn IOServiceMatching(serviceName: *const c_char) -> *mut ffi_c_void;
    /// Open IOKit service
    pub fn IOServiceOpen(service: u32, owningTask: u32, type_: u32, handle: *mut u32) -> i32;
    /// Close IOKit service
    pub fn IOServiceClose(handle: u32) -> i32;
    /// Create CF properties from IOKit registry entry
    pub fn IORegistryEntryCreateCFProperties(
        entry: u32,
        properties: *mut *mut ffi_c_void,
        allocator: *mut ffi_c_void,
        options: u32,
    ) -> i32;

    /// Call IOKit service method with struct parameters
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
// Filesystem functions
//------------------------------------------------------------------------------

/// Don't block for filesystem sync
pub const MNT_NOWAIT: c_int = 2;

#[link(name = "System", kind = "framework")]
extern "C" {
    /// Get statistics about a mounted filesystem
    pub fn statfs(path: *const c_char, buf: *mut Statfs) -> c_int;
    /// Get statistics about all mounted filesystems
    pub fn getfsstat(buf: *mut Statfs, bufsize: c_int, flags: c_int) -> c_int;
}

//------------------------------------------------------------------------------
// IOKit constants and types
//------------------------------------------------------------------------------

/// Default master port for IOKit
pub const K_IOMASTER_PORT_DEFAULT: mach_port_t = 0;
/// Default master port for IOKit (alternative name)
pub const IOMASTER_PORT_DEFAULT: mach_port_t = K_IOMASTER_PORT_DEFAULT;

/// IO object type
pub type io_object_t = u32;
/// IO iterator type
pub type io_iterator_t = io_object_t;
/// IO registry entry type
pub type io_registry_entry_t = io_object_t;
/// IO service type
pub type io_service_t = io_object_t;

/// IOKit service matching dictionary keys
pub mod io_service_keys {
    /// Key for matching service class
    pub const K_IOPROVIDER_CLASS_KEY: &str = "IOProviderClass";
    /// Key for matching service name
    pub const K_IONAME_MATCH_KEY: &str = "IONameMatch";
}

/// IOKit return codes
pub mod io_return {
    /// Operation completed successfully
    pub const K_IORETURN_SUCCESS: i32 = 0;
    /// General error
    pub const K_IORETURN_ERROR: i32 = 0x2bc;
    /// Operation timed out
    pub const K_IORETURN_TIMEOUT: i32 = 0x2d0;
}

/// SMC key structure for temperature sensors
#[repr(C)]
pub struct SmcKey {
    pub key: [u8; 4],
}

impl SmcKey {
    /// Creates a new SmcKey from an array of 4 characters
    pub fn from_chars(chars: [char; 4]) -> Self {
        let mut key = [0; 4];
        for (i, &c) in chars.iter().enumerate() {
            key[i] = c as u8;
        }
        SmcKey { key }
    }

    /// Creates a new SmcKey from a string
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice that should be exactly 4 characters long
    ///
    /// # Returns
    ///
    /// * `Ok(SmcKey)` - If the string is exactly 4 characters
    /// * `Err(String)` - If the string is not exactly 4 characters
    ///
    /// # Examples
    ///
    /// ```
    /// use darwin_metrics::utils::bindings::SmcKey;
    ///
    /// let key = SmcKey::from_str("TC0P").unwrap();
    /// assert_eq!(key.to_string(), "TC0P");
    /// ```
    pub fn from_str(s: &str) -> Result<Self, String> {
        if s.len() != 4 {
            return Err(format!("SMC key must be exactly 4 characters, got {}", s.len()));
        }

        let chars: Vec<char> = s.chars().collect();
        Ok(Self::from_chars([chars[0], chars[1], chars[2], chars[3]]))
    }

    /// Converts the SmcKey to a string
    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.key).to_string()
    }

    /// Converts the SmcKey to a C-compatible array of characters
    ///
    /// This is useful when interacting with the SMC API which expects
    /// keys in the form of a 4-character array.
    ///
    /// # Returns
    ///
    /// * `[i8; 4]` - A C-compatible array of characters
    ///
    /// # Examples
    ///
    /// ```
    /// use darwin_metrics::utils::bindings::SmcKey;
    ///
    /// let key = SmcKey::from_str("TC0P").unwrap();
    /// let chars = key.to_chars();
    /// assert_eq!(chars, [84, 67, 48, 80]); // ASCII values for 'T', 'C', '0', 'P'
    /// ```
    pub fn to_chars(&self) -> [i8; 4] {
        let mut result = [0; 4];
        for i in 0..4 {
            // Convert u8 to i8 (safe for ASCII values)
            result[i] = self.key[i] as i8;
        }
        result
    }
}

/// SMC value structure
#[repr(C)]
pub struct SmcVal {
    pub data_size: u32,
    pub data_type: [u8; 4],
    pub data_attributes: u8,
    pub data: [u8; 32],
}

/// SMC command to read key
pub const SMC_CMD_READ_KEY: u8 = 5;
/// SMC command to read index
pub const SMC_CMD_READ_INDEX: u8 = 8;
