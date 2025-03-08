use crate::hardware::iokit::{IOKit, IOKitImpl};
use crate::utils::{autorelease_pool, objc_safe_exec};
use crate::error::{Error, Result};
use objc2::msg_send;
use objc2::runtime::AnyObject;
use std::ffi::c_void;

pub const MAX_GPU_MEMORY: u64 = 16 * 1024 * 1024 * 1024;
pub const MAX_UTILIZATION: f32 = 100.0;
type MTLDeviceRef = *mut c_void;

pub mod gpu;

#[link(name = "Metal", kind = "framework")]
extern "C" {
    fn MTLCreateSystemDefaultDevice() -> MTLDeviceRef;
}

pub trait GpuMetricsProvider {
    fn get_metrics(&self) -> Result<GpuMetrics>;
    fn name(&self) -> Result<String>;
}

#[derive(Debug, Clone, Default)]
pub struct GpuMemoryInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

#[derive(Debug, Clone, Default)]
pub struct GpuMetrics {
    pub utilization: f32,
    pub memory: GpuMemoryInfo,
    pub temperature: Option<f32>,
    pub power_usage: Option<f32>,
    pub name: String,
}

#[derive(Debug)]
pub struct GPU {
    iokit: Box<dyn IOKit>,
    // Keep the Metal device for future advanced metrics
    metal_device: Option<MTLDeviceRef>,
}

impl GPU {
    pub fn new() -> Result<Self> {
        let iokit = Box::new(IOKitImpl::default());
        
        // Create Metal device but don't use it for basic metrics
        let metal_device = unsafe { 
            let device = MTLCreateSystemDefaultDevice();
            if device.is_null() {
                None
            } else {
                Some(device)
            }
        };
        
        Ok(Self { iokit, metal_device })
    }

    pub fn name(&self) -> Result<String> {
        // Get GPU stats using IOKit
        let stats = self.iokit.get_gpu_stats()?;
        
        if !stats.name.is_empty() {
            return Ok(stats.name);
        }
        
        // Fallback to Metal API if needed
        if let Some(device) = self.metal_device {
            return autorelease_pool(|| {
                objc_safe_exec(|| {
                    unsafe {
                        let device_obj: *mut AnyObject = device.cast();
                        let name_obj: *mut AnyObject = msg_send![device_obj, name];
                        if name_obj.is_null() {
                            return Err(Error::NotAvailable("Could not get GPU name".into()));
                        }
                        let utf8_string: *const u8 = msg_send![name_obj, UTF8String];
                        if utf8_string.is_null() {
                            return Err(Error::NotAvailable("Could not convert GPU name to string".into()));
                        }
                        let c_str = std::ffi::CStr::from_ptr(utf8_string as *const i8);
                        let name = c_str.to_string_lossy().into_owned();
                        Ok(name)
                    }
                })
            });
        }
        
        // Final fallback
        Ok("Unknown GPU".to_string())
    }

    pub fn metrics(&self) -> Result<GpuMetrics> {
        // Get stats from IOKit
        let gpu_stats = self.iokit.get_gpu_stats()?;
        
        // Try to get temperature
        let temperature = match self.iokit.get_gpu_temperature() {
            Ok(temp) => Some(temp as f32),
            Err(_) => None
        };
        
        let memory = GpuMemoryInfo {
            total: gpu_stats.memory_total,
            used: gpu_stats.memory_used,
            free: gpu_stats.memory_total.saturating_sub(gpu_stats.memory_used),
        };
        
        // Fallback for memory if needed
        let memory = if memory.total == 0 {
            GpuMemoryInfo {
                total: MAX_GPU_MEMORY,
                used: 0,
                free: MAX_GPU_MEMORY,
            }
        } else {
            memory
        };
        
        Ok(GpuMetrics {
            name: gpu_stats.name,
            utilization: gpu_stats.utilization as f32,
            memory,
            temperature,
            power_usage: None, // We'll implement this in the future
        })
    }

    pub fn memory_info(&self) -> Result<GpuMemoryInfo> {
        let gpu_stats = self.iokit.get_gpu_stats()?;
        
        let total = if gpu_stats.memory_total > 0 {
            gpu_stats.memory_total
        } else {
            MAX_GPU_MEMORY
        };
        
        let used = gpu_stats.memory_used;
        let free = total.saturating_sub(used);
        
        Ok(GpuMemoryInfo { total, used, free })
    }

    pub fn utilization(&self) -> Result<f32> {
        let gpu_stats = self.iokit.get_gpu_stats()?;
        Ok(gpu_stats.utilization as f32)
    }

    pub fn temperature(&self) -> Result<f32> {
        let temp = self.iokit.get_gpu_temperature()?;
        Ok(temp as f32)
    }

    pub fn power_usage(&self) -> Result<f32> {
        // This will be implemented in the future
        Err(Error::not_implemented("GPU power usage monitoring"))
    }
}

// We need to manually implement Drop to release the Metal device
impl Drop for GPU {
    fn drop(&mut self) {
        if let Some(device) = self.metal_device.take() {
            if !device.is_null() {
                autorelease_pool(|| {
                    // We don't need to explicitly release Metal devices
                    // as they are managed by the Objective-C runtime
                });
            }
        }
    }
}

unsafe impl Send for GPU {}
unsafe impl Sync for GPU {}