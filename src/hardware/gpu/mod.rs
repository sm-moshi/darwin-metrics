use crate::hardware::iokit::{IOKit, IOKitImpl};
use crate::utils::{autorelease_pool, objc_safe_exec};
use crate::error::{Error, Result};
use objc2::msg_send;
use objc2::runtime::AnyObject;
use objc2_foundation::NSString;
use std::ffi::c_void;

pub const MAX_GPU_MEMORY: u64 = 16 * 1024 * 1024 * 1024;
pub const MAX_UTILIZATION: f32 = 100.0;
type MTLDeviceRef = *mut c_void;

#[link(name = "Metal", kind = "framework")]
extern "C" {
    fn MTLCreateSystemDefaultDevice() -> MTLDeviceRef;
}

pub trait GpuMetricsProvider {
    fn get_metrics(&self) -> Result<GpuMetrics>;
    fn name(&self) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct GpuMemoryInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

#[derive(Debug, Clone)]
pub struct GpuMetrics {
    pub utilization: f32,
    pub memory: GpuMemoryInfo,
    pub temperature: Option<f32>,
    pub power_usage: Option<f32>,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct GPUMemoryInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

#[derive(Debug, Clone)]
pub struct GPUMetrics {
    pub utilization: f32,
    pub memory: GPUMemoryInfo,
    pub temperature: Option<f32>,
    pub power_usage: Option<f32>,
    pub name: String,
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

    pub fn name(&self) -> Result<String> {
        if self.device.is_null() {
            return Err(Error::NotAvailable("No GPU device available".into()));
        }
        autorelease_pool(|| {
            objc_safe_exec(|| {
                unsafe {
                    let device_obj: *mut AnyObject = self.device.cast();
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
        })
    }

    pub fn metrics(&self) -> Result<GPUMetrics> {
        if self.device.is_null() {
            return Err(Error::NotAvailable("No Metal device available".into()));
        }
        let name = self.get_name()?;
        let memory = self.get_memory_info()?;
        let utilization = self.get_utilization()?;
        let temperature = self.get_temperature().ok();
        let power_usage = self.get_power_usage().ok();
        Ok(GPUMetrics {
            name,
            memory,
            utilization,
            temperature,
            power_usage,
        })
    }

    pub fn memory_info(&self) -> Result<GPUMemoryInfo> {
        let matching = self.iokit.io_service_matching("IOAccelerator");
        let service = self.iokit.io_service_get_matching_service(&matching);
        let Some(service) = service else {
            return Err(Error::not_available("Could not find GPU service"));
        };
        let _properties = self.iokit.io_registry_entry_create_cf_properties(&service)?;
        Ok(GPUMemoryInfo {
            total: 0,
            used: 0,
            free: 0,
        })
    }

    pub fn utilization(&self) -> Result<f32> {
        Ok(0.0)
    }

    pub fn temperature(&self) -> Result<f32> {
        Err(Error::not_implemented("GPU temperature monitoring"))
    }

    pub fn power_usage(&self) -> Result<f32> {
        Err(Error::not_implemented("GPU power usage monitoring"))
    }

    fn get_name(&self) -> Result<String> {
        if self.device.is_null() {
            return Err(Error::not_available("No Metal device available"));
        }
        objc_safe_exec(|| unsafe {
            let device = self.device as *const AnyObject;
            if device.is_null() {
                return Err(Error::not_available("Invalid Metal device"));
            }
            let device_ref = device
                .as_ref()
                .ok_or_else(|| Error::not_available("Invalid Metal device reference"))?;
            let name: *mut AnyObject = msg_send![device_ref, name];
            if name.is_null() {
                return Err(Error::not_available("Could not get GPU name"));
            }
            let ns_string = (name as *const NSString)
                .as_ref()
                .ok_or_else(|| Error::not_available("Invalid name string"))?;
            Ok(ns_string.to_string())
        })
    }

    fn get_metrics(&self) -> Result<GPUMetrics> {
        if self.device.is_null() {
            return Err(Error::not_available("No Metal device available"));
        }
        let name = self.get_name()?;
        let memory = self.get_memory_info()?;
        let utilization = self.get_utilization()?;
        let temperature = self.get_temperature().ok();
        let power_usage = self.get_power_usage().ok();
        Ok(GPUMetrics {
            name,
            memory,
            utilization,
            temperature,
            power_usage,
        })
    }

    fn get_memory_info(&self) -> Result<GPUMemoryInfo> {
        let matching = self.iokit.io_service_matching("IOAccelerator");
        let service = self.iokit.io_service_get_matching_service(&matching);
        let Some(service) = service else {
            return Err(Error::not_available("Could not find GPU service"));
        };
        let _properties = self.iokit.io_registry_entry_create_cf_properties(&service)?;
        Ok(GPUMemoryInfo {
            total: 0,
            used: 0,
            free: 0,
        })
    }

    fn get_utilization(&self) -> Result<f32> {
        Ok(0.0)
    }

    fn get_temperature(&self) -> Result<f32> {
        Err(Error::not_implemented("GPU temperature monitoring"))
    }

    fn get_power_usage(&self) -> Result<f32> {
        Err(Error::not_implemented("GPU power usage monitoring"))
    }
}

impl Drop for GPU {
    fn drop(&mut self) {
        if !self.device.is_null() {
            autorelease_pool(|| {
                self.device = std::ptr::null_mut();
            });
        }
    }
}

unsafe impl Send for GPU {}
unsafe impl Sync for GPU {}