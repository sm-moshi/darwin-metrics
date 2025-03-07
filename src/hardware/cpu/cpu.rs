use super::{CpuMetrics, MAX_CORES, MAX_FREQUENCY_MHZ};
use crate::hardware::iokit::{IOKit, IOKitImpl};
use crate::error::{Error, Result};
use objc2::runtime::AnyObject;
use objc2::{class, msg_send};
use objc2_foundation::NSNumber;

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

    fn update(&mut self) -> Result<()> {
        let service = self.iokit.get_service("AppleACPICPU")?;
        let physical_cores: u32 = msg_send![service, numberOfCores];
        let logical_cores: u32 = msg_send![service, numberOfProcessorCores];
        self.physical_cores = physical_cores;
        self.logical_cores = logical_cores;

        let freq: f64 = msg_send![service, currentProcessorClockSpeed];
        self.frequency_mhz = freq / 1000000.0;

        let usage: Vec<f64> = (0..logical_cores)
            .map(|i| {
                let core = self.iokit.get_service("AppleACPICPU")?.get_core(i);
                let usage: f64 = msg_send![core, getUsage];
                usage
            })
            .collect();
        self.core_usage = usage;

        let model_name: String = msg_send![service, name];
        self.model_name = model_name;

        let temperature: f64 = self.iokit.get_temperature()?;
        self.temperature = Some(temperature);

        Ok(())
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

    pub fn core_usage(&self) -> Vec<f64> {
        self.core_usage.clone()
    }

    pub fn model_name(&self) -> String {
        self.model_name.clone()
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

    #[test]
    fn test_cpu_metrics() {
        let cpu = CPU::new().unwrap();
        assert!(cpu.get_cpu_usage() >= 0.0);
        assert!(cpu.get_cpu_frequency() > 0.0);
    }
}
