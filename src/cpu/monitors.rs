use crate::{
    core::metrics::Metric,
    core::types::{Percentage, Temperature},
    cpu::{CPU, CpuUtilization},
    error::{Error, Result},
    traits::{CpuMonitor, HardwareMonitor, TemperatureMonitor, UtilizationMonitor},
};
use async_trait::async_trait;
use std::time::{Instant, SystemTime};
use libc;
use std::ffi::CString;

//=============================================================================
// CPU Temperature Monitor
//=============================================================================

/// Monitor for CPU temperature
#[derive(Debug)]
pub struct CpuTemperatureMonitor {
    cpu: CPU,
    device_id: String,
}

impl CpuTemperatureMonitor {
    /// Creates a new CpuTemperatureMonitor for the provided CPU and device ID
    pub fn new(cpu: CPU, device_id: String) -> Self {
        Self { cpu, device_id }
    }
}

#[async_trait]
impl HardwareMonitor for CpuTemperatureMonitor {
    type MetricType = Temperature;

    async fn name(&self) -> Result<String> {
        Ok("CPU Temperature Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("CPU".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.device_id.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let temp = self.cpu.temperature().unwrap_or(0.0);
        let temp = Temperature::new_celsius(temp);
        Ok(Metric::new(temp))
    }
}

#[async_trait]
impl TemperatureMonitor for CpuTemperatureMonitor {
    async fn temperature(&self) -> Result<f64> {
        Ok(self.cpu.temperature().unwrap_or(0.0))
    }
}

//=============================================================================
// CPU Utilization Monitor
//=============================================================================

/// Monitor for CPU utilization metrics
#[derive(Debug)]
pub struct CpuUtilizationMonitor {
    cpu: CPU,
    device_id: String,
    last_utilization: Option<CpuUtilization>,
}

impl CpuUtilizationMonitor {
    /// Creates a new CpuUtilizationMonitor with the provided CPU and device ID
    pub fn new(cpu: CPU, device_id: String) -> Self {
        Self {
            cpu,
            device_id,
            last_utilization: None,
        }
    }
    
    /// Get detailed CPU utilization information
    pub async fn utilization_info(&self) -> Result<CpuUtilization> {
        // This is a placeholder implementation
        // In a real implementation, you would get the actual utilization metrics
        let usage = self.cpu.average_utilization().await?;
        Ok(CpuUtilization {
            user: usage * 0.7,
            system: usage * 0.2,
            idle: 100.0 - usage,
            nice: usage * 0.1,
            timestamp: Instant::now(),
        })
    }
}

#[async_trait]
impl HardwareMonitor for CpuUtilizationMonitor {
    type MetricType = Percentage;

    async fn name(&self) -> Result<String> {
        Ok("CPU Utilization Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("CPU".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.device_id.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let percentage_value = self.utilization().await?;
        let percentage = Percentage::new(percentage_value).unwrap_or(Percentage::new(0.0).unwrap());
        Ok(Metric::new(percentage))
    }
}

#[async_trait]
impl UtilizationMonitor for CpuUtilizationMonitor {
    async fn utilization(&self) -> Result<f64> {
        self.cpu.average_utilization().await
    }
}

//=============================================================================
// FrequencyMetrics and FrequencyMonitor 
//=============================================================================

/// Frequency metrics structure containing current, min, max, and available frequencies
#[derive(Debug, Clone, PartialEq)]
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

/// Low-level monitor for CPU frequency that directly accesses system information
/// 
/// This monitor uses syscalls to fetch frequency information directly from the OS
#[derive(Debug)]
pub struct FrequencyMonitor;

impl Default for FrequencyMonitor {
    fn default() -> Self {
        FrequencyMonitor
    }
}

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
    /// This method queries the system for detailed frequency information, including current, minimum, and maximum
    /// frequencies.
    ///
    /// # Returns
    ///
    /// * `Result<FrequencyMetrics>` - CPU frequency metrics or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the system calls fail or if the frequency information cannot be retrieved.
    pub fn get_metrics(&self) -> Result<FrequencyMetrics> {
        fetch_cpu_frequencies()
    }
}

//=============================================================================
// CPU Frequency Monitor
//=============================================================================

/// High-level monitor for CPU frequency metrics
#[derive(Debug)]
pub struct CpuFrequencyMonitor {
    cpu: CPU,
    device_id: String,
}

impl CpuFrequencyMonitor {
    /// Creates a new CpuFrequencyMonitor with the provided CPU and device ID
    pub fn new(cpu: CPU, device_id: String) -> Self {
        Self { cpu, device_id }
    }
    
    /// Gets the current CPU frequency in MHz
    pub fn current_frequency(&self) -> f64 {
        self.cpu.frequency_mhz()
    }
    
    /// Gets the minimum CPU frequency in MHz
    pub fn min_frequency(&self) -> Option<f64> {
        self.cpu.min_frequency_mhz()
    }
    
    /// Gets the maximum CPU frequency in MHz
    pub fn max_frequency(&self) -> Option<f64> {
        self.cpu.max_frequency_mhz()
    }
    
    /// Gets the available CPU frequency steps in MHz
    pub fn available_frequencies(&self) -> Option<&[f64]> {
        self.cpu.available_frequencies()
    }
    
    /// Retrieves the current CPU frequency metrics.
    ///
    /// This method queries the system for detailed frequency information, including current, minimum, and maximum
    /// frequencies.
    ///
    /// # Returns
    ///
    /// * `Result<FrequencyMetrics>` - CPU frequency metrics or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the system calls fail or if the frequency information cannot be retrieved.
    pub fn get_metrics(&self) -> Result<FrequencyMetrics> {
        let current = self.current_frequency();
        let min = self.min_frequency().unwrap_or(0.0);
        let max = self.max_frequency().unwrap_or(0.0);
        let available = self.available_frequencies()
            .map(|freqs| freqs.to_vec())
            .unwrap_or_else(|| {
                // Create a reasonable set of frequency steps
                if min > 0.0 && max > min {
                    let step = (max - min) / 4.0;
                    vec![
                        min,
                        min + step,
                        min + step * 2.0,
                        min + step * 3.0,
                        max,
                    ]
                } else {
                    vec![current]
                }
            });
            
        Ok(FrequencyMetrics {
            current,
            min,
            max,
            available,
        })
    }
}

#[async_trait]
impl HardwareMonitor for CpuFrequencyMonitor {
    type MetricType = f64;

    async fn name(&self) -> Result<String> {
        Ok("CPU Frequency Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("CPU".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.device_id.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let freq = self.current_frequency();
        Ok(Metric::new(freq))
    }
}

#[async_trait]
impl CpuMonitor for CpuFrequencyMonitor {
    async fn frequency(&self) -> Result<f64> {
        Ok(self.current_frequency())
    }

    async fn min_frequency(&self) -> Result<f64> {
        Ok(self.min_frequency().unwrap_or(0.0))
    }

    async fn max_frequency(&self) -> Result<f64> {
        Ok(self.max_frequency().unwrap_or(0.0))
    }

    async fn available_frequencies(&self) -> Result<Vec<f64>> {
        Ok(self.available_frequencies().unwrap_or(&[]).to_vec())
    }

    async fn physical_cores(&self) -> Result<u32> {
        self.cpu.core_count().await
    }

    async fn logical_cores(&self) -> Result<u32> {
        self.cpu.core_count().await
    }

    async fn model_name(&self) -> Result<String> {
        Ok("Unknown CPU Model".to_string())
    }

    async fn temperature(&self) -> Result<Option<f64>> {
        Ok(self.cpu.temperature())
    }

    async fn power_consumption(&self) -> Result<Option<f64>> {
        Ok(None)
    }

    async fn core_usage(&self) -> Result<Vec<f64>> {
        self.cpu.utilization().await
    }

    async fn total_usage(&self) -> Result<f64> {
        self.cpu.average_utilization().await
    }
}

//=============================================================================
// Internal Helper Functions
//=============================================================================

/// Internal CPU information structure used during frequency retrieval.
#[derive(Default)]
struct CpuInfo {
    current_frequency: f64,
    min_frequency: f64,
    max_frequency: f64,
    available_frequencies: Vec<f64>,
}

/// Fetches CPU frequency information and returns it as FrequencyMetrics.
///
/// # Returns
///
/// * `Result<FrequencyMetrics>` - CPU frequency metrics or an error
pub fn fetch_cpu_frequencies() -> Result<FrequencyMetrics> {
    let cpu_info = unsafe { retrieve_cpu_info()? };
    Ok(FrequencyMetrics {
        current: cpu_info.current_frequency,
        min: cpu_info.min_frequency,
        max: cpu_info.max_frequency,
        available: cpu_info.available_frequencies,
    })
}

/// Retrieves detailed CPU frequency information from the system.
///
/// # Safety
///
/// This function is unsafe because it calls libc functions.
///
/// # Returns
///
/// * `Result<CpuInfo>` - CPU frequency information or an error
pub unsafe fn retrieve_cpu_info() -> Result<CpuInfo> {
    // Get CPU frequency using proper MIBs On macOS, "hw.cpufrequency" gives current CPU frequency in Hz
    // "hw.cpufrequency_min" gives min frequency, "hw.cpufrequency_max" gives max

    // Convert Hz to MHz
    let current_frequency = fetch_sysctl_frequency_by_name("hw.cpufrequency")? / 1_000_000.0;
    let min_frequency = fetch_sysctl_frequency_by_name("hw.cpufrequency_min")? / 1_000_000.0;
    let max_frequency = fetch_sysctl_frequency_by_name("hw.cpufrequency_max")? / 1_000_000.0;

    // For available frequencies, we use min/max and interpolate since macOS doesn't provide a direct way to get all
    // steps
    let mut available_frequencies = Vec::new();
    if min_frequency > 0.0 && max_frequency > min_frequency {
        let step = (max_frequency - min_frequency) / 4.0;
        available_frequencies = vec![
            min_frequency,
            min_frequency + step,
            min_frequency + step * 2.0,
            min_frequency + step * 3.0,
            max_frequency,
        ];
    }

    Ok(CpuInfo { current_frequency, min_frequency, max_frequency, available_frequencies })
}

/// Fetches a frequency value from sysctl by name.
///
/// # Safety
///
/// This function is unsafe because it calls libc functions.
///
/// # Arguments
///
/// * `name` - The name of the sysctl parameter to fetch
///
/// # Returns
///
/// * `Result<f64>` - The frequency value or an error
pub unsafe fn fetch_sysctl_frequency_by_name(name: &str) -> Result<f64> {
    // Create null-terminated C string for the sysctl name
    let c_name = CString::new(name)
        .map_err(|_| Error::system(format!("Failed to create C string for sysctl name: {}", name)))?;

    let mut freq: u64 = 0;
    let mut size = std::mem::size_of::<u64>();

    let result = libc::sysctlbyname(
        c_name.as_ptr(),
        &mut freq as *mut _ as *mut libc::c_void,
        &mut size,
        std::ptr::null_mut(),
        0,
    );

    if result != 0 {
        return Err(Error::system(format!("Failed to fetch CPU frequency via sysctlbyname: {}", name)));
    }

    Ok(freq as f64)
} 