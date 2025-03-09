# FFI Bindings

The `darwin-metrics` library provides a centralized approach to FFI (Foreign Function Interface) bindings for macOS system APIs. This page explains how these bindings are organized and used throughout the codebase.

## Overview

All low-level FFI bindings are centralized in the `src/utils/bindings.rs` file. This architectural decision provides several benefits:

1. **Maintainability**: Changes to FFI interfaces only need to be made in one place
2. **Consistency**: Prevents duplicate and potentially conflicting definitions
3. **Safety**: Centralizes unsafe code, making auditing easier
4. **Reusability**: Allows sharing of bindings across different modules

## Included Bindings

The bindings module includes interfaces to various macOS frameworks:

### System Framework

```rust,no_run,ignore
// System framework bindings
use std::os::raw::{c_int, c_uint, c_void};

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
```

### IOKit Framework

```rust,no_run,ignore
// IOKit framework bindings
use std::os::raw::c_char;

// A type alias for easier readability
type ffi_c_void = std::os::raw::c_void;

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    // IOService functions
    pub fn IOServiceGetMatchingService(masterPort: u32, matchingDict: *const ffi_c_void) -> u32;
    pub fn IOServiceMatching(serviceName: *const c_char) -> *mut ffi_c_void;
    pub fn IOServiceOpen(service: u32, owningTask: u32, type_: u32, handle: *mut u32) -> i32;
    // ... and more
}
```

### Mach Host Functions

```rust,no_run,ignore
// Mach host functions
// Type definitions for Mach types
type MachPortT = u32;
type HostInfoT = *mut std::os::raw::c_void;

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
```

## Constants and Types

The module also provides centralized definitions for:

- Type aliases for platform-specific types
- Constants used in system API calls
- Data structures for FFI data exchange

## Helper Functions

To make these low-level bindings more usable, helper functions are provided:

```rust,no_run,ignore
// Helper functions for working with low-level macOS APIs
use std::os::raw::c_char;

// Process info structure definition (simplified)
#[repr(C)]
pub struct kinfo_proc {
    // Process basic info
    pub kp_proc: proc_info,
    // Process extra info
    pub kp_eproc: eproc_info,
}

#[repr(C)]
pub struct proc_info {
    // Process ID, state, etc.
    pub p_pid: i32,
    // Other fields...
}

#[repr(C)]
pub struct eproc_info {
    // Process name as a fixed-size array of bytes
    pub p_comm: [u8; 16],
    // Other fields...
}

/// Convert a char array to an SMC key integer
pub fn smc_key_from_chars(key: [c_char; 4]) -> u32 {
    let mut result: u32 = 0;
    for &k in &key {
        result = (result << 8) | (k as u8 as u32);
    }
    result
}

/// Extract the process name from a kinfo_proc structure
pub fn extract_proc_name(proc_info: &kinfo_proc) -> String {
    let raw_name = &proc_info.kp_eproc.p_comm;
    let end = raw_name.iter().position(|&c| c == 0).unwrap_or(raw_name.len());
    let name_slice = &raw_name[0..end];
    String::from_utf8_lossy(name_slice).to_string()
}
```

## Usage Example

Here's how modules use these centralized bindings:

```rust,no_run,ignore
// Example of how to use the centralized bindings in your code
use std::os::raw::{c_int, c_uint, c_void};

// Custom error handling for this example
#[derive(Debug)]
pub enum Error {
    System(String),
    // Other error variants...
}

// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

// Import bindings from the central module
use crate::utils::bindings::{
    sysctl, vm_statistics64, xsw_usage, vm_kernel_page_size,
    host_statistics64, mach_host_self,
    KERN_SUCCESS, HOST_VM_INFO64, HOST_VM_INFO64_COUNT,
    HostInfoT,
    sysctl_constants::{CTL_HW, HW_MEMSIZE, CTL_VM, VM_SWAPUSAGE}
};

// Now you can use these bindings safely
fn get_total_memory() -> Result<u64> {
    let mut size = 0u64;
    let mut size_len = std::mem::size_of::<u64>();
    let mib = [CTL_HW, HW_MEMSIZE];

    let result = unsafe {
        sysctl(
            mib.as_ptr(),
            mib.len() as u32,
            &mut size as *mut u64 as *mut _,
            &mut size_len,
            std::ptr::null(),
            0,
        )
    };

    if result == 0 {
        Ok(size)
    } else {
        Err(Error::System("Failed to get total memory".to_string()))
    }
}
```

## Safety Considerations

While the bindings are centralized, using them still requires care:

1. **Unsafe Blocks**: Always use `unsafe` blocks when calling FFI functions
2. **Error Handling**: Check return values from system calls
3. **Memory Management**: Be careful with memory allocated by system functions
4. **Type Conversions**: Ensure proper conversion between Rust and C types

## Maintaining Bindings

When adding new FFI bindings:

1. Add them to `src/utils/bindings.rs` with proper documentation
2. Group related bindings together with clear section markers
3. Provide helper functions when appropriate
4. Don't expose unsafe interfaces directly in your API

By following this centralized approach, `darwin-metrics` maintains a cleaner, safer, and more maintainable codebase.
