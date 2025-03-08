mod frequency;

pub use frequency::FrequencyMetrics;

pub const MAX_CORES: u32 = 64;
pub const MAX_FREQUENCY_MHZ: f64 = 5000.0;

pub trait CpuMetrics {
    fn get_cpu_usage(&self) -> f64;
    fn get_cpu_temperature(&self) -> Option<f64>;
    fn get_cpu_frequency(&self) -> f64;
}

use crate::error::{Error, Result};
use objc2::runtime::AnyObject;
use objc2::{class, msg_send};
use objc2_foundation::NSNumber;

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
    iokit: Box<dyn crate::hardware::iokit::IOKit>,
    #[cfg(test)]
    pub(crate) iokit: Box<dyn crate::hardware::iokit::IOKit>,
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
            iokit: Box::new(crate::hardware::iokit::IOKitImpl::default()),
        };
        cpu.update()?;
        Ok(cpu)
    }

    pub fn update(&mut self) -> Result<()> {
        unsafe {
            let process_info: *mut AnyObject = msg_send![class!(NSProcessInfo), processInfo];
            if process_info.is_null() {
                return Err(Error::not_available("Could not get process info"));
            }

            let physical_cores: u32 = msg_send![process_info, activeProcessorCount];
            self.physical_cores = physical_cores;
            self.logical_cores = physical_cores;

            let freq: f64 = msg_send![process_info, processorFrequency];
            self.frequency_mhz = freq / 1_000_000.0;

            let matching = self.iokit.io_service_matching("IOPlatformExpertDevice");
            let service = self.iokit.io_service_get_matching_service(&matching);

            if let Some(service) = service {
                let properties = self
                    .iokit
                    .io_registry_entry_create_cf_properties(&service)?;
                if let Some(name) = self.iokit.get_string_property(&properties, "cpu-type") {
                    self.model_name = name;
                }

                if let Some(temp) = self
                    .iokit
                    .get_number_property(&properties, "cpu-die-temperature")
                {
                    self.temperature = Some(temp as f64 / 100.0);
                }
            }

            self.update_cpu_usage()?;
        }

        Ok(())
    }

    fn update_cpu_usage(&mut self) -> Result<()> {
        unsafe {
            let host: *mut AnyObject = msg_send![class!(NSHost), currentHost];
            if host.is_null() {
                return Err(Error::not_available("Could not get host info"));
            }

            let cpu_load: *mut AnyObject = msg_send![host, cpuLoadInfo];
            if cpu_load.is_null() {
                return Err(Error::not_available("Could not get CPU load info"));
            }

            let usage_array: *mut AnyObject = msg_send![cpu_load, cpuUsagePerProcessor];
            let count: usize = msg_send![usage_array, count];

            self.core_usage.clear();
            for i in 0..count {
                let usage: *mut NSNumber = msg_send![usage_array, objectAtIndex:i];
                let value: f64 = msg_send![usage, doubleValue];
                self.core_usage.push(value);
            }
        }

        Ok(())
    }

    pub fn get_info(&self) -> Result<Self> {
        Ok(self.clone())
    }

    pub fn average_usage(&self) -> f64 {
        if self.core_usage.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.core_usage.iter().sum();
        sum / self.core_usage.len() as f64
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
        self.average_usage()
    }

    fn get_cpu_temperature(&self) -> Option<f64> {
        self.temperature
    }

    fn get_cpu_frequency(&self) -> f64 {
        self.frequency_mhz
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
            iokit: Box::new(crate::hardware::iokit::IOKitImpl::default()),
        }
    }
}
