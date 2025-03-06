//! Memory metrics and information for macOS systems.
//!
//! This module provides functionality to gather memory-related metrics and information
//! on macOS systems using the vm_statistics64 API and sysctl calls. It includes:
//! - Physical and virtual memory usage
//! - Memory pressure monitoring
//! - Active, inactive, and compressed memory tracking
//! - Swap usage and activity statistics
//!
//! # Examples
//!
//! ```no_run
//! use darwin_metrics::prelude::*;
//!
//! fn main() -> darwin_metrics::Result<()> {
//!     let memory = Memory::new()?;
//!     println!("Memory usage: {:.1}%", memory.usage_percentage());
//!     println!("Available: {:.2} GB", memory.available as f64 / 1_073_741_824.0);
//!     println!("Memory pressure: {}", memory.pressure_level());
//!     Ok(())
//! }
//! ```

use crate::hardware::iokit::{IOKit, IOKitImpl};
use crate::{Error, Result};
use crate::utils::{objc_utils, property_utils, test_utils};
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

// External C functions from macOS
use std::ffi::c_void;

/// Represents memory pressure levels
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PressureLevel {
    /// Normal memory pressure (0-0.65)
    Normal,
    /// Warning level memory pressure (0.65-0.85)
    Warning,
    /// Critical memory pressure (0.85-1.0)
    Critical,
}

impl std::fmt::Display for PressureLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "Normal"),
            Self::Warning => write!(f, "Warning"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Represents memory page states
#[derive(Debug, PartialEq, Clone, Default)]
pub struct PageStates {
    /// Active memory pages in bytes
    pub active: u64,
    /// Inactive memory pages in bytes
    pub inactive: u64,
    /// Wired memory pages in bytes
    pub wired: u64,
    /// Free memory pages in bytes
    pub free: u64,
    /// Compressed memory in bytes
    pub compressed: u64,
}

/// Represents swap usage information
#[derive(Debug, PartialEq, Clone)]
pub struct SwapUsage {
    /// Total swap space in bytes
    pub total: u64,
    /// Used swap space in bytes
    pub used: u64,
    /// Free swap space in bytes
    pub free: u64,
    /// Swap in operations per second
    pub ins: f64,
    /// Swap out operations per second
    pub outs: f64,
    /// Swap pressure level (0-1)
    pub pressure: f64,
}

impl Default for SwapUsage {
    fn default() -> Self {
        Self {
            total: 0,
            used: 0,
            free: 0,
            ins: 0.0,
            outs: 0.0,
            pressure: 0.0,
        }
    }
}

/// Type definition for memory pressure callback
pub type PressureCallback = Box<dyn Fn(PressureLevel) + Send + Sync>;

/// Represents system memory information
pub struct Memory {
    /// Total physical memory in bytes
    pub total: u64,
    /// Available memory in bytes
    pub available: u64,
    /// Used memory in bytes
    pub used: u64,
    /// Memory used by wired/kernel in bytes
    pub wired: u64,
    /// Memory pressure level (0-1)
    pub pressure: f64,
    /// Detailed page state information
    pub page_states: PageStates,
    /// Swap usage information
    pub swap_usage: SwapUsage,
    /// History of memory usage percentages
    history: VecDeque<f64>,
    /// Maximum history items to keep
    history_max_items: usize,
    /// Pressure threshold for warning level (0-1)
    pressure_warning_threshold: f64,
    /// Pressure threshold for critical level (0-1)
    pressure_critical_threshold: f64,
    /// Registered pressure callbacks
    pressure_callbacks: Arc<Mutex<Vec<PressureCallback>>>,
    /// Last update timestamp
    last_update: Instant,
    /// Previous swap in/out values for rate calculation
    prev_swap_in: u64,
    prev_swap_out: u64,
    /// IOKit interface for hardware access
    iokit: Option<Box<dyn IOKit>>,
}

// Implement Debug manually instead of deriving it
impl std::fmt::Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let callback_count = match self.pressure_callbacks.try_lock() {
            Ok(callbacks) => callbacks.len(),
            Err(_) => 0, // If we can't lock, just report 0
        };

        f.debug_struct("Memory")
            .field("total", &self.total)
            .field("available", &self.available)
            .field("used", &self.used)
            .field("wired", &self.wired)
            .field("pressure", &self.pressure)
            .field("page_states", &self.page_states)
            .field("swap_usage", &self.swap_usage)
            .field("history", &self.history)
            .field("history_max_items", &self.history_max_items)
            .field(
                "pressure_warning_threshold",
                &self.pressure_warning_threshold,
            )
            .field(
                "pressure_critical_threshold",
                &self.pressure_critical_threshold,
            )
            .field(
                "pressure_callbacks",
                &format!("<{} callbacks>", callback_count),
            )
            .field("last_update", &self.last_update)
            .field("prev_swap_in", &self.prev_swap_in)
            .field("prev_swap_out", &self.prev_swap_out)
            .field(
                "iokit",
                &if self.iokit.is_some() {
                    "Some(IOKit)"
                } else {
                    "None"
                },
            )
            .finish()
    }
}

// Implement Clone manually instead of deriving it
impl Clone for Memory {
    fn clone(&self) -> Self {
        // Create a new instance without the unclonable fields
        Self {
            total: self.total,
            available: self.available,
            used: self.used,
            wired: self.wired,
            pressure: self.pressure,
            page_states: self.page_states.clone(),
            swap_usage: self.swap_usage.clone(),
            history: self.history.clone(),
            history_max_items: self.history_max_items,
            pressure_warning_threshold: self.pressure_warning_threshold,
            pressure_critical_threshold: self.pressure_critical_threshold,
            pressure_callbacks: Arc::new(Mutex::new(Vec::new())), // Empty callbacks for the clone
            last_update: self.last_update,
            prev_swap_in: self.prev_swap_in,
            prev_swap_out: self.prev_swap_out,
            iokit: None, // Don't clone the IOKit instance
        }
    }
}

impl PartialEq for Memory {
    fn eq(&self, other: &Self) -> bool {
        self.total == other.total
            && self.available == other.available
            && self.used == other.used
            && self.wired == other.wired
            && self.pressure == other.pressure
            && self.page_states == other.page_states
            && self.swap_usage == other.swap_usage
    }
}

impl Memory {
    /// Creates a new Memory instance and initializes it with system values.
    ///
    /// This function creates a new Memory instance and immediately populates it with
    /// current system metrics using the vm_statistics64 API and sysctl calls.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Memory>` which is:
    /// - `Ok(Memory)` containing the initialized Memory instance
    /// - `Err(Error)` if system metrics cannot be gathered
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use darwin_metrics::prelude::*;
    ///
    /// let memory = Memory::new()?;
    /// println!("Memory usage: {:.1}%", memory.usage_percentage());
    /// # Ok::<(), darwin_metrics::Error>(())
    /// ```
    pub fn new() -> Result<Self> {
        let mut memory = Self {
            total: 0,
            available: 0,
            used: 0,
            wired: 0,
            pressure: 0.0,
            page_states: PageStates::default(),
            swap_usage: SwapUsage::default(),
            history: VecDeque::with_capacity(60), // Store last minute at 1 sec intervals
            history_max_items: 60,
            pressure_warning_threshold: 0.65,
            pressure_critical_threshold: 0.85,
            pressure_callbacks: Arc::new(Mutex::new(Vec::new())),
            last_update: Instant::now(),
            prev_swap_in: 0,
            prev_swap_out: 0,
            iokit: Some(Box::new(IOKitImpl)),
        };

        memory.update()?;
        Ok(memory)
    }

    /// Creates a new Memory instance with the given values
    ///
    /// # Arguments
    /// * `total` - Total physical memory in bytes
    /// * `available` - Available memory in bytes
    /// * `used` - Used memory in bytes
    /// * `wired` - Memory used by wired/kernel in bytes
    /// * `pressure` - Memory pressure level (0-1)
    /// * `page_states` - Detailed page state information
    /// * `swap_usage` - Swap usage information
    pub fn with_values(
        total: u64,
        available: u64,
        used: u64,
        wired: u64,
        pressure: f64,
        page_states: PageStates,
        swap_usage: SwapUsage,
    ) -> Self {
        Self {
            total,
            available,
            used,
            wired,
            pressure,
            page_states,
            swap_usage,
            history: VecDeque::with_capacity(60),
            history_max_items: 60,
            pressure_warning_threshold: 0.65,
            pressure_critical_threshold: 0.85,
            pressure_callbacks: Arc::new(Mutex::new(Vec::new())),
            last_update: Instant::now(),
            prev_swap_in: 0,
            prev_swap_out: 0,
            iokit: None,
        }
    }

    /// Creates a simplified Memory instance with minimal information
    ///
    /// # Arguments
    /// * `total` - Total physical memory in bytes
    /// * `available` - Available memory in bytes
    /// * `used` - Used memory in bytes
    /// * `wired` - Memory used by wired/kernel in bytes
    /// * `pressure` - Memory pressure level (0-1)
    pub fn with_basic_info(
        total: u64,
        available: u64,
        used: u64,
        wired: u64,
        pressure: f64,
    ) -> Self {
        Self::with_values(
            total,
            available,
            used,
            wired,
            pressure,
            PageStates {
                active: 0,
                inactive: 0,
                wired,
                free: available,
                compressed: 0,
            },
            SwapUsage::default(),
        )
    }

    /// Updates memory metrics from the system
    ///
    /// This method refreshes all memory metrics including:
    /// - Physical memory usage
    /// - Page states
    /// - Swap usage
    /// - Memory pressure
    ///
    /// It also updates the memory history and checks pressure thresholds.
    ///
    /// # Returns
    ///
    /// Returns a `Result<()>` which is:
    /// - `Ok(())` if the update was successful
    /// - `Err(Error)` if metrics could not be gathered
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use darwin_metrics::prelude::*;
    ///
    /// let mut memory = Memory::new()?;
    /// // Wait some time...
    /// memory.update()?;
    /// println!("Current memory usage: {:.1}%", memory.usage_percentage());
    /// # Ok::<(), darwin_metrics::Error>(())
    /// ```
    pub fn update(&mut self) -> Result<()> {
        // Get total memory
        self.total = Self::get_total_memory()?;

        // Get VM statistics
        let vmstat = Self::get_vm_statistics()?;

        // Calculate page states
        let page_size = Self::get_page_size()?;

        self.page_states.free = vmstat.free_count as u64 * page_size;
        self.page_states.active = vmstat.active_count as u64 * page_size;
        self.page_states.inactive = vmstat.inactive_count as u64 * page_size;
        self.page_states.wired = vmstat.wire_count as u64 * page_size;
        self.page_states.compressed = vmstat.compressor_page_count as u64 * page_size;

        // Update memory totals
        self.available = self.page_states.free + self.page_states.inactive;
        self.used = self.total - self.available;
        self.wired = self.page_states.wired;

        // Calculate memory pressure
        self.pressure = 1.0 - (self.available as f64 / self.total as f64).clamp(0.0, 1.0);

        // Get swap usage
        let mut swap = Self::get_swap_usage()?;

        // Calculate swap in/out rates
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();

        if elapsed > 0.0 && self.prev_swap_in > 0 && self.prev_swap_out > 0 {
            let swap_in_diff = if vmstat.swapins >= self.prev_swap_in {
                vmstat.swapins - self.prev_swap_in
            } else {
                vmstat.swapins
            };

            let swap_out_diff = if vmstat.swapouts >= self.prev_swap_out {
                vmstat.swapouts - self.prev_swap_out
            } else {
                vmstat.swapouts
            };

            swap.ins = swap_in_diff as f64 / elapsed;
            swap.outs = swap_out_diff as f64 / elapsed;
        }

        // Update swap pressure
        swap.pressure = if swap.total > 0 {
            (swap.used as f64 / swap.total as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };

        self.swap_usage = swap;

        // Save values for next rate calculation
        self.prev_swap_in = vmstat.swapins;
        self.prev_swap_out = vmstat.swapouts;
        self.last_update = now;

        // Add current usage to history
        self.history.push_back(self.usage_percentage());
        if self.history.len() > self.history_max_items {
            self.history.pop_front();
        }

        // Check pressure thresholds and trigger callbacks if needed
        self.check_pressure_thresholds();

        Ok(())
    }

    /// Get current memory information
    ///
    /// # Returns
    /// Returns a `Result` containing memory information or an error if the information
    /// cannot be retrieved.
    ///
    /// # Examples
    /// ```no_run
    /// use darwin_metrics::memory::Memory;
    ///
    /// let memory = Memory::get_info().unwrap();
    /// println!("Memory usage: {:.1}%", memory.usage_percentage());
    /// ```
    pub fn get_info() -> Result<Self> {
        let mut memory = Self {
            total: 0,
            available: 0,
            used: 0,
            wired: 0,
            pressure: 0.0,
            page_states: PageStates::default(),
            swap_usage: SwapUsage::default(),
            history: VecDeque::with_capacity(60),
            history_max_items: 60,
            pressure_warning_threshold: 0.65,
            pressure_critical_threshold: 0.85,
            pressure_callbacks: Arc::new(Mutex::new(Vec::new())),
            last_update: Instant::now(),
            prev_swap_in: 0,
            prev_swap_out: 0,
            iokit: Some(Box::new(IOKitImpl)),
        };

        memory.update()?;
        Ok(memory)
    }

    /// Returns memory usage as a percentage (0-100)
    pub fn usage_percentage(&self) -> f64 {
        (self.used as f64 / self.total as f64 * 100.0).clamp(0.0, 100.0)
    }

    /// Returns memory pressure as a percentage (0-100)
    pub fn pressure_percentage(&self) -> f64 {
        (self.pressure * 100.0).clamp(0.0, 100.0)
    }

    /// Returns current memory pressure level
    pub fn pressure_level(&self) -> PressureLevel {
        if self.pressure >= self.pressure_critical_threshold {
            PressureLevel::Critical
        } else if self.pressure >= self.pressure_warning_threshold {
            PressureLevel::Warning
        } else {
            PressureLevel::Normal
        }
    }

    /// Sets pressure thresholds
    ///
    /// # Arguments
    /// * `warning` - Warning threshold (0-1)
    /// * `critical` - Critical threshold (0-1)
    ///
    /// # Returns
    /// Returns a `Result<()>` which is:
    /// - `Ok(())` if thresholds were set successfully
    /// - `Err(Error)` if thresholds are invalid
    pub fn set_pressure_thresholds(&mut self, warning: f64, critical: f64) -> Result<()> {
        if !(0.0..=1.0).contains(&warning) || !(0.0..=1.0).contains(&critical) || warning > critical
        {
            return Err(Error::invalid_value(
                "Invalid pressure thresholds: must be between 0 and 1, and warning must be less than critical",
            ));
        }

        self.pressure_warning_threshold = warning;
        self.pressure_critical_threshold = critical;

        Ok(())
    }

    /// Register a callback for memory pressure changes
    ///
    /// # Arguments
    /// * `callback` - Function to call when memory pressure level changes
    pub fn on_pressure_change<F>(&self, callback: F)
    where
        F: Fn(PressureLevel) + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = self.pressure_callbacks.lock() {
            callbacks.push(Box::new(callback));
        }
    }

    /// Returns memory usage history
    pub fn usage_history(&self) -> &VecDeque<f64> {
        &self.history
    }

    /// Start monitoring memory in the background
    ///
    /// This method will spawn a background task that periodically updates
    /// memory information and triggers callbacks when pressure levels change.
    ///
    /// # Arguments
    /// * `interval_ms` - Update interval in milliseconds
    ///
    /// # Returns
    /// Returns a `Result<MemoryMonitorHandle>` which is:
    /// - `Ok(handle)` containing a handle that can be used to stop monitoring
    /// - `Err(Error)` if monitoring cannot be started
    pub async fn start_monitoring(&self, interval_ms: u64) -> Result<MemoryMonitorHandle> {
        let callbacks = self.pressure_callbacks.clone();
        let warning_threshold = self.pressure_warning_threshold;
        let critical_threshold = self.pressure_critical_threshold;
        let active = Arc::new(AtomicBool::new(true));
        let handle_active = active.clone();

        // Spawn monitoring task
        tokio::spawn(async move {
            let mut prev_level = None;

            while handle_active.load(Ordering::SeqCst) {
                if let Ok(memory) = Self::get_info() {
                    // Check if pressure level has changed
                    let current_level = if memory.pressure >= critical_threshold {
                        PressureLevel::Critical
                    } else if memory.pressure >= warning_threshold {
                        PressureLevel::Warning
                    } else {
                        PressureLevel::Normal
                    };

                    // Only trigger callbacks if level changed
                    if prev_level != Some(current_level) {
                        if let Ok(callbacks) = callbacks.lock() {
                            for callback in callbacks.iter() {
                                callback(current_level);
                            }
                        }
                        prev_level = Some(current_level);
                    }
                }

                tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            }
        });

        Ok(MemoryMonitorHandle { active })
    }

    // Private helper methods

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
            Err(Error::system_error(format!(
                "Failed to get total memory: {}",
                result
            )))
        }
    }

    fn get_page_size() -> Result<u64> {
        Ok(unsafe { vm_kernel_page_size as u64 })
    }

    fn get_vm_statistics() -> Result<vm_statistics64> {
        let mut info = vm_statistics64::default();
        let mut count = HOST_VM_INFO64_COUNT;

        let kern_result = unsafe {
            host_statistics64(
                mach_host_self(),
                HOST_VM_INFO64,
                (&mut info as *mut vm_statistics64) as HostInfoT,
                &mut count,
            )
        };

        if kern_result != KERN_SUCCESS {
            return Err(Error::system_error(format!(
                "Failed to get VM statistics: {}",
                kern_result
            )));
        }

        Ok(info)
    }

    fn get_swap_usage() -> Result<SwapUsage> {
        let mut xsw_usage = xsw_usage::default();
        let mut size = std::mem::size_of::<xsw_usage>();

        let mib = [CTL_VM, VM_SWAPUSAGE];

        let result = unsafe {
            sysctl(
                mib.as_ptr(),
                mib.len() as u32,
                &mut xsw_usage as *mut xsw_usage as *mut _,
                &mut size,
                std::ptr::null(),
                0,
            )
        };

        if result != 0 {
            return Err(Error::system_error(format!(
                "Failed to get swap usage: {}",
                result
            )));
        }

        Ok(SwapUsage {
            total: xsw_usage.xsu_total,
            used: xsw_usage.xsu_used,
            free: xsw_usage.xsu_avail,
            ins: 0.0,
            outs: 0.0,
            pressure: if xsw_usage.xsu_total > 0 {
                xsw_usage.xsu_used as f64 / xsw_usage.xsu_total as f64
            } else {
                0.0
            },
        })
    }

    fn check_pressure_thresholds(&self) {
        let level = self.pressure_level();

        if let Ok(callbacks) = self.pressure_callbacks.lock() {
            for callback in callbacks.iter() {
                callback(level);
            }
        }
    }
}

/// Handle for memory monitoring
pub struct MemoryMonitorHandle {
    active: Arc<AtomicBool>,
}

impl MemoryMonitorHandle {
    /// Stop memory monitoring
    pub fn stop(&self) {
        self.active.store(false, Ordering::SeqCst);
    }

    /// Check if monitoring is active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}

impl Drop for MemoryMonitorHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

// Constants and FFI definitions for macOS VM statistics
const CTL_HW: i32 = 6;
const HW_MEMSIZE: i32 = 24;
const CTL_VM: i32 = 2;
const VM_SWAPUSAGE: i32 = 5;
const KERN_SUCCESS: i32 = 0;
const HOST_VM_INFO64: i32 = 4;
const HOST_VM_INFO64_COUNT: u32 = 38;

// Fix type names to use proper camel case
type HostInfoT = *mut i32;
type MachPortT = u32;

#[repr(C)]
#[derive(Debug, Default)]
struct vm_statistics64 {
    free_count: u32,
    active_count: u32,
    inactive_count: u32,
    wire_count: u32,
    zero_fill_count: u64,
    reactivations: u64,
    pageins: u64,
    pageouts: u64,
    faults: u64,
    cow_faults: u64,
    lookups: u64,
    hits: u64,
    purges: u64,
    purgeable_count: u32,
    speculative_count: u32,
    decompressions: u64,
    compressions: u64,
    swapins: u64,
    swapouts: u64,
    compressor_page_count: u32,
    throttled_count: u32,
    external_page_count: u32,
    internal_page_count: u32,
    total_uncompressed_pages_in_compressor: u64,
}

#[repr(C)]
#[derive(Debug, Default)]
struct xsw_usage {
    xsu_total: u64,
    xsu_used: u64,
    xsu_avail: u64,
}

// Mark the extern block as unsafe
unsafe extern "C" {
    static vm_kernel_page_size: u32;

    fn host_statistics64(
        host_priv: MachPortT,
        flavor: i32,
        host_info_out: HostInfoT,
        host_info_outCnt: *mut u32,
    ) -> i32;

    fn mach_host_self() -> MachPortT;

    fn sysctl(
        name: *const i32,
        namelen: u32,
        oldp: *mut c_void,
        oldlenp: *mut usize,
        newp: *const c_void,
        newlen: usize,
    ) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::{create_mock_iokit, create_test_dictionary};

    #[test]
    fn test_memory_with_basic_info() {
        let memory = Memory::with_basic_info(
            16 * 1024 * 1024 * 1024, // 16GB total
            8 * 1024 * 1024 * 1024,  // 8GB available
            8 * 1024 * 1024 * 1024,  // 8GB used
            2 * 1024 * 1024 * 1024,  // 2GB wired
            0.5,                     // 50% pressure
        );

        assert_eq!(memory.usage_percentage(), 50.0);
        assert_eq!(memory.pressure_percentage(), 50.0);
        assert_eq!(memory.pressure_level(), PressureLevel::Normal);
        assert_eq!(memory.total, 16 * 1024 * 1024 * 1024);
        assert_eq!(memory.available, 8 * 1024 * 1024 * 1024);
        assert_eq!(memory.used, 8 * 1024 * 1024 * 1024);
        assert_eq!(memory.wired, 2 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_memory_with_values() {
        let page_states = PageStates {
            active: 4 * 1024 * 1024 * 1024,
            inactive: 2 * 1024 * 1024 * 1024,
            wired: 2 * 1024 * 1024 * 1024,
            free: 8 * 1024 * 1024 * 1024,
            compressed: 1 * 1024 * 1024 * 1024,
        };

        let swap_usage = SwapUsage {
            total: 8 * 1024 * 1024 * 1024,
            used: 2 * 1024 * 1024 * 1024,
            free: 6 * 1024 * 1024 * 1024,
            ins: 10.5,
            outs: 5.2,
            pressure: 0.25,
        };

        let memory = Memory::with_values(
            16 * 1024 * 1024 * 1024, // 16GB total
            10 * 1024 * 1024 * 1024, // 10GB available
            6 * 1024 * 1024 * 1024,  // 6GB used
            2 * 1024 * 1024 * 1024,  // 2GB wired
            0.4,                     // 40% pressure
            page_states.clone(),
            swap_usage.clone(),
        );

        assert_eq!(memory.usage_percentage(), 37.5);
        assert_eq!(memory.pressure_percentage(), 40.0);
        assert_eq!(memory.page_states, page_states);
        assert_eq!(memory.swap_usage, swap_usage);
    }

    #[test]
    fn test_pressure_thresholds() {
        let mut memory = Memory::with_basic_info(
            16 * 1024 * 1024 * 1024, // 16GB total
            4 * 1024 * 1024 * 1024,  // 4GB available
            12 * 1024 * 1024 * 1024, // 12GB used
            2 * 1024 * 1024 * 1024,  // 2GB wired
            0.7,                     // 70% pressure
        );

        assert_eq!(memory.pressure_level(), PressureLevel::Warning);

        // Set lower thresholds
        memory.set_pressure_thresholds(0.5, 0.8).unwrap();
        assert_eq!(memory.pressure_level(), PressureLevel::Warning);

        // Set higher thresholds
        memory.set_pressure_thresholds(0.8, 0.9).unwrap();
        assert_eq!(memory.pressure_level(), PressureLevel::Normal);

        // Test invalid thresholds
        assert!(memory.set_pressure_thresholds(-0.1, 0.9).is_err());
        assert!(memory.set_pressure_thresholds(0.5, 1.1).is_err());
        assert!(memory.set_pressure_thresholds(0.9, 0.8).is_err());
    }

    #[test]
    fn test_pressure_callbacks() {
        let memory = Memory::with_basic_info(
            16 * 1024 * 1024 * 1024, // 16GB total
            4 * 1024 * 1024 * 1024,  // 4GB available
            12 * 1024 * 1024 * 1024, // 12GB used
            2 * 1024 * 1024 * 1024,  // 2GB wired
            0.9,                     // 90% pressure
        );

        let callback_counter = Arc::new(AtomicI32::new(0));
        let critical_counter = Arc::new(AtomicI32::new(0));

        let counter_clone = callback_counter.clone();
        let critical_clone = critical_counter.clone();

        // Register callback
        memory.on_pressure_change(move |level| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            if level == PressureLevel::Critical {
                critical_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        // Trigger callbacks
        memory.check_pressure_thresholds();

        assert_eq!(callback_counter.load(Ordering::SeqCst), 1);
        assert_eq!(critical_counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_history_tracking() {
        let mut memory = Memory::with_basic_info(
            16 * 1024 * 1024 * 1024, // 16GB total
            8 * 1024 * 1024 * 1024,  // 8GB available
            8 * 1024 * 1024 * 1024,  // 8GB used
            2 * 1024 * 1024 * 1024,  // 2GB wired
            0.5,                     // 50% pressure
        );

        // Initially empty
        assert_eq!(memory.usage_history().len(), 0);

        // Simulate update
        memory.history.push_back(50.0);
        memory.history.push_back(55.0);
        memory.history.push_back(60.0);

        // Check history
        assert_eq!(memory.usage_history().len(), 3);
        assert_eq!(memory.usage_history()[0], 50.0);
        assert_eq!(memory.usage_history()[1], 55.0);
        assert_eq!(memory.usage_history()[2], 60.0);

        // Test history limit
        let orig_capacity = memory.history_max_items;
        memory.history_max_items = 3;

        // This should push out the first item
        memory.history.push_back(65.0);
        if memory.history.len() > memory.history_max_items {
            memory.history.pop_front();
        }

        assert_eq!(memory.usage_history().len(), 3);
        assert_eq!(memory.usage_history()[0], 55.0);
        assert_eq!(memory.usage_history()[1], 60.0);
        assert_eq!(memory.usage_history()[2], 65.0);

        // Restore original capacity
        memory.history_max_items = orig_capacity;
    }

    #[test]
    fn test_memory_display_formatting() {
        let memory = Memory::with_basic_info(
            16 * 1024 * 1024 * 1024, // 16GB total
            8 * 1024 * 1024 * 1024,  // 8GB available
            8 * 1024 * 1024 * 1024,  // 8GB used
            2 * 1024 * 1024 * 1024,  // 2GB wired
            0.5,                     // 50% pressure
        );

        // Test pressure level formatting
        assert_eq!(format!("{}", memory.pressure_level()), "Normal");

        let warning_memory = Memory::with_basic_info(
            16 * 1024 * 1024 * 1024,
            4 * 1024 * 1024 * 1024,
            12 * 1024 * 1024 * 1024,
            2 * 1024 * 1024 * 1024,
            0.7,
        );
        assert_eq!(format!("{}", warning_memory.pressure_level()), "Warning");

        let critical_memory = Memory::with_basic_info(
            16 * 1024 * 1024 * 1024,
            1 * 1024 * 1024 * 1024,
            15 * 1024 * 1024 * 1024,
            2 * 1024 * 1024 * 1024,
            0.9,
        );
        assert_eq!(format!("{}", critical_memory.pressure_level()), "Critical");
    }

    #[test]
    fn test_memory_edge_cases() {
        // Edge case: 0% memory usage
        let empty_memory = Memory::with_basic_info(
            16 * 1024 * 1024 * 1024, // 16GB total
            16 * 1024 * 1024 * 1024, // 16GB available (impossible, but testing edge case)
            0,                       // 0GB used
            0,                       // 0GB wired
            0.0,                     // 0% pressure
        );

        assert_eq!(empty_memory.usage_percentage(), 0.0);
        assert_eq!(empty_memory.pressure_percentage(), 0.0);
        assert_eq!(empty_memory.pressure_level(), PressureLevel::Normal);

        // Edge case: 100% memory usage
        let full_memory = Memory::with_basic_info(
            16 * 1024 * 1024 * 1024, // 16GB total
            0,                       // 0GB available
            16 * 1024 * 1024 * 1024, // 16GB used
            4 * 1024 * 1024 * 1024,  // 4GB wired
            1.0,                     // 100% pressure
        );

        assert_eq!(full_memory.usage_percentage(), 100.0);
        assert_eq!(full_memory.pressure_percentage(), 100.0);
        assert_eq!(full_memory.pressure_level(), PressureLevel::Critical);
    }

    #[tokio::test]
    async fn test_memory_monitor_handle() {
        let handle = MemoryMonitorHandle {
            active: Arc::new(AtomicBool::new(true)),
        };

        assert!(handle.is_active());
        handle.stop();
        assert!(!handle.is_active());
    }

    #[tokio::test]
    async fn test_memory_monitoring() {
        // Skip this test in CI environments where we can't reliably
        // test real memory monitoring
        if std::env::var("CI").is_ok() {
            return;
        }

        let memory = Memory::with_basic_info(
            16 * 1024 * 1024 * 1024, // 16GB total
            8 * 1024 * 1024 * 1024,  // 8GB available
            8 * 1024 * 1024 * 1024,  // 8GB used
            2 * 1024 * 1024 * 1024,  // 2GB wired
            0.5,                     // 50% pressure
        );

        let callback_counter = Arc::new(AtomicU64::new(0));
        let counter_clone = callback_counter.clone();

        // Register callback
        memory.on_pressure_change(move |_level| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Very short interval for testing
        let handle = memory
            .start_monitoring(10)
            .await
            .expect("Failed to start monitoring");

        // Wait briefly to allow some monitoring cycles
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Stop monitoring
        handle.stop();
        assert!(!handle.is_active());

        // We can't reliably assert the exact number of callbacks in this test
        // but we can ensure the monitoring starts and stops properly
    }

    #[test]
    fn test_swap_usage() {
        let swap = SwapUsage {
            total: 8 * 1024 * 1024 * 1024,
            used: 2 * 1024 * 1024 * 1024,
            free: 6 * 1024 * 1024 * 1024,
            ins: 10.5,
            outs: 5.2,
            pressure: 0.25,
        };

        assert_eq!(swap.total, 8 * 1024 * 1024 * 1024);
        assert_eq!(swap.used, 2 * 1024 * 1024 * 1024);
        assert_eq!(swap.free, 6 * 1024 * 1024 * 1024);
        assert_eq!(swap.ins, 10.5);
        assert_eq!(swap.outs, 5.2);
        assert_eq!(swap.pressure, 0.25);

        // Test Default implementation
        let default_swap = SwapUsage::default();
        assert_eq!(default_swap.total, 0);
        assert_eq!(default_swap.used, 0);
        assert_eq!(default_swap.free, 0);
        assert_eq!(default_swap.ins, 0.0);
        assert_eq!(default_swap.outs, 0.0);
        assert_eq!(default_swap.pressure, 0.0);
    }

    #[test]
    fn test_page_states() {
        let pages = PageStates {
            active: 4 * 1024 * 1024 * 1024,
            inactive: 2 * 1024 * 1024 * 1024,
            wired: 2 * 1024 * 1024 * 1024,
            free: 8 * 1024 * 1024 * 1024,
            compressed: 1 * 1024 * 1024 * 1024,
        };

        assert_eq!(pages.active, 4 * 1024 * 1024 * 1024);
        assert_eq!(pages.inactive, 2 * 1024 * 1024 * 1024);
        assert_eq!(pages.wired, 2 * 1024 * 1024 * 1024);
        assert_eq!(pages.free, 8 * 1024 * 1024 * 1024);
        assert_eq!(pages.compressed, 1 * 1024 * 1024 * 1024);

        // Test Default implementation
        let default_pages = PageStates::default();
        assert_eq!(default_pages.active, 0);
        assert_eq!(default_pages.inactive, 0);
        assert_eq!(default_pages.wired, 0);
        assert_eq!(default_pages.free, 0);
        assert_eq!(default_pages.compressed, 0);
    }
}
