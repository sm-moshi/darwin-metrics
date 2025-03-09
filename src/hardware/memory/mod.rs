//! Memory analysis and monitoring module
//!
//! This module provides comprehensive memory metrics and monitoring capabilities for macOS systems.
//! It tracks system memory usage, page states, memory pressure, and swap activity.
//!
//! # Features
//!
//! - System memory metrics (total, available, used, wired)
//! - Detailed page states (active, inactive, wired, free, compressed)
//! - Memory pressure monitoring with configurable thresholds
//! - Swap usage tracking with activity rates
//! - Asynchronous memory monitoring capabilities
//! - Memory pressure callbacks for event-driven monitoring
//!
//! # Examples
//!
//! Basic memory information:
//!
//! ```no_run
//! use darwin_metrics::hardware::memory::Memory;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let memory = Memory::new()?;
//!     
//!     println!("Total Memory: {} bytes", memory.total);
//!     println!("Available Memory: {} bytes", memory.available);
//!     println!("Used Memory: {} bytes", memory.used);
//!     println!("Memory Usage: {:.1}%", memory.usage_percentage());
//!     println!("Memory Pressure: {:.1}%", memory.pressure_percentage());
//!     
//!     Ok(())
//! }
//! ```
//!
//! Monitoring memory pressure changes:
//!
//! ```no_run
//! use darwin_metrics::hardware::memory::{Memory, PressureLevel};
//! 
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let memory = Memory::new()?;
//!     
//!     // Register a callback for memory pressure changes
//!     memory.on_pressure_change(|level| {
//!         match level {
//!             PressureLevel::Normal => println!("Memory pressure is NORMAL"),
//!             PressureLevel::Warning => println!("Memory pressure is HIGH"),
//!             PressureLevel::Critical => println!("Memory pressure is CRITICAL"),
//!         }
//!     });
//!     
//!     Ok(())
//! }
//! ```
//!
//! For more examples, see the `examples/memory_monitor.rs` and `examples/memory_monitor_async.rs` files.

use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use crate::{
    error::{Error, Result},
    hardware::iokit::{IOKit, IOKitImpl},
    utils::bindings::{
        host_statistics64, mach_host_self, sysctl,
        sysctl_constants::{CTL_HW, CTL_VM, HW_MEMSIZE, VM_SWAPUSAGE},
        vm_kernel_page_size, vm_statistics64, xsw_usage, HostInfoT, HOST_VM_INFO64,
        HOST_VM_INFO64_COUNT, KERN_SUCCESS,
    },
};

/// Memory pressure level indicator
///
/// Used to report the current memory pressure state of the system.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum PressureLevel {
    /// Normal memory pressure - sufficient memory available
    Normal,
    /// Warning level memory pressure - memory is becoming constrained
    Warning,
    /// Critical memory pressure - system is under severe memory constraints
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

/// Detailed memory page states
///
/// Provides a breakdown of how memory pages are being used in the system.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct PageStates {
    /// Memory pages actively in use
    pub active: u64,
    /// Memory pages that haven't been accessed recently but still in RAM
    pub inactive: u64,
    /// Memory pages that cannot be paged out (kernel and other critical components)
    pub wired: u64,
    /// Memory pages immediately available for allocation
    pub free: u64,
    /// Memory pages that have been compressed to save physical RAM
    pub compressed: u64,
}

/// Swap file usage and activity metrics
///
/// Tracks swap space utilization and activity rates.
#[derive(Debug, PartialEq, Clone)]
pub struct SwapUsage {
    /// Total swap space in bytes
    pub total: u64,
    /// Used swap space in bytes
    pub used: u64,
    /// Available swap space in bytes
    pub free: u64,
    /// Rate of data being swapped in (pages/sec)
    pub ins: f64,
    /// Rate of data being swapped out (pages/sec)
    pub outs: f64,
    /// Swap utilization as a percentage (0.0-1.0)
    pub pressure: f64,
}

impl Default for SwapUsage {
    fn default() -> Self {
        Self { total: 0, used: 0, free: 0, ins: 0.0, outs: 0.0, pressure: 0.0 }
    }
}

/// Type definition for memory pressure callback functions
pub type PressureCallback = Box<dyn Fn(PressureLevel) + Send + Sync>;

/// Main memory monitoring and analysis interface
///
/// Provides comprehensive memory metrics and monitoring capabilities.
pub struct Memory {
    /// Total physical memory in bytes
    pub total: u64,
    /// Available memory in bytes (free + inactive)
    pub available: u64,
    /// Used memory in bytes (total - available)
    pub used: u64,
    /// Wired memory in bytes (cannot be paged out)
    pub wired: u64,
    /// Memory pressure value between 0.0-1.0
    pub pressure: f64,
    /// Detailed memory page states
    pub page_states: PageStates,
    /// Swap usage and activity metrics
    pub swap_usage: SwapUsage,
    /// History of memory usage percentages
    history: VecDeque<f64>,
    /// Maximum number of history items to keep
    history_max_items: usize,
    /// Threshold for warning memory pressure (0.0-1.0)
    pressure_warning_threshold: f64,
    /// Threshold for critical memory pressure (0.0-1.0)
    pressure_critical_threshold: f64,
    /// Registered callbacks for memory pressure changes
    pressure_callbacks: Arc<Mutex<Vec<PressureCallback>>>,
    /// Timestamp of last update
    last_update: Instant,
    /// Previous swap-in operations count for rate calculation
    prev_swap_in: u64,
    /// Previous swap-out operations count for rate calculation
    prev_swap_out: u64,
    /// IOKit interface for hardware access
    iokit: Option<Box<dyn IOKit>>,
}

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
            .field("pressure_warning_threshold", &self.pressure_warning_threshold)
            .field("pressure_critical_threshold", &self.pressure_critical_threshold)
            .field("pressure_callbacks", &format!("<{} callbacks>", callback_count))
            .field("last_update", &self.last_update)
            .field("prev_swap_in", &self.prev_swap_in)
            .field("prev_swap_out", &self.prev_swap_out)
            .field("iokit", &if self.iokit.is_some() { "Some(IOKit)" } else { "None" })
            .finish()
    }
}

impl Clone for Memory {
    fn clone(&self) -> Self {
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
            pressure_callbacks: Arc::new(Mutex::new(Vec::new())),
            last_update: self.last_update,
            prev_swap_in: self.prev_swap_in,
            prev_swap_out: self.prev_swap_out,
            iokit: None,
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
            && self.history == other.history
            && self.history_max_items == other.history_max_items
            && (self.pressure_warning_threshold - other.pressure_warning_threshold).abs()
                < f64::EPSILON
            && (self.pressure_critical_threshold - other.pressure_critical_threshold).abs()
                < f64::EPSILON
            && self.last_update == other.last_update
            && self.prev_swap_in == other.prev_swap_in
            && self.prev_swap_out == other.prev_swap_out
    }
}

impl Memory {
    /// Create a new Memory monitoring instance
    ///
    /// Initializes a Memory struct with default settings and performs an initial update.
    /// 
    /// # Returns
    /// 
    /// A Result containing the initialized Memory struct, or an Error if initialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use darwin_metrics::hardware::memory::Memory;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let memory = Memory::new()?;
    ///     println!("Total memory: {} bytes", memory.total);
    ///     Ok(())
    /// }
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
            PageStates { active: 0, inactive: 0, wired, free: available, compressed: 0 },
            SwapUsage::default(),
        )
    }

    /// Update memory metrics
    ///
    /// Refreshes all memory metrics by querying the system for current memory usage information.
    /// This method should be called periodically to get up-to-date memory statistics.
    ///
    /// # Returns
    ///
    /// A Result that is Ok if the update succeeded, or an Error if it failed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use darwin_metrics::hardware::memory::Memory;
    /// use std::thread::sleep;
    /// use std::time::Duration;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut memory = Memory::new()?;
    ///     
    ///     // Initial reading
    ///     println!("Initial memory usage: {:.1}%", memory.usage_percentage());
    ///     
    ///     // Wait a moment then update
    ///     sleep(Duration::from_secs(5));
    ///     memory.update()?;
    ///     
    ///     // Get updated reading
    ///     println!("Updated memory usage: {:.1}%", memory.usage_percentage());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn update(&mut self) -> Result<()> {
        self.total = Self::get_total_memory()?;

        let vmstat = Self::get_vm_statistics()?;

        let page_size = Self::get_page_size()?;

        self.page_states.free = vmstat.free_count as u64 * page_size;
        self.page_states.active = vmstat.active_count as u64 * page_size;
        self.page_states.inactive = vmstat.inactive_count as u64 * page_size;
        self.page_states.wired = vmstat.wire_count as u64 * page_size;
        self.page_states.compressed = vmstat.compressor_page_count as u64 * page_size;

        self.available = self.page_states.free + self.page_states.inactive;
        self.used = self.total - self.available;
        self.wired = self.page_states.wired;

        self.pressure = 1.0 - (self.available as f64 / self.total as f64).clamp(0.0, 1.0);

        let mut swap = Self::get_swap_usage()?;

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

        swap.pressure = if swap.total > 0 {
            (swap.used as f64 / swap.total as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };

        self.swap_usage = swap;

        self.prev_swap_in = vmstat.swapins;
        self.prev_swap_out = vmstat.swapouts;
        self.last_update = now;

        self.history.push_back(self.usage_percentage());
        if self.history.len() > self.history_max_items {
            self.history.pop_front();
        }

        self.check_pressure_thresholds();

        Ok(())
    }

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

    pub fn usage_percentage(&self) -> f64 {
        (self.used as f64 / self.total as f64 * 100.0).clamp(0.0, 100.0)
    }

    pub fn pressure_percentage(&self) -> f64 {
        (self.pressure * 100.0).clamp(0.0, 100.0)
    }

    pub fn pressure_level(&self) -> PressureLevel {
        if self.pressure >= self.pressure_critical_threshold {
            PressureLevel::Critical
        } else if self.pressure >= self.pressure_warning_threshold {
            PressureLevel::Warning
        } else {
            PressureLevel::Normal
        }
    }

    pub fn set_pressure_thresholds(&mut self, warning: f64, critical: f64) -> Result<()> {
        if !(0.0..=1.0).contains(&warning) || !(0.0..=1.0).contains(&critical) || warning > critical
        {
            return Err(Error::invalid_data(
                "Invalid pressure thresholds: must be between 0 and 1, and warning must be less \
                 than critical",
            ));
        }
        self.pressure_warning_threshold = warning;
        self.pressure_critical_threshold = critical;
        Ok(())
    }

    pub fn on_pressure_change<F>(&self, callback: F)
    where
        F: Fn(PressureLevel) + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = self.pressure_callbacks.lock() {
            callbacks.push(Box::new(callback));
        }
    }

    pub fn usage_history(&self) -> &VecDeque<f64> {
        &self.history
    }

    pub async fn start_monitoring(&self, interval_ms: u64) -> Result<MemoryMonitorHandle> {
        let callbacks = self.pressure_callbacks.clone();
        let warning_threshold = self.pressure_warning_threshold;
        let critical_threshold = self.pressure_critical_threshold;
        let active = Arc::new(AtomicBool::new(true));
        let handle_active = active.clone();

        tokio::spawn(async move {
            let mut prev_level = None;

            while handle_active.load(Ordering::SeqCst) {
                if let Ok(memory) = Self::get_info() {
                    let current_level = if memory.pressure >= critical_threshold {
                        PressureLevel::Critical
                    } else if memory.pressure >= warning_threshold {
                        PressureLevel::Warning
                    } else {
                        PressureLevel::Normal
                    };

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
            Err(Error::system(format!("Failed to get total memory: {}", result)))
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
            return Err(Error::system(format!("Failed to get VM statistics: {}", kern_result)));
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

        // If we get an error, return a default SwapUsage instead of failing
        // This is more resilient in test environments or systems without swap
        if result != 0 {
            // Log the error but don't fail - this is often expected in test environments
            eprintln!("Warning: Failed to get swap usage, using defaults (error: {})", result);
            return Ok(SwapUsage::default());
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

pub struct MemoryMonitorHandle {
    active: Arc<AtomicBool>,
}

impl MemoryMonitorHandle {
    pub fn stop(&self) {
        self.active.store(false, Ordering::SeqCst);
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}

impl Drop for MemoryMonitorHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Async wrapper for the memory module
impl Memory {
    /// Update memory metrics asynchronously
    ///
    /// This method offloads the blocking system calls to a separate thread
    /// using tokio's spawn_blocking, making it suitable for use in async contexts
    /// without blocking the async runtime.
    ///
    /// # Returns
    ///
    /// A Result that is Ok if the update succeeded, or an Error if it failed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use darwin_metrics::hardware::memory::Memory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut memory = Memory::new()?;
    ///     
    ///     // Update memory metrics asynchronously
    ///     memory.update_async().await?;
    ///     println!("Memory usage: {:.1}%", memory.usage_percentage());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn update_async(&mut self) -> Result<()> {
        // Perform the blocking operation in a separate thread
        // Clone first so it can be moved into the closure
        let clone = self.clone();
        let updated_memory = tokio::task::spawn_blocking(move || {
            let mut memory = clone;
            match memory.update() {
                Ok(()) => Ok(memory),
                Err(e) => Err(e),
            }
        }).await.map_err(|e| Error::system(format!("Failed to update memory metrics: {}", e)))?;
        
        // Update fields from the result if successful
        match updated_memory {
            Ok(memory) => {
                self.total = memory.total;
                self.available = memory.available;
                self.used = memory.used;
                self.wired = memory.wired;
                self.pressure = memory.pressure;
                self.page_states = memory.page_states;
                self.swap_usage = memory.swap_usage;
                self.history = memory.history;
                self.prev_swap_in = memory.prev_swap_in;
                self.prev_swap_out = memory.prev_swap_out;
                self.last_update = memory.last_update;
                Ok(())
            },
            Err(e) => Err(e),
        }
    }
    
    /// Get memory info asynchronously
    ///
    /// Creates a new Memory instance asynchronously without blocking the async runtime.
    ///
    /// # Returns
    ///
    /// A Result containing the initialized Memory struct, or an Error if initialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use darwin_metrics::hardware::memory::Memory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Get memory info asynchronously
    ///     let memory = Memory::get_info_async().await?;
    ///     println!("Memory usage: {:.1}%", memory.usage_percentage());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_info_async() -> Result<Self> {
        tokio::task::spawn_blocking(|| {
            Self::get_info()
        }).await.map_err(|e| Error::system(format!("Failed to get memory info: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_initialization() {
        let memory = Memory::new();
        assert!(memory.is_ok(), "Should be able to initialize Memory");
    }
    
    #[test]
    fn test_memory_update() {
        let mut memory = Memory::new().unwrap();
        let result = memory.update();
        assert!(result.is_ok(), "Update should succeed");
    }
    
    #[test]
    fn test_memory_metrics() {
        let memory = Memory::new().unwrap();
        
        // Basic validations
        assert!(memory.total > 0, "Total memory should be positive");
        assert!(memory.available > 0, "Available memory should be positive");
        assert!(memory.used > 0, "Used memory should be positive");
        assert!(memory.used <= memory.total, "Used memory should not exceed total");
        assert!(memory.pressure >= 0.0 && memory.pressure <= 1.0, "Pressure should be between 0 and 1");
        
        // Page state validations
        assert!(memory.page_states.free > 0, "Free pages should be positive");
        assert!(memory.page_states.active > 0, "Active pages should be positive");
        
        // Swap usage validation - might be 0 on systems without swap
        // For u64 values, they are always >= 0, so no need to test that
        if memory.swap_usage.total > 0 {
            assert!(memory.swap_usage.used <= memory.swap_usage.total, "Used swap should not exceed total");
        }
    }
    
    #[test]
    fn test_usage_percentage() {
        let memory = Memory::new().unwrap();
        let percentage = memory.usage_percentage();
        
        assert!((0.0..=100.0).contains(&percentage),
            "Usage percentage should be between 0 and 100, got {}", percentage);
    }
    
    #[test]
    fn test_pressure_callbacks() {
        let memory = Memory::new().unwrap();
        let pressure_level = Arc::new(Mutex::new(PressureLevel::Normal));
        let pressure_level_clone = pressure_level.clone();
        
        // Add a callback that updates the pressure level
        memory.on_pressure_change(move |level| {
            let mut guard = pressure_level_clone.lock().unwrap();
            *guard = level;
        });
        
        // Now force a check
        memory.check_pressure_thresholds();
        
        // The level should match the current pressure
        let level = memory.pressure_level();
        let callback_level = *pressure_level.lock().unwrap();
        
        assert_eq!(level, callback_level, "Callback pressure level should match current level");
    }
    
    #[test]
    fn test_custom_thresholds() {
        let mut memory = Memory::new().unwrap();
        
        // Set custom thresholds
        let result = memory.set_pressure_thresholds(0.25, 0.75);
        assert!(result.is_ok(), "Setting thresholds should succeed");
        
        // Test invalid thresholds (warning > critical)
        let result = memory.set_pressure_thresholds(0.8, 0.5);
        assert!(result.is_err(), "Setting invalid thresholds should fail");
        
        // Test invalid thresholds (out of range)
        let result = memory.set_pressure_thresholds(-0.1, 0.5);
        assert!(result.is_err(), "Setting out-of-range thresholds should fail");
        
        let result = memory.set_pressure_thresholds(0.3, 1.5);
        assert!(result.is_err(), "Setting out-of-range thresholds should fail");
    }
    
    #[test]
    fn test_memory_info_functions() {
        // Test individual info functions
        let total = Memory::get_total_memory();
        assert!(total.is_ok(), "Should get total memory");
        assert!(total.unwrap() > 0, "Total memory should be positive");
        
        let page_size = Memory::get_page_size();
        assert!(page_size.is_ok(), "Should get page size");
        assert!(page_size.unwrap() > 0, "Page size should be positive");
        
        let vm_stats = Memory::get_vm_statistics();
        assert!(vm_stats.is_ok(), "Should get VM statistics");
        
        let swap = Memory::get_swap_usage();
        assert!(swap.is_ok(), "Should get swap usage");
    }
    
    #[tokio::test]
    async fn test_memory_monitoring() {
        let memory = Memory::new().unwrap();
        let monitor_handle = memory.start_monitoring(100).await;
        
        assert!(monitor_handle.is_ok(), "Should start monitoring successfully");
        
        let handle = monitor_handle.unwrap();
        assert!(handle.is_active(), "Monitor should be active initially");
        
        // Sleep briefly to allow monitor to run
        tokio::time::sleep(Duration::from_millis(250)).await;
        
        // Stop the monitor
        handle.stop();
        assert!(!handle.is_active(), "Monitor should be inactive after stopping");
    }
    
    #[tokio::test]
    async fn test_update_async() {
        let mut memory = Memory::new().unwrap();
        let result = memory.update_async().await;
        assert!(result.is_ok(), "Async update should succeed");
    }
    
    #[tokio::test]
    async fn test_get_info_async() {
        let memory_result = Memory::get_info_async().await;
        assert!(memory_result.is_ok(), "Async get_info should succeed");
        
        let memory = memory_result.unwrap();
        assert!(memory.total > 0, "Total memory should be positive");
    }
}
