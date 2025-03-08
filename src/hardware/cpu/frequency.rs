use libc;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct FrequencyMetrics {
    pub current: f64,
    pub min: f64,
    pub max: f64,
    pub available: Vec<f64>,
}

#[derive(Debug)]
pub struct FrequencyMonitor;

impl FrequencyMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn get_metrics(&self) -> Result<FrequencyMetrics> {
        let cpu_info = unsafe { get_cpu_info()? };
        Ok(FrequencyMetrics {
            current: cpu_info.current_frequency,
            min: cpu_info.min_frequency,
            max: cpu_info.max_frequency,
            available: cpu_info.available_frequencies,
        })
    }
}

#[derive(Default)]
struct CpuInfo {
    current_frequency: f64,
    min_frequency: f64,
    max_frequency: f64,
    available_frequencies: Vec<f64>,
}

unsafe fn get_cpu_info() -> Result<CpuInfo> {
    let mut cpu_info = CpuInfo::default();

    let mut size = std::mem::size_of::<u64>();
    let mut freq: u64 = 0;
    let result = libc::sysctl(
        [libc::CTL_HW, 0].as_mut_ptr(),
        2,
        &mut freq as *mut _ as *mut libc::c_void,
        &mut size,
        std::ptr::null_mut(),
        0,
    );
    if result == 0 {
        cpu_info.current_frequency = freq as f64 / 1_000_000.0;
    }

    let mut min_freq: u64 = 0;
    let mut max_freq: u64 = 0;
    let result = libc::sysctl(
        [libc::CTL_HW, 0].as_mut_ptr(),
        2,
        &mut min_freq as *mut _ as *mut libc::c_void,
        &mut size,
        std::ptr::null_mut(),
        0,
    );
    if result == 0 {
        cpu_info.min_frequency = min_freq as f64 / 1_000_000.0;
    }

    let result = libc::sysctl(
        [libc::CTL_HW, 0].as_mut_ptr(),
        2,
        &mut max_freq as *mut _ as *mut libc::c_void,
        &mut size,
        std::ptr::null_mut(),
        0,
    );
    if result == 0 {
        cpu_info.max_frequency = max_freq as f64 / 1_000_000.0;
    }

    Ok(cpu_info)
}
