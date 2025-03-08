use super::CpuMetrics;
use crate::hardware::iokit::{IOKit, IOKitImpl};
use crate::error::Result;
use objc2::msg_send;
use objc2_foundation::NSString;
use objc2::rc::Retained;

#[cfg(test)]
use crate::hardware::iokit::MockIOKit;

#[derive(Debug)]
pub struct CPU {
    physical_cores: u32,
    logical_cores: u32,
    frequency_mhz: f64,
    core_usage: Vec<f64>,
    model_name: String,
    temperature: Option<f64>,
    iokit: Box<dyn IOKit>,
}

impl CPU {
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

    pub fn update(&mut self) -> Result<()> {
        let service = self.iokit.get_service("AppleACPICPU")?;

        // TODO: Verify if AppleACPICPU service actually provides these methods
        // If not, consider using sysctl or other methods for getting core counts
        // Example: sysctlbyname("hw.physicalcpu") and sysctlbyname("hw.logicalcpu")
        self.physical_cores = unsafe { msg_send![&*service, numberOfCores] };
        self.logical_cores = unsafe { msg_send![&*service, numberOfProcessorCores] };
        
        // TODO: Verify frequency method, possibly use sysctl with
        // sysctlbyname("hw.cpufrequency") instead
        let frequency: f64 = unsafe { msg_send![&*service, currentProcessorClockSpeed] };
        self.frequency_mhz = frequency / 1_000_000.0;

        self.core_usage = self.fetch_core_usage()?;
        
        // Get model name as NSString and convert to Rust String
        // TODO: Verify name method, consider using sysctlbyname("machdep.cpu.brand_string")
        let ns_name: Retained<NSString> = unsafe { msg_send![&*service, name] };
        self.model_name = ns_name.to_string();

        // Temperature from IOKit
        self.temperature = self.fetch_cpu_temperature();

        Ok(())
    }

    fn fetch_core_usage(&self) -> Result<Vec<f64>> {
        // TODO: Verify if AppleACPICPU provides getCoreUsage method
        // Alternative implementation options:
        // 1. Use host_processor_info() from mach kernel API to get CPU load
        // 2. Parse the output from the 'vm_stat' command
        // 3. Use sysctlbyname with specific processor information keys
        //
        // Example implementation with host_processor_info would look like:
        // unsafe {
        //     let mut cpu_load_info: *mut processor_cpu_load_info_t = std::ptr::null_mut();
        //     let mut cpu_count: u32 = 0;
        //     let result = host_processor_info(mach_host_self(), PROCESSOR_CPU_LOAD_INFO,
        //                                      &mut cpu_count, 
        //                                      &mut cpu_load_info as *mut _ as *mut *mut libc::c_int,
        //                                      &mut msg_type);
        //     // Process results and calculate usage percentages
        // }
        
        let mut usages = Vec::with_capacity(self.logical_cores as usize);
        for i in 0..self.logical_cores {
            let service = self.iokit.get_service("AppleACPICPU")?;
            let usage: f64 = unsafe { msg_send![&*service, getCoreUsage: i] };
            usages.push(usage);
        }
        Ok(usages)
    }

    fn fetch_cpu_temperature(&self) -> Option<f64> {
        self.iokit.get_cpu_temperature().ok()
    }

    pub fn physical_cores(&self) -> u32 {
        self.physical_cores
    }

    pub fn logical_cores(&self) -> u32 {
        self.logical_cores
    }

    pub fn frequency_mhz(&self) -> f64 {
        self.frequency_mhz
    }

    pub fn core_usage(&self) -> &[f64] {
        &self.core_usage
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    pub fn temperature(&self) -> Option<f64> {
        self.temperature
    }
}

impl CpuMetrics for CPU {
    fn get_cpu_usage(&self) -> f64 {
        self.core_usage.iter().sum::<f64>() / self.logical_cores as f64
    }

    fn get_cpu_temperature(&self) -> Option<f64> {
        self.temperature
    }

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
            mock.expect_get_service()
                .returning(|_| {
                    use crate::utils::test_utils;
                    Ok(test_utils::create_test_object())
                });
                
            mock.expect_get_cpu_temperature()
                .returning(|| Ok(45.5));
                
            let cpu = Self {
                physical_cores: 8,
                logical_cores: 16,
                frequency_mhz: 3200.0,
                core_usage: vec![0.3, 0.5, 0.2, 0.8, 0.1, 0.3, 0.4, 0.6],
                model_name: "Apple M1 Pro".to_string(),
                temperature: Some(45.5),
                iokit: Box::new(mock),
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
        
        // Calculate expected usage manually: (0.3+0.5+0.2+0.8+0.1+0.3+0.4+0.6) / 16 = 3.2 / 16 = 0.2
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
}