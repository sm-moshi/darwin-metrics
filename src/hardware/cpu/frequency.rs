use libc;
use crate::error::{Error, Result};

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
        fetch_cpu_frequencies()
    }
}

#[derive(Default)]
struct CpuInfo {
    current_frequency: f64,
    min_frequency: f64,
    max_frequency: f64,
    available_frequencies: Vec<f64>,
}

fn fetch_cpu_frequencies() -> Result<FrequencyMetrics> {
    let cpu_info = unsafe { retrieve_cpu_info()? };
    Ok(FrequencyMetrics {
        current: cpu_info.current_frequency,
        min: cpu_info.min_frequency,
        max: cpu_info.max_frequency,
        available: cpu_info.available_frequencies,
    })
}

unsafe fn retrieve_cpu_info() -> Result<CpuInfo> {
    let mut cpu_info = CpuInfo::default();
    let size = std::mem::size_of::<u64>();

    cpu_info.current_frequency = fetch_sysctl_frequency(libc::CTL_HW, 0, size)?;
    cpu_info.min_frequency = fetch_sysctl_frequency(libc::CTL_HW, 0, size)?;
    cpu_info.max_frequency = fetch_sysctl_frequency(libc::CTL_HW, 0, size)?;

    Ok(cpu_info)
}

unsafe fn fetch_sysctl_frequency(mib1: i32, mib2: i32, mut size: usize) -> Result<f64> {
    let mut freq: u64 = 0;
    let result = libc::sysctl(
        [mib1, mib2].as_mut_ptr(),
        2,
        &mut freq as *mut _ as *mut libc::c_void,
        &mut size,
        std::ptr::null_mut(),
        0,
    );

    if result != 0 {
        return Err(Error::system("Failed to fetch CPU frequency via sysctl"));
    }

    Ok(freq as f64 / 1_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frequency_metrics() {
        let metrics = FrequencyMetrics {
            current: 2400.0,
            min: 1200.0,
            max: 3600.0,
            available: vec![1200.0, 1800.0, 2400.0, 3000.0, 3600.0],
        };
        
        assert_eq!(metrics.current, 2400.0);
        assert_eq!(metrics.min, 1200.0);
        assert_eq!(metrics.max, 3600.0);
        assert_eq!(metrics.available.len(), 5);
    }
    
    #[test]
    fn test_frequency_monitor_new() {
        let monitor = FrequencyMonitor::new();
        // Simply test that we can create the monitor
        assert!(matches!(monitor, FrequencyMonitor));
    }
    
    // Create a mock implementation for testing sysctl calls
    #[test]
    #[cfg(target_os = "macos")]
    fn test_fetch_sysctl_frequency() {
        // We can't easily test the actual syscalls, but we can test our error handling
        // by passing invalid MIBs
        
        unsafe {
            let result = fetch_sysctl_frequency(-1, -1, std::mem::size_of::<u64>());
            assert!(result.is_err());
            
            if let Err(err) = result {
                assert!(matches!(err, Error::System(_)));
            }
        }
    }
}
