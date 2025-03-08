use crate::hardware::iokit::{IOKit, IOKitImpl};
use crate::error::{Error, Result};
use crate::utils::bindings::{
    sysctl, vm_statistics64, xsw_usage, vm_kernel_page_size,
    host_statistics64, mach_host_self, 
    KERN_SUCCESS, HOST_VM_INFO64, HOST_VM_INFO64_COUNT,
    HostInfoT,
    sysctl_constants::{CTL_HW, HW_MEMSIZE, CTL_VM, VM_SWAPUSAGE}
};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PressureLevel {
    Normal,
    Warning,
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

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PageStates {
    pub active: u64,
    pub inactive: u64,
    pub wired: u64,
    pub free: u64,
    pub compressed: u64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SwapUsage {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub ins: f64,
    pub outs: f64,
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

pub type PressureCallback = Box<dyn Fn(PressureLevel) + Send + Sync>;

pub struct Memory {
    pub total: u64,
    pub available: u64,
    pub used: u64,
    pub wired: u64,
    pub pressure: f64,
    pub page_states: PageStates,
    pub swap_usage: SwapUsage,
    history: VecDeque<f64>,
    history_max_items: usize,
    pressure_warning_threshold: f64,
    pressure_critical_threshold: f64,
    pressure_callbacks: Arc<Mutex<Vec<PressureCallback>>>,
    last_update: Instant,
    prev_swap_in: u64,
    prev_swap_out: u64,
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
            && (self.pressure_warning_threshold - other.pressure_warning_threshold).abs() < f64::EPSILON
            && (self.pressure_critical_threshold - other.pressure_critical_threshold).abs() < f64::EPSILON
            && self.last_update == other.last_update
            && self.prev_swap_in == other.prev_swap_in
            && self.prev_swap_out == other.prev_swap_out
    }
}

impl Memory {
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
                "Invalid pressure thresholds: must be between 0 and 1, and warning must be less than critical",
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
            Err(Error::system(format!(
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
            return Err(Error::system(format!(
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
            return Err(Error::system(format!(
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
