use libc::sysctlbyname;
use objc2::msg_send;
use objc2::runtime::AnyObject;
use std::{ffi::CString, ptr};

use super::{CpuMetrics, CpuMetricsData};
use crate::core::metrics::Metric;
use crate::core::types::Temperature;
use crate::hardware::iokit::IOKit;
use crate::utils::ffi;
#[cfg(test)]
use crate::utils::tests::test_utils::MockIOKit;
use crate::{Error, Result};
use crate::{FrequencyMetrics, FrequencyMonitor};

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
    pub fn new(iokit: Box<dyn IOKit>) -> Self {
        Self {
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
        }
    }

    pub fn update(&mut self) -> Result<()> {
        self.physical_cores = self.iokit.get_physical_cores()? as u32;
        self.logical_cores = self.iokit.get_logical_cores()? as u32;

        let physical_core_usage = self.iokit.get_core_usage()?;
        self.core_usage = Vec::with_capacity(self.logical_cores as usize);

        for i in 0..self.logical_cores {
            let physical_core_index = (i % self.physical_cores) as usize;
            let usage = physical_core_usage.get(physical_core_index).copied().unwrap_or(0.0);
            self.core_usage.push(usage);
        }

        self.update_frequency_information()?;

        Ok(())
    }

    fn update_frequency_information(&mut self) -> Result<()> {
        match self.frequency_monitor.get_metrics() {
            Ok(metrics) => {
                self.frequency_mhz = metrics.current;
                self.frequency_metrics = Some(metrics);
                Ok(())
            },
            Err(primary_error) => match self.get_frequency_from_iokit() {
                Ok(frequency_mhz) => {
                    self.frequency_mhz = frequency_mhz;
                    self.frequency_metrics = None;
                    Ok(())
                },
                Err(fallback_error) => Err(Error::system(format!(
                    "Failed to retrieve CPU frequency: primary method error: {}, fallback method error: {}",
                    primary_error, fallback_error
                ))),
            },
        }
    }

    fn get_frequency_from_iokit(&self) -> Result<f64> {
        let service = self.iokit.get_service_matching("AppleACPICPU")?;
        let _service_ref = service.as_ref().ok_or_else(|| Error::iokit_error(0, "Failed to get CPU service"))?;

        let obj = unsafe {
            let ptr = _service_ref as *const _ as *mut AnyObject;
            &*ptr
        };
        let frequency: f64 = unsafe { msg_send![obj, currentProcessorClockSpeed] };

        Ok(if frequency > 1000.0 { frequency / 1000.0 } else { frequency })
    }

    fn fetch_core_usage(&self) -> Result<Vec<f64>> {
        let service = self.iokit.get_service_matching("AppleACPICPU")?;
        let _service_ref = service.ok_or_else(|| Error::iokit_error(0, "Failed to get CPU service"))?;

        let mut usages = Vec::with_capacity(self.logical_cores as usize);

        for i in 0..self.logical_cores {
            let physical_core_index = i % self.physical_cores;
            let usage = self.iokit.get_core_usage()?.get(physical_core_index as usize).copied().unwrap_or(0.0);
            usages.push(usage);
        }

        Ok(usages)
    }

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

    pub fn power(&self) -> Option<f64> {
        self.power
    }

    pub fn frequency_metrics(&self) -> Option<&FrequencyMetrics> {
        self.frequency_metrics.as_ref()
    }

    pub fn min_frequency_mhz(&self) -> Option<f64> {
        self.frequency_metrics.as_ref().map(|m| m.min)
    }

    pub fn max_frequency_mhz(&self) -> Option<f64> {
        self.frequency_metrics.as_ref().map(|m| m.max)
    }

    pub fn available_frequencies(&self) -> Option<&[f64]> {
        self.frequency_metrics.as_ref().map(|m| m.available.as_slice())
    }

    pub fn metrics(&self) -> Result<CpuMetricsData> {
        Ok(CpuMetricsData {
            usage: self.get_core_usage(),
            temperature: self.get_cpu_temperature(),
            frequency: self.get_cpu_frequency(),
        })
    }

    /// Clone method that creates a new CPU instance with a cloned IOKit box
    pub fn clone(&self) -> Self {
        Self {
            physical_cores: self.physical_cores,
            logical_cores: self.logical_cores,
            frequency_mhz: self.frequency_mhz,
            core_usage: self.core_usage.clone(),
            model_name: self.model_name.clone(),
            temperature: self.temperature,
            power: self.power,
            iokit: self.iokit.clone_box(),
            frequency_monitor: FrequencyMonitor::new(),
            frequency_metrics: self.frequency_metrics.clone(),
        }
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
}

impl CpuMetrics for CPU {
    fn get_core_usage(&self) -> f64 {
        let total_usage: f64 = self.core_usage.iter().sum();
        if self.logical_cores > 0 {
            total_usage / self.logical_cores as f64
        } else {
            0.0
        }
    }

    fn get_cpu_temperature(&self) -> Option<f64> {
        self.temperature
    }

    fn get_cpu_frequency(&self) -> f64 {
        self.frequency_mhz
    }
}
