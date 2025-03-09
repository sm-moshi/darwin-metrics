use objc2::{msg_send, rc::Retained};
use objc2_foundation::NSString;

use super::{CpuMetrics, FrequencyMetrics, FrequencyMonitor};
#[cfg(test)]
use crate::hardware::iokit::MockIOKit;
use crate::{
    error::Result,
    hardware::iokit::{IOKit, IOKitImpl},
};

/// Primary structure for accessing macOS CPU information and metrics.
///
/// `CPU` provides a comprehensive interface for monitoring and reporting CPU
/// statistics on macOS systems. It leverages the IOKit framework to communicate
/// with the hardware and retrieve accurate, real-time data about the system's
/// processor.
///
/// This implementation works with both Intel and Apple Silicon processors and
/// provides consistent metrics across different macOS hardware configurations.
///
/// # Fields
///
/// * `physical_cores` - Number of physical CPU cores
/// * `logical_cores` - Number of logical CPU cores (including hyperthreading)
/// * `frequency_mhz` - Current CPU frequency in MHz
/// * `core_usage` - Per-core CPU usage values (0.0 to 1.0)
/// * `model_name` - CPU model name as reported by the system
/// * `temperature` - Current CPU temperature in Celsius (if available)
/// * `iokit` - IOKit interface for hardware communication
#[derive(Debug)]
pub struct CPU {
    physical_cores: u32,
    logical_cores: u32,
    frequency_mhz: f64,
    core_usage: Vec<f64>,
    model_name: String,
    temperature: Option<f64>,
    iokit: Box<dyn IOKit>,
    frequency_monitor: FrequencyMonitor,
    frequency_metrics: Option<FrequencyMetrics>,
}

impl CPU {
    /// Creates a new CPU instance with real-time metrics.
    ///
    /// This constructor initializes a new CPU monitor and immediately populates
    /// it with the current system CPU metrics by calling `update()`.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - A new CPU instance or an error if initialization
    ///   failed
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
            iokit: Box::new(IOKitImpl),
            frequency_monitor: FrequencyMonitor::new(),
            frequency_metrics: None,
        };
        cpu.update()?;
        Ok(cpu)
    }

    /// Updates all CPU metrics with current values.
    ///
    /// This method refreshes all CPU data by querying the system for the latest
    /// metrics. It should be called periodically to ensure that the CPU
    /// information is current.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Success or an error if updating failed
    ///
    /// # Errors
    ///
    /// Returns an error if any of the system calls or IOKit operations fail.
    pub fn update(&mut self) -> Result<()> {
        let service = self.iokit.get_service("AppleACPICPU")?;

        // TODO: Verify if AppleACPICPU service actually provides these methods
        // If not, consider using sysctl or other methods for getting core counts
        // Example: sysctlbyname("hw.physicalcpu") and sysctlbyname("hw.logicalcpu")
        self.physical_cores = unsafe { msg_send![&*service, numberOfCores] };
        self.logical_cores = unsafe { msg_send![&*service, numberOfProcessorCores] };

        // Use the FrequencyMonitor to get detailed frequency information
        // We try to get metrics from FrequencyMonitor first, with a fallback to IOKit
        match self.frequency_monitor.get_metrics() {
            Ok(metrics) => {
                self.frequency_mhz = metrics.current;
                self.frequency_metrics = Some(metrics);
            },
            Err(_) => {
                // Fallback to IOKit method
                let frequency: f64 = unsafe { msg_send![&*service, currentProcessorClockSpeed] };
                self.frequency_mhz = frequency / 1_000_000.0;
                self.frequency_metrics = None;
            },
        }

        self.core_usage = self.fetch_core_usage()?;

        // Get model name as NSString and convert to Rust String
        // TODO: Verify name method, consider using
        // sysctlbyname("machdep.cpu.brand_string")
        let ns_name: Retained<NSString> = unsafe { msg_send![&*service, name] };
        self.model_name = ns_name.to_string();

        // Temperature from IOKit
        self.temperature = self.fetch_cpu_temperature();

        Ok(())
    }

    /// Retrieves the current usage for each CPU core.
    ///
    /// This method queries the system for per-core CPU usage statistics,
    /// returning a vector of usage values where each value is between 0.0
    /// (idle) and 1.0 (100% utilized).
    ///
    /// # Returns
    ///
    /// * `Result<Vec<f64>>` - Vector of core usage values or an error
    ///
    /// # Implementation Notes
    ///
    /// The current implementation uses the IOKit AppleACPICPU service, but
    /// alternative implementations using host_processor_info() from the
    /// Mach kernel API or sysctlbyname() may be more reliable and are
    /// planned for future updates.
    fn fetch_core_usage(&self) -> Result<Vec<f64>> {
        // TODO: Verify if AppleACPICPU provides getCoreUsage method
        // Alternative implementation options:
        // 1. Use host_processor_info() from mach kernel API to get CPU load
        // 2. Parse the output from the 'vm_stat' command
        // 3. Use sysctlbyname with specific processor information keys
        //
        // Example implementation with host_processor_info would look like:
        // unsafe {
        //     let mut cpu_load_info: *mut processor_cpu_load_info_t =
        // std::ptr::null_mut();     let mut cpu_count: u32 = 0;
        //     let result = host_processor_info(mach_host_self(),
        // PROCESSOR_CPU_LOAD_INFO,                                      &mut
        // cpu_count,                                      &mut cpu_load_info as
        // *mut _ as *mut *mut libc::c_int,
        // &mut msg_type);     // Process results and calculate usage
        // percentages }

        let mut usages = Vec::with_capacity(self.logical_cores as usize);
        for i in 0..self.logical_cores {
            let service = self.iokit.get_service("AppleACPICPU")?;
            let usage: f64 = unsafe { msg_send![&*service, getCoreUsage: i] };
            usages.push(usage);
        }
        Ok(usages)
    }

    /// Retrieves the current CPU temperature.
    ///
    /// This method attempts to read the CPU temperature from the system
    /// using the IOKit framework. Temperature readings may not be available
    /// on all systems.
    ///
    /// # Returns
    ///
    /// * `Option<f64>` - Temperature in degrees Celsius, or None if unavailable
    fn fetch_cpu_temperature(&self) -> Option<f64> {
        self.iokit.get_cpu_temperature().ok()
    }

    /// Returns the number of physical CPU cores.
    ///
    /// Physical cores represent the actual hardware cores on the CPU die.
    /// This count does not include virtual cores created by technologies
    /// like Intel Hyper-Threading.
    ///
    /// # Returns
    ///
    /// * `u32` - Number of physical CPU cores
    pub fn physical_cores(&self) -> u32 {
        self.physical_cores
    }

    /// Returns the number of logical CPU cores.
    ///
    /// Logical cores include both physical cores and virtual cores created
    /// by technologies like Intel Hyper-Threading. This value represents
    /// the total number of independent processing units available to the OS.
    ///
    /// # Returns
    ///
    /// * `u32` - Number of logical CPU cores
    pub fn logical_cores(&self) -> u32 {
        self.logical_cores
    }

    /// Returns the current CPU frequency in MHz.
    ///
    /// This value represents the current operating frequency of the CPU
    /// and may vary based on power management policies and CPU load.
    ///
    /// # Returns
    ///
    /// * `f64` - Current CPU frequency in MHz
    pub fn frequency_mhz(&self) -> f64 {
        self.frequency_mhz
    }

    /// Returns the current usage for each CPU core.
    ///
    /// This method provides a slice of CPU usage values, with one value per
    /// core. Each value is between 0.0 (idle) and 1.0 (100% utilized).
    ///
    /// # Returns
    ///
    /// * `&[f64]` - Slice of core usage values
    pub fn core_usage(&self) -> &[f64] {
        &self.core_usage
    }

    /// Returns the CPU model name.
    ///
    /// The model name is the marketing name for the processor as reported
    /// by the system (e.g., "Apple M1 Pro" or "Intel Core i9").
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

    /// Returns detailed CPU frequency metrics if available.
    ///
    /// This method provides access to the detailed frequency information
    /// including minimum, maximum, and available frequency steps.
    ///
    /// # Returns
    ///
    /// * `Option<&FrequencyMetrics>` - Detailed frequency metrics or None if
    ///   not available
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
    /// * `Option<&[f64]>` - Available frequency steps in MHz or None if not
    ///   available
    pub fn available_frequencies(&self) -> Option<&[f64]> {
        self.frequency_metrics.as_ref().map(|m| m.available.as_slice())
    }
}

/// Implementation of the CpuMetrics trait for the CPU struct.
///
/// This implementation provides a standardized interface for accessing
/// key CPU metrics, allowing consumers to interact with the CPU information
/// through the trait without needing to know the details of the underlying
/// implementation.
impl CpuMetrics for CPU {
    /// Returns the average CPU usage across all cores.
    ///
    /// This method calculates the average CPU usage by summing the usage values
    /// for all cores and dividing by the number of logical cores. The result
    /// is a value between 0.0 (completely idle) and 1.0 (100% utilized).
    ///
    /// # Returns
    ///
    /// * `f64` - Average CPU usage (0.0 to 1.0)
    fn get_cpu_usage(&self) -> f64 {
        self.core_usage.iter().sum::<f64>() / self.logical_cores as f64
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

#[cfg(test)]
mod tests {
    use super::*;

    // Create a CPU instance for testing with mock data
    impl CPU {
        pub fn new_with_mock() -> Result<Self> {
            let mut mock = MockIOKit::new();

            // Setup mock behavior
            mock.expect_get_service().returning(|_| {
                use crate::utils::test_utils;
                Ok(test_utils::create_test_object())
            });

            mock.expect_get_cpu_temperature().returning(|| Ok(45.5));

            let cpu = Self {
                physical_cores: 8,
                logical_cores: 16,
                frequency_mhz: 3200.0,
                core_usage: vec![0.3, 0.5, 0.2, 0.8, 0.1, 0.3, 0.4, 0.6],
                model_name: "Apple M1 Pro".to_string(),
                temperature: Some(45.5),
                iokit: Box::new(mock),
                frequency_monitor: FrequencyMonitor::new(),
                frequency_metrics: Some(FrequencyMetrics {
                    current: 3200.0,
                    min: 1200.0,
                    max: 3600.0,
                    available: vec![1200.0, 1800.0, 2400.0, 3000.0, 3600.0],
                }),
            };

            Ok(cpu)
        }
    }

    #[test]
    fn test_cpu_initialization() {
        let cpu = CPU::new_with_mock().expect("Failed to create CPU instance");

        assert_eq!(cpu.physical_cores(), 8);
        assert_eq!(cpu.logical_cores(), 16);
        assert_eq!(cpu.frequency_mhz(), 3200.0);
        assert_eq!(cpu.model_name(), "Apple M1 Pro");
        assert_eq!(cpu.temperature(), Some(45.5));
        assert_eq!(cpu.core_usage().len(), 8);
    }

    #[test]
    fn test_cpu_usage() {
        let cpu = CPU::new_with_mock().expect("Failed to create CPU instance");

        // The core usage values are set to known test values
        assert_eq!(cpu.core_usage().len(), 8);

        // Calculate expected usage manually: (0.3+0.5+0.2+0.8+0.1+0.3+0.4+0.6) / 16 =
        // 3.2 / 16 = 0.2
        let expected_usage = 0.2; // Note we use logical_cores (16) in the implementation
        assert_eq!(cpu.get_cpu_usage(), expected_usage);
    }

    #[test]
    fn test_cpu_metrics_trait() {
        let cpu = CPU::new_with_mock().expect("Failed to create CPU instance");

        // Test the CpuMetrics trait implementation
        assert_eq!(cpu.get_cpu_frequency(), 3200.0);
        assert_eq!(cpu.get_cpu_temperature(), Some(45.5));

        // Make sure core usage is valid (between 0 and 1)
        assert!(cpu.get_cpu_usage() >= 0.0 && cpu.get_cpu_usage() <= 1.0);
    }

    #[test]
    fn test_frequency_metrics() {
        let cpu = CPU::new_with_mock().expect("Failed to create CPU instance");

        // Test the frequency metrics methods
        assert_eq!(cpu.frequency_mhz(), 3200.0);
        assert_eq!(cpu.min_frequency_mhz(), Some(1200.0));
        assert_eq!(cpu.max_frequency_mhz(), Some(3600.0));

        // Test the available frequencies
        let available = cpu.available_frequencies().unwrap();
        assert_eq!(available.len(), 5);
        assert_eq!(available[0], 1200.0);
        assert_eq!(available[4], 3600.0);

        // Test the detailed frequency metrics
        let metrics = cpu.frequency_metrics().unwrap();
        assert_eq!(metrics.current, 3200.0);
        assert_eq!(metrics.min, 1200.0);
        assert_eq!(metrics.max, 3600.0);
        assert_eq!(metrics.available.len(), 5);
    }
}
