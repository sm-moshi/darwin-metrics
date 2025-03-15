use objc2::msg_send; // rc::Retained};
                     // use objc2_foundation::NSString;
use libc::sysctlbyname;
use objc2::runtime::AnyObject;
use std::{ffi::CString, ptr};

use super::{CpuMetrics, FrequencyMetrics, FrequencyMonitor};
#[cfg(test)]
use crate::hardware::iokit::mock::MockIOKit;
use crate::{
    error::{Error, Result},
    hardware::iokit::{IOKit, IOKitImpl},
};

/// Primary structure for accessing macOS CPU information and metrics.
///
/// `CPU` provides a comprehensive interface for monitoring and reporting CPU statistics on macOS systems. It leverages
/// the IOKit framework to communicate with the hardware and retrieve accurate, real-time data about the system's
/// processor.
///
/// This implementation works with both Intel and Apple Silicon processors and provides consistent metrics across
/// different macOS hardware configurations.
///
/// # Fields
///
/// * `physical_cores` - Number of physical CPU cores
/// * `logical_cores` - Number of logical CPU cores (including hyperthreading)
/// * `frequency_mhz` - Current CPU frequency in MHz
/// * `core_usage` - Per-core CPU usage values (0.0 to 1.0)
/// * `model_name` - CPU model name as reported by the system
/// * `temperature` - Current CPU temperature in Celsius (if available)
/// * `power` - Current CPU power in watts (if available)
/// * `iokit` - IOKit interface for hardware communication
#[derive(Debug)]
pub struct CPU {
    physical_cores: u32,
    logical_cores: u32,
    frequency_mhz: f64,
    core_usage: Vec<f64>,
    model_name: String,
    temperature: Option<f64>,
    power: Option<f64>,
    iokit: Box<dyn IOKit>,
    frequency_monitor: FrequencyMonitor,
    frequency_metrics: Option<FrequencyMetrics>,
}

impl CPU {
    /// Creates a new CPU instance with real-time metrics.
    ///
    /// This constructor initializes a new CPU monitor and immediately populates it with the current system CPU metrics
    /// by calling `update()`.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - A new CPU instance or an error if initialization failed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The IOKit service cannot be accessed
    /// * Required system calls fail
    /// * CPU metrics cannot be retrieved
    ///
    /// # Example
    ///
    /// ```no_run
    /// use darwin_metrics::hardware::cpu::CPU;
    ///
    /// fn main() -> darwin_metrics::error::Result<()> {
    ///     let cpu = CPU::new()?;
    ///     println!("CPU: {}", cpu.model_name());
    ///     Ok(())
    /// }
    /// ```
    pub fn new() -> Result<Self> {
        let mut cpu = Self {
            physical_cores: 0,
            logical_cores: 0,
            frequency_mhz: 0.0,
            core_usage: Vec::new(),
            model_name: String::new(),
            temperature: None,
            power: None,
            iokit: Box::new(IOKitImpl::default()),
            frequency_monitor: FrequencyMonitor::new(),
            frequency_metrics: None,
        };
        cpu.update()?;
        Ok(cpu)
    }

    /// Updates all CPU metrics with current values.
    ///
    /// This method refreshes all CPU data by querying the system for the latest metrics. It should be called
    /// periodically to ensure that the CPU information is current.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Success or an error if updating failed
    ///
    /// # Errors
    ///
    /// Returns an error if any of the system calls or IOKit operations fail.
    pub fn update(&mut self) -> Result<()> {
        // Update physical and logical core counts
        self.physical_cores = self.iokit.get_physical_cores()? as u32;
        self.logical_cores = self.iokit.get_logical_cores()? as u32;

        // Update core usage
        let physical_core_usage = self.iokit.get_core_usage()?;
        self.core_usage = Vec::with_capacity(self.logical_cores as usize);

        // For each logical core, use the corresponding physical core's usage
        for i in 0..self.logical_cores {
            let physical_core_index = (i % self.physical_cores) as usize;
            let usage = physical_core_usage.get(physical_core_index).copied().unwrap_or(0.0);
            self.core_usage.push(usage);
        }

        // Update frequency information
        self.update_frequency_information()?;

        Ok(())
    }

    /// Updates CPU frequency information using the most reliable method available.
    ///
    /// This method attempts to get frequency information in the following order:
    /// 1. From FrequencyMonitor (which uses sysctl calls)
    /// 2. Falling back to IOKit if FrequencyMonitor fails
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Success or an error if all frequency retrieval methods fail
    fn update_frequency_information(&mut self) -> Result<()> {
        // First attempt: Use FrequencyMonitor to get detailed frequency metrics
        match self.frequency_monitor.get_metrics() {
            Ok(metrics) => {
                // Successfully retrieved metrics from FrequencyMonitor
                self.frequency_mhz = metrics.current;
                self.frequency_metrics = Some(metrics);
                Ok(())
            },
            Err(primary_error) => {
                // FrequencyMonitor failed, try fallback method with IOKit
                match self.get_frequency_from_iokit() {
                    Ok(frequency_mhz) => {
                        // Fallback succeeded
                        self.frequency_mhz = frequency_mhz;
                        self.frequency_metrics = None;
                        Ok(())
                    },
                    Err(fallback_error) => {
                        // Both methods failed, return the primary error
                        Err(Error::system(format!(
                            "Failed to retrieve CPU frequency: primary method error: {}, fallback method error: {}",
                            primary_error, fallback_error
                        )))
                    },
                }
            },
        }
    }

    /// Retrieves CPU frequency using IOKit as a fallback method.
    ///
    /// # Returns
    ///
    /// * `Result<f64>` - CPU frequency in MHz or an error
    fn get_frequency_from_iokit(&self) -> Result<f64> {
        let service = self.iokit.get_service_matching("AppleACPICPU")?;
        let _service_ref = service.as_ref().ok_or_else(|| Error::iokit_error(0, "Failed to get CPU service"))?;

        // Create an AnyObject from the raw pointer
        let obj = unsafe {
            let ptr = _service_ref as *const _ as *mut AnyObject;
            &*ptr
        };
        let frequency: f64 = unsafe { msg_send![obj, currentProcessorClockSpeed] };

        // Convert to MHz if needed (IOKit might return in Hz)
        Ok(if frequency > 1000.0 { frequency / 1000.0 } else { frequency })
    }

    /// Retrieves the current usage for each CPU core.
    ///
    /// This method queries the system for per-core CPU usage statistics, returning a vector of usage values where each
    /// value is between 0.0 (idle) and 1.0 (100% utilized).
    ///
    /// # Returns
    ///
    /// * `Result<Vec<f64>>` - Vector of core usage values or an error
    ///
    /// # Implementation Notes
    ///
    /// The current implementation uses the IOKit AppleACPICPU service, but alternative implementations using
    /// host_processor_info() from the Mach kernel API or sysctlbyname() may be more reliable and are planned for future
    /// updates.
    fn fetch_core_usage(&self) -> Result<Vec<f64>> {
        // Get a single service instance to use for all cores
        let service = self.iokit.get_service_matching("AppleACPICPU")?;
        let _service_ref = service.ok_or_else(|| Error::iokit_error(0, "Failed to get CPU service"))?;

        // Pre-allocate the vector with the correct capacity
        let mut usages = Vec::with_capacity(self.logical_cores as usize);

        // Get usage for each core
        for i in 0..self.logical_cores {
            // For logical cores beyond physical cores, use the corresponding physical core's usage
            let physical_core_index = i % self.physical_cores;
            let usage = self.iokit.get_core_usage()?.get(physical_core_index as usize).copied().unwrap_or(0.0);
            usages.push(usage);
        }

        Ok(usages)
    }

    /// Retrieves the CPU model name using sysctlbyname.
    ///
    /// This method uses the macOS sysctlbyname API to get the CPU brand string,
    /// which is more reliable than using IOKit for this purpose.
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The CPU model name or an error
    fn get_cpu_model_name_sysctl(&self) -> Result<String> {
        let mut buffer = [0u8; 128];
        let mut size = buffer.len();
        let name = CString::new("machdep.cpu.brand_string").unwrap();

        let result =
            unsafe { sysctlbyname(name.as_ptr(), buffer.as_mut_ptr() as *mut _, &mut size, ptr::null_mut(), 0) };

        if result != 0 {
            return Err(Error::iokit_error(result, "Failed to get CPU model name"));
        }

        Ok(String::from_utf8_lossy(&buffer[..size]).trim_matches(|c: char| c == '\0' || c.is_whitespace()).to_string())
    }

    /// Returns the number of physical CPU cores.
    ///
    /// Physical cores represent the actual hardware cores on the CPU die. This count does not include virtual cores
    /// created by technologies like Intel Hyper-Threading.
    ///
    /// # Returns
    ///
    /// * `u32` - Number of physical CPU cores
    pub fn physical_cores(&self) -> u32 {
        self.physical_cores
    }

    /// Returns the number of logical CPU cores.
    ///
    /// Logical cores include both physical cores and virtual cores created by technologies like Intel Hyper-Threading.
    /// This value represents the total number of independent processing units available to the OS.
    ///
    /// # Returns
    ///
    /// * `u32` - Number of logical CPU cores
    pub fn logical_cores(&self) -> u32 {
        self.logical_cores
    }

    /// Returns the current CPU frequency in MHz.
    ///
    /// This value represents the current operating frequency of the CPU and may vary based on power management policies
    /// and CPU load.
    ///
    /// # Returns
    ///
    /// * `f64` - Current CPU frequency in MHz
    pub fn frequency_mhz(&self) -> f64 {
        self.frequency_mhz
    }

    /// Returns the current usage for each CPU core.
    ///
    /// This method provides a slice of CPU usage values, with one value per core. Each value is between 0.0 (idle) and
    /// 1.0 (100% utilized).
    ///
    /// # Returns
    ///
    /// * `&[f64]` - Slice of core usage values
    pub fn core_usage(&self) -> &[f64] {
        &self.core_usage
    }

    /// Returns the CPU model name.
    ///
    /// The model name is the marketing name for the processor as reported by the system (e.g., "Apple M1 Pro" or "Intel
    /// Core i9").
    ///
    /// # Returns
    ///
    /// * `&str` - CPU model name
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Returns the current CPU temperature in degrees Celsius, if available.
    ///
    /// # Returns
    ///
    /// * `Option<f64>` - Temperature in degrees Celsius, or None if unavailable
    pub fn temperature(&self) -> Option<f64> {
        self.temperature
    }

    /// Returns the current CPU power in watts, if available.
    ///
    /// # Returns
    ///
    /// * `Option<f64>` - Power in watts, or None if unavailable
    pub fn power(&self) -> Option<f64> {
        self.power
    }

    /// Returns detailed CPU frequency metrics if available.
    ///
    /// This method provides access to the detailed frequency information including minimum, maximum, and available
    /// frequency steps.
    ///
    /// # Returns
    ///
    /// * `Option<&FrequencyMetrics>` - Detailed frequency metrics or None if not available
    pub fn frequency_metrics(&self) -> Option<&FrequencyMetrics> {
        self.frequency_metrics.as_ref()
    }

    /// Returns the minimum CPU frequency in MHz if available.
    ///
    /// # Returns
    ///
    /// * `Option<f64>` - Minimum frequency in MHz or None if not available
    pub fn min_frequency_mhz(&self) -> Option<f64> {
        self.frequency_metrics.as_ref().map(|m| m.min)
    }

    /// Returns the maximum CPU frequency in MHz if available.
    ///
    /// # Returns
    ///
    /// * `Option<f64>` - Maximum frequency in MHz or None if not available
    pub fn max_frequency_mhz(&self) -> Option<f64> {
        self.frequency_metrics.as_ref().map(|m| m.max)
    }

    /// Returns the available CPU frequency steps in MHz if available.
    ///
    /// # Returns
    ///
    /// * `Option<&[f64]>` - Available frequency steps in MHz or None if not available
    pub fn available_frequencies(&self) -> Option<&[f64]> {
        self.frequency_metrics.as_ref().map(|m| m.available.as_slice())
    }

    #[cfg(test)]
    pub fn new_with_mock() -> Result<Self> {
        let mock = MockIOKit::new().expect("Failed to create MockIOKit");
        Ok(Self {
            physical_cores: 8,
            logical_cores: 16,
            frequency_mhz: 3200.0,
            core_usage: vec![0.3, 0.5, 0.2, 0.8, 0.1, 0.3, 0.4, 0.6],
            model_name: String::from("Apple M1 Pro"),
            temperature: Some(45.5),
            power: Some(20.0),
            iokit: Box::new(mock),
            frequency_monitor: FrequencyMonitor::new(),
            frequency_metrics: Some(FrequencyMetrics {
                current: 3200.0,
                min: 1200.0,
                max: 3600.0,
                available: vec![1200.0, 1800.0, 2400.0, 3000.0, 3600.0],
            }),
        })
    }

    #[cfg(test)]
    pub fn new_with_iokit(iokit: Box<dyn IOKit>) -> Result<Self> {
        let mut cpu = Self {
            physical_cores: 0,
            logical_cores: 0,
            frequency_mhz: 0.0,
            core_usage: Vec::new(),
            model_name: String::new(),
            temperature: None,
            power: None,
            iokit,
            frequency_monitor: FrequencyMonitor::new(),
            frequency_metrics: None,
        };
        cpu.update()?;
        Ok(cpu)
    }
}

/// Implementation of the CpuMetrics trait for the CPU struct.
///
/// This implementation provides a standardized interface for accessing key CPU metrics, allowing consumers to interact
/// with the CPU struct through a consistent API regardless of the underlying implementation details.
impl CpuMetrics for CPU {
    /// Returns the average CPU usage across all cores.
    ///
    /// This method calculates the average CPU usage by summing the usage values for all cores and dividing by the
    /// number of logical cores. The result is a value between 0.0 (completely idle) and 1.0 (100% utilized).
    ///
    /// # Returns
    ///
    /// * `f64` - Average CPU usage (0.0 to 1.0)
    fn get_cpu_usage(&self) -> f64 {
        // Sum all core usages
        let total_usage: f64 = self.core_usage.iter().sum();

        // Calculate average, ensuring we don't divide by zero
        if self.logical_cores > 0 {
            total_usage / self.logical_cores as f64
        } else {
            0.0
        }
    }

    /// Returns the current CPU temperature in degrees Celsius, if available.
    ///
    /// # Returns
    ///
    /// * `Option<f64>` - Temperature in degrees Celsius, or None if unavailable
    fn get_cpu_temperature(&self) -> Option<f64> {
        self.temperature
    }

    /// Returns the current CPU frequency in MHz.
    ///
    /// # Returns
    ///
    /// * `f64` - Current CPU frequency in MHz
    fn get_cpu_frequency(&self) -> f64 {
        self.frequency_mhz
    }
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            physical_cores: 0,
            logical_cores: 0,
            frequency_mhz: 0.0,
            core_usage: Vec::new(),
            model_name: String::new(),
            temperature: None,
            power: None,
            iokit: Box::new(IOKitImpl::default()),
            frequency_monitor: FrequencyMonitor::new(),
            frequency_metrics: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::iokit::mock::MockIOKit;

    #[test]
    fn test_cpu_metrics() {
        let mock_iokit = MockIOKit::new()
            .expect("Failed to create MockIOKit")
            .with_physical_cores(4)
            .expect("Failed to set physical cores")
            .with_logical_cores(8)
            .expect("Failed to set logical cores")
            .with_core_usage(vec![0.3, 0.4, 0.5, 0.6])
            .expect("Failed to set core usage");

        let cpu = CPU::new_with_iokit(Box::new(mock_iokit)).expect("Failed to create CPU");

        assert_eq!(cpu.physical_cores(), 4);
        assert_eq!(cpu.logical_cores(), 8);

        let core_usage = cpu.core_usage();
        assert_eq!(core_usage.len(), 8);
        for i in 0..4 {
            assert_eq!(core_usage[i], vec![0.3, 0.4, 0.5, 0.6][i]);
            assert_eq!(core_usage[i + 4], vec![0.3, 0.4, 0.5, 0.6][i]);
        }
    }

    #[test]
    fn test_cpu_metrics_with_usage() {
        let mock_iokit = MockIOKit::new()
            .expect("Failed to create MockIOKit")
            .with_physical_cores(4)
            .expect("Failed to set physical cores")
            .with_logical_cores(8)
            .expect("Failed to set logical cores")
            .with_core_usage(vec![0.5, 0.6, 0.7, 0.8])
            .expect("Failed to set core usage");

        let cpu = CPU::new_with_iokit(Box::new(mock_iokit)).expect("Failed to create CPU");

        let core_usage = cpu.core_usage();

        assert_eq!(core_usage.len(), 4);
        assert_eq!(core_usage[0], 0.5);
        assert_eq!(core_usage[1], 0.6);
        assert_eq!(core_usage[2], 0.7);
        assert_eq!(core_usage[3], 0.8);
    }

    #[test]
    fn test_core_usage_with_different_core_counts() {
        let mock_iokit = MockIOKit::new()
            .expect("Failed to create MockIOKit")
            .with_physical_cores(2)
            .expect("Failed to set physical cores")
            .with_logical_cores(4)
            .expect("Failed to set logical cores")
            .with_core_usage(vec![0.3, 0.4])
            .expect("Failed to set core usage");

        let cpu = CPU::new_with_iokit(Box::new(mock_iokit)).expect("Failed to create CPU");

        let core_usage = cpu.core_usage();

        // Since we have 4 logical cores but 2 physical cores,
        // we expect the usage pattern to repeat: [0.3, 0.4, 0.3, 0.4]
        assert_eq!(core_usage.len(), 4);
        assert_eq!(core_usage[0], 0.3);
        assert_eq!(core_usage[1], 0.4);
        assert_eq!(core_usage[2], 0.3);
        assert_eq!(core_usage[3], 0.4);
        assert_eq!(cpu.physical_cores(), 2);
        assert_eq!(cpu.logical_cores(), 4);
    }
}
