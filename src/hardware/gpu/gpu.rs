use super::{MAX_GPU_MEMORY, MAX_UTILIZATION};
use crate::hardware::iokit::{IOKit, IOKitImpl};
use crate::utils::{autorelease_pool, objc_safe_exec};
use crate::{Error, Result};
use objc2::msg_send;
use objc2::runtime::AnyObject;
use objc2_foundation::NSString;
use std::ffi::c_void;

type MTLDeviceRef = *mut c_void;

#[link(name = "Metal", kind = "framework")]
extern "C" {
    fn MTLCreateSystemDefaultDevice() -> MTLDeviceRef;
}

#[derive(Debug, Clone)]
pub struct GPUMemoryInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

#[derive(Debug, Clone, Default)]
pub struct GpuMetrics {
    pub utilization: f64,
    pub temperature: f64,
    pub memory_used: u64,
    pub memory_total: u64,
}

impl GpuMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn charts(&self) -> Vec<Chart> {
        vec![
            Chart::new("GPU Utilization")
                .with_unit("%")
                .with_value(self.utilization),
            Chart::new("GPU Temperature")
                .with_unit("°C")
                .with_value(self.temperature),
            Chart::new("GPU Memory Usage")
                .with_unit("MB")
                .with_value((self.memory_used / 1024 / 1024) as f64)
                .with_max((self.memory_total / 1024 / 1024) as f64),
        ]
    }
}

#[derive(Debug)]
pub struct GPU {
    device: MTLDeviceRef,
    iokit: Box<dyn IOKit>,
}

impl GPU {
    pub fn new() -> Result<Self> {
        let device = unsafe { MTLCreateSystemDefaultDevice() };
        if device.is_null() {
            return Err(Error::NotAvailable("No GPU device found".into()));
        }
        let iokit = Box::new(IOKitImpl::default());
        Ok(Self { device, iokit })
    }
}

impl GpuMetrics for GPU {
    fn get_metrics(&self) -> Result<GpuMetrics> {
    }

    fn memory_info(&self) -> Result<GPUMemoryInfo> {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_metrics() {
    }
}
