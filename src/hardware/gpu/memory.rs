use std::{
    os::raw::c_void, // c_char,
};

use crate::error::Result;
use objc2::{msg_send, rc::autoreleasepool, runtime::AnyObject};

use super::gpu_impl::Gpu;
use crate::utils::bindings::mach_host_self;

/// Holds information about GPU memory usage
#[derive(Debug, Clone, Default)]
pub struct GpuMemoryInfo {
    /// Total GPU memory in bytes
    pub total: u64,
    /// Used GPU memory in bytes
    pub used: u64,
    /// Free GPU memory in bytes
    pub free: u64,
}

impl Gpu {
    /// Estimates GPU memory information
    ///
    /// This method attempts to retrieve memory information for the GPU.
    /// For Apple Silicon GPUs, this is an estimate based on system RAM allocation.
    /// For discrete GPUs, this uses Metal API to get more accurate information.
    pub fn estimate_memory_info(&self) -> Result<GpuMemoryInfo> {
        // Try to get memory info using Metal API first
        if let Some(memory_info) = self.get_memory_stats() {
            return Ok(memory_info);
        }

        // Fallback to system-based estimation
        let characteristics = self.get_characteristics();

        // For Apple Silicon, estimate based on system RAM
        if characteristics.is_apple_silicon {
            let system_ram = self.get_system_ram()?;

            // Apple Silicon GPUs use unified memory architecture
            // The GPU portion varies by model but is typically 30-50% of system RAM
            let gpu_portion = if let Some(chip_info) = self.detect_apple_silicon_chip() {
                if chip_info.contains("M3 Max")
                    || chip_info.contains("M2 Max")
                    || chip_info.contains("M1 Max")
                {
                    0.5 // 50% for Max chips
                } else if chip_info.contains("M3 Pro")
                    || chip_info.contains("M2 Pro")
                    || chip_info.contains("M1 Pro")
                {
                    0.4 // 40% for Pro chips
                } else {
                    0.3 // 30% for base chips
                }
            } else {
                0.3 // Default to 30%
            };

            let total = (system_ram as f64 * gpu_portion) as u64;

            // Estimate usage based on system memory pressure
            let memory_pressure = self.get_memory_pressure()?;
            let used = (total as f64 * memory_pressure) as u64;
            let free = total.saturating_sub(used);

            return Ok(GpuMemoryInfo { total, used, free });
        }

        // For integrated Intel GPUs, use a similar approach but with different ratios
        if characteristics.is_integrated {
            let system_ram = self.get_system_ram()?;

            // Intel integrated GPUs typically get 20-25% of system RAM
            let total = (system_ram as f64 * 0.25) as u64;

            // Estimate usage based on system memory pressure
            let memory_pressure = self.get_memory_pressure()?;
            let used = (total as f64 * memory_pressure) as u64;
            let free = total.saturating_sub(used);

            return Ok(GpuMemoryInfo { total, used, free });
        }

        // For discrete GPUs without Metal API info, provide a conservative estimate
        // This is a fallback and should rarely be used
        Ok(GpuMemoryInfo {
            total: 2 * 1024 * 1024 * 1024, // 2GB as a safe minimum
            used: 1 * 1024 * 1024 * 1024,  // 1GB as a reasonable usage estimate
            free: 1 * 1024 * 1024 * 1024,  // 1GB free
        })
    }

    /// Gets memory statistics using Metal API if available
    fn get_memory_stats(&self) -> Option<GpuMemoryInfo> {
        if let Some(device) = self.get_metal_device() {
            autoreleasepool(|_| unsafe {
                let device_obj: *mut AnyObject = device.cast();

                // Try to get total GPU memory
                let supports_size_selector = msg_send![device_obj, respondsToSelector: objc2::sel!(recommendedMaxWorkingSetSize)];

                if supports_size_selector {
                    let total_memory: u64 = msg_send![device_obj, recommendedMaxWorkingSetSize];

                    if total_memory > 0 {
                        // For most GPUs, we can only get the total memory from Metal
                        // We'll estimate usage based on system memory pressure
                        let memory_pressure = self.get_memory_pressure().unwrap_or(0.5);
                        let used = (total_memory as f64 * memory_pressure) as u64;
                        let free = total_memory.saturating_sub(used);

                        return Some(GpuMemoryInfo { total: total_memory, used, free });
                    }
                }

                None
            })
        } else {
            None
        }
    }

    /// Gets the total system RAM in bytes
    fn get_system_ram(&self) -> Result<u64> {
        unsafe {
            let mut mem_size: u64 = 0;
            let mut size = std::mem::size_of::<u64>();
            let hw_memsize_cstring = std::ffi::CString::new("hw.memsize").unwrap();

            let result = libc::sysctlbyname(
                hw_memsize_cstring.as_ptr(),
                &mut mem_size as *mut u64 as *mut c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            );

            if result != 0 || mem_size == 0 {
                return Ok(16 * 1024 * 1024 * 1024); // Fallback to 16GB if sysctl fails
            }

            Ok(mem_size)
        }
    }

    /// Gets the current memory pressure as a value between 0.0 and 1.0
    fn get_memory_pressure(&self) -> Result<f64> {
        // Get VM statistics to estimate memory pressure
        unsafe {
            // Use mach_vm_statistics64 to get memory stats
            let port = mach_host_self();
            let mut stats = std::mem::zeroed::<libc::vm_statistics64>();
            let mut count = libc::HOST_VM_INFO64_COUNT;

            let kr = libc::host_statistics64(
                port,
                libc::HOST_VM_INFO64,
                &mut stats as *mut libc::vm_statistics64 as *mut i32,
                &mut count,
            );

            if kr != libc::KERN_SUCCESS {
                return Ok(0.5); // Default to 50% pressure if stats unavailable
            }

            // Calculate memory pressure based on free vs. active/inactive pages
            let free_count = stats.free_count as f64;
            let active_count = stats.active_count as f64;
            let inactive_count = stats.inactive_count as f64;
            let wire_count = stats.wire_count as f64;

            let total_pages = free_count + active_count + inactive_count + wire_count;

            if total_pages > 0.0 {
                let pressure = (active_count + wire_count) / total_pages;
                Ok(pressure.min(1.0).max(0.0))
            } else {
                Ok(0.5) // Default to 50% pressure
            }
        }
    }
}
