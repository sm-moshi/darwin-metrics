//! CPU metrics and information for macOS systems.
//!
//! This module provides functionality to gather CPU-related metrics and information
//! on macOS systems using IOKit and Foundation frameworks. It supports both Intel
//! and Apple Silicon processors.
//!
//! # Examples
//!
//! ```no_run
//! use darwin_metrics::prelude::*;
//!
//! fn main() -> darwin_metrics::Result<()> {
//!     let cpu = CPU::new()?;
//!     println!("CPU Model: {}", cpu.model_name());
//!     println!("Physical cores: {}", cpu.physical_cores());
//!     println!("Current frequency: {} MHz", cpu.frequency_mhz());
//!     println!("Average CPU usage: {}%", cpu.average_usage());
//!     Ok(())
//! }
//! ```

use crate::iokit::{IOKit, IOKitImpl};
use crate::{Error, Result};
use objc2::runtime::AnyObject;
use objc2::{msg_send, class};
use objc2_foundation::NSNumber;

/// Represents CPU information and metrics for macOS systems.
///
/// This struct provides access to various CPU metrics and information, including:
/// - Core count (physical and logical)
/// - CPU frequency
/// - Per-core usage statistics
/// - CPU model information
/// - Temperature data (when available)
///
/// The struct maintains its own state and can be updated to fetch the latest metrics.
///
/// # Examples
///
/// ```no_run
/// use darwin_metrics::prelude::*;
///
/// let mut cpu = CPU::new()?;
/// 
/// // Get current metrics
/// println!("CPU Usage: {}%", cpu.average_usage());
/// 
/// // Update metrics
/// cpu.update()?;
/// 
/// // Get per-core usage
/// for (i, usage) in cpu.core_usage().iter().enumerate() {
///     println!("Core {}: {}%", i, usage);
/// }
/// # Ok::<(), darwin_metrics::Error>(())
/// ```
#[derive(Debug)]
pub struct CPU {
    #[cfg(not(test))]
    physical_cores: u32,
    #[cfg(test)]
    pub(crate) physical_cores: u32,
    
    #[cfg(not(test))]
    logical_cores: u32,
    #[cfg(test)]
    pub(crate) logical_cores: u32,
    
    #[cfg(not(test))]
    frequency_mhz: f64,
    #[cfg(test)]
    pub(crate) frequency_mhz: f64,
    
    #[cfg(not(test))]
    core_usage: Vec<f64>,
    #[cfg(test)]
    pub(crate) core_usage: Vec<f64>,
    
    #[cfg(not(test))]
    model_name: String,
    #[cfg(test)]
    pub(crate) model_name: String,
    
    #[cfg(not(test))]
    temperature: Option<f64>,
    #[cfg(test)]
    pub(crate) temperature: Option<f64>,
    
    #[cfg(not(test))]
    iokit: Box<dyn IOKit>,
    #[cfg(test)]
    pub(crate) iokit: Box<dyn IOKit>,
}

impl CPU {
    /// Creates a new CPU instance and initializes it with system values.
    ///
    /// This function creates a new CPU instance and immediately populates it with
    /// current system metrics. It uses the IOKit framework to gather hardware
    /// information and the Foundation framework for performance metrics.
    ///
    /// # Returns
    ///
    /// Returns a `Result<CPU>` which is:
    /// - `Ok(CPU)` containing the initialized CPU instance
    /// - `Err(Error)` if system metrics cannot be gathered
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use darwin_metrics::prelude::*;
    ///
    /// let cpu = CPU::new()?;
    /// println!("CPU Model: {}", cpu.model_name());
    /// # Ok::<(), darwin_metrics::Error>(())
    /// ```
    pub fn new() -> Result<Self> {
        let mut cpu = Self {
            physical_cores: 0,
            logical_cores: 0,
            frequency_mhz: 0.0,
            core_usage: Vec::new(),
            model_name: String::new(),
            temperature: None,
            iokit: Box::new(IOKitImpl::default()),
        };
        cpu.update()?;
        Ok(cpu)
    }

    /// Updates CPU metrics from the system.
    ///
    /// This method refreshes all CPU metrics including:
    /// - Core counts
    /// - CPU frequency
    /// - Core usage percentages
    /// - Temperature (if available)
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
    /// let mut cpu = CPU::new()?;
    /// cpu.update()?;
    /// println!("Current CPU frequency: {} MHz", cpu.frequency_mhz());
    /// # Ok::<(), darwin_metrics::Error>(())
    /// ```
    pub fn update(&mut self) -> Result<()> {
        unsafe {
            // Get processor info using NSProcessInfo
            let process_info: *mut AnyObject = msg_send![class!(NSProcessInfo), processInfo];
            if process_info.is_null() {
                return Err(Error::not_available("Could not get process info"));
            }

            // Get processor count
            let physical_cores: u32 = msg_send![process_info, activeProcessorCount];
            self.physical_cores = physical_cores;
            self.logical_cores = physical_cores; // On Apple Silicon, these are the same

            // Get CPU frequency
            let freq: f64 = msg_send![process_info, processorFrequency];
            self.frequency_mhz = freq / 1_000_000.0; // Convert Hz to MHz

            // Get CPU model name using IOKit
            let matching = self.iokit.io_service_matching("IOPlatformExpertDevice");
            let service = self.iokit.io_service_get_matching_service(&matching);

            if let Some(service) = service {
                let properties = self.iokit.io_registry_entry_create_cf_properties(&service)?;
                if let Some(name) = self.iokit.get_string_property(&properties, "cpu-type") {
                    self.model_name = name;
                }

                // Try to get CPU temperature if available
                if let Some(temp) = self.iokit.get_number_property(&properties, "cpu-die-temperature") {
                    self.temperature = Some(temp as f64 / 100.0); // Convert to Celsius
                }
            }

            // Get CPU usage per core
            self.update_cpu_usage()?;
        }

        Ok(())
    }

    /// Updates CPU usage statistics for all cores.
    ///
    /// This internal method refreshes the per-core CPU usage percentages
    /// using the Foundation framework's host statistics API.
    ///
    /// # Returns
    ///
    /// Returns a `Result<()>` which is:
    /// - `Ok(())` if usage statistics were successfully updated
    /// - `Err(Error)` if statistics could not be gathered
    fn update_cpu_usage(&mut self) -> Result<()> {
        unsafe {
            // Get host statistics using NSHost
            let host: *mut AnyObject = msg_send![class!(NSHost), currentHost];
            if host.is_null() {
                return Err(Error::not_available("Could not get host info"));
            }

            // Get CPU load info
            let cpu_load: *mut AnyObject = msg_send![host, cpuLoadInfo];
            if cpu_load.is_null() {
                return Err(Error::not_available("Could not get CPU load info"));
            }

            // Get usage percentages
            let usage_array: *mut AnyObject = msg_send![cpu_load, cpuUsagePerProcessor];
            let count: usize = msg_send![usage_array, count];

            self.core_usage.clear();
            for i in 0..count {
                let usage: *mut NSNumber = msg_send![usage_array, objectAtIndex:i];
                let value: f64 = msg_send![usage, doubleValue];
                self.core_usage.push(value.clamp(0.0, 100.0));
            }
        }

        Ok(())
    }

    /// Gets the current CPU information.
    ///
    /// Returns a copy of the current CPU instance with all its metrics.
    ///
    /// # Returns
    ///
    /// Returns a `Result<CPU>` containing a clone of the current instance.
    pub fn get_info(&self) -> Result<Self> {
        Ok(self.clone())
    }

    /// Calculates the average CPU usage across all cores.
    ///
    /// This method computes the arithmetic mean of all core usage percentages.
    /// The result is clamped between 0.0 and 100.0.
    ///
    /// # Returns
    ///
    /// Returns a `f64` representing the average CPU usage percentage.
    /// Returns 0.0 if no core usage data is available.
    pub fn average_usage(&self) -> f64 {
        if self.core_usage.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.core_usage.iter().sum();
        (sum / self.core_usage.len() as f64).clamp(0.0, 100.0)
    }

    /// Gets the number of physical CPU cores.
    ///
    /// # Returns
    ///
    /// Returns a `u32` representing the number of physical CPU cores.
    pub fn physical_cores(&self) -> u32 {
        self.physical_cores
    }

    /// Gets the number of logical CPU cores.
    ///
    /// On Apple Silicon, this is typically the same as physical cores.
    /// On Intel processors, this may include virtual cores (Hyper-Threading).
    ///
    /// # Returns
    ///
    /// Returns a `u32` representing the number of logical CPU cores.
    pub fn logical_cores(&self) -> u32 {
        self.logical_cores
    }

    /// Gets the current CPU frequency in MHz.
    ///
    /// # Returns
    ///
    /// Returns a `f64` representing the current CPU frequency in MHz.
    pub fn frequency_mhz(&self) -> f64 {
        self.frequency_mhz
    }

    /// Gets a slice of core usage percentages.
    ///
    /// Returns the current CPU usage percentage for each core.
    /// Each value is clamped between 0.0 and 100.0.
    ///
    /// # Returns
    ///
    /// Returns a slice of `f64` values representing per-core CPU usage percentages.
    pub fn core_usage(&self) -> &[f64] {
        &self.core_usage
    }

    /// Gets the CPU model name.
    ///
    /// # Returns
    ///
    /// Returns a string slice containing the CPU model name.
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Gets the CPU temperature in Celsius if available.
    ///
    /// Not all systems provide temperature information.
    ///
    /// # Returns
    ///
    /// Returns an `Option<f64>` which is:
    /// - `Some(temperature)` containing the CPU temperature in Celsius
    /// - `None` if temperature information is not available
    pub fn temperature(&self) -> Option<f64> {
        self.temperature
    }
}

impl Clone for CPU {
    fn clone(&self) -> Self {
        Self {
            physical_cores: self.physical_cores,
            logical_cores: self.logical_cores,
            frequency_mhz: self.frequency_mhz,
            core_usage: self.core_usage.clone(),
            model_name: self.model_name.clone(),
            temperature: self.temperature,
            iokit: Box::new(IOKitImpl::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iokit::MockIOKit; // This is now re-exported from iokit module
    

    fn setup_test_environment() {
        // No setup needed for now, but keeping the function for consistency
    }

    fn create_test_cpu() -> CPU {
        let mut mock_iokit = MockIOKit::new();
        mock_iokit.expect_io_service_matching()
            .returning(|_| unsafe { 
                let dict: *mut objc2::runtime::AnyObject = objc2::msg_send![objc2::class!(NSDictionary), new];
                objc2::rc::Retained::from_raw(dict.cast()).unwrap()
            });
        
        CPU {
            physical_cores: 4,
            logical_cores: 8,
            frequency_mhz: 2400.0,
            core_usage: vec![50.0, 60.0, 70.0, 70.0],
            model_name: "Test CPU".to_string(),
            temperature: Some(45.0),
            iokit: Box::new(mock_iokit),
        }
    }

    #[test]
    fn test_new_cpu() {
        setup_test_environment();
        let cpu = create_test_cpu();
        
        // Test basic properties
        assert_eq!(cpu.physical_cores(), 4);
        assert_eq!(cpu.logical_cores(), 8);
        assert_eq!(cpu.frequency_mhz(), 2400.0);
        assert_eq!(cpu.model_name(), "Test CPU");
    }

    #[test]
    fn test_average_usage() {
        setup_test_environment();
        let cpu = create_test_cpu();
        assert_eq!(cpu.average_usage(), 62.5);
    }

    #[test]
    fn test_empty_usage() {
        setup_test_environment();
        // Create a CPU with empty usage data
        let mut cpu = create_test_cpu();
        cpu.core_usage = vec![];
        assert_eq!(cpu.average_usage(), 0.0);

    }
}

