use crate::error::{Error, Result};
use libc;

/// Container for CPU frequency-related metrics.
///
/// This structure holds comprehensive information about CPU frequency
/// capabilities and current state, including minimum, maximum, and
/// current operating frequencies.
///
/// # Fields
///
/// * `current` - Current CPU frequency in MHz
/// * `min` - Minimum supported CPU frequency in MHz
/// * `max` - Maximum supported CPU frequency in MHz
/// * `available` - List of all available frequency steps in MHz
///
/// # Example
///
/// ```rust
/// use darwin_metrics::hardware::cpu::FrequencyMetrics;
///
/// // Example of using FrequencyMetrics
/// let metrics = FrequencyMetrics {
///     current: 2400.0,
///     min: 1200.0,
///     max: 3600.0,
///     available: vec![1200.0, 1800.0, 2400.0, 3000.0, 3600.0],
/// };
///
/// println!("Current frequency: {} MHz", metrics.current);
/// println!("Min frequency: {} MHz", metrics.min);
/// println!("Max frequency: {} MHz", metrics.max);
/// println!("Available steps: {:?} MHz", metrics.available);
/// ```
#[derive(Debug, Clone)]
pub struct FrequencyMetrics {
    /// Current CPU frequency in MHz
    pub current: f64,

    /// Minimum supported CPU frequency in MHz
    pub min: f64,

    /// Maximum supported CPU frequency in MHz
    pub max: f64,

    /// List of all available frequency steps in MHz
    pub available: Vec<f64>,
}

/// Monitor for CPU frequency metrics.
///
/// This struct provides methods to retrieve detailed CPU frequency information
/// from the macOS system, including current operating frequency and the range
/// of supported frequencies.
///
/// Note: This implementation is still in development and might be improved
/// in future versions to provide more accurate frequency data.
#[derive(Debug)]
pub struct FrequencyMonitor;

impl FrequencyMonitor {
    /// Creates a new FrequencyMonitor instance.
    ///
    /// # Returns
    ///
    /// * `Self` - A new FrequencyMonitor instance
    pub fn new() -> Self {
        Self
    }

    /// Retrieves the current CPU frequency metrics.
    ///
    /// This method queries the system for detailed frequency information,
    /// including current, minimum, and maximum frequencies.
    ///
    /// # Returns
    ///
    /// * `Result<FrequencyMetrics>` - CPU frequency metrics or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the system calls fail or if the frequency
    /// information cannot be retrieved.
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
    
    // Get CPU frequency using proper MIBs
    // On macOS, "hw.cpufrequency" gives current CPU frequency in Hz
    // "hw.cpufrequency_min" gives min frequency, "hw.cpufrequency_max" gives max
    
    // Convert to MHz
    cpu_info.current_frequency = fetch_sysctl_frequency_by_name("hw.cpufrequency")? / 1_000_000.0;
    cpu_info.min_frequency = fetch_sysctl_frequency_by_name("hw.cpufrequency_min")? / 1_000_000.0;
    cpu_info.max_frequency = fetch_sysctl_frequency_by_name("hw.cpufrequency_max")? / 1_000_000.0;
    
    // For available frequencies, we use min/max and interpolate
    // since macOS doesn't provide a direct way to get all steps
    if cpu_info.min_frequency > 0.0 && cpu_info.max_frequency > cpu_info.min_frequency {
        let step = (cpu_info.max_frequency - cpu_info.min_frequency) / 4.0;
        cpu_info.available_frequencies = vec![
            cpu_info.min_frequency,
            cpu_info.min_frequency + step,
            cpu_info.min_frequency + step * 2.0,
            cpu_info.min_frequency + step * 3.0,
            cpu_info.max_frequency,
        ];
    }

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
