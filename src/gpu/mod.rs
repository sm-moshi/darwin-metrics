//! GPU metrics collection for macOS
//!
//! This module provides access to GPU metrics using the Metal framework.

use crate::iokit::{IOKit, IOKitImpl};
use crate::{Error, Result};
use objc2::runtime::AnyObject;
use objc2::msg_send;
use objc2_foundation::NSString;
use std::ffi::c_void;

// Metal framework types and functions
type MTLDeviceRef = *mut c_void;

#[link(name = "Metal", kind = "framework")]
unsafe extern "C" {
    fn MTLCreateSystemDefaultDevice() -> MTLDeviceRef;
}

/// Represents GPU memory usage statistics
#[derive(Debug, Clone)]
pub struct GPUMemoryInfo {
    /// Total GPU memory in bytes
    pub total: u64,
    /// Used GPU memory in bytes
    pub used: u64,
    /// Free GPU memory in bytes
    pub free: u64,
}

/// Represents GPU performance metrics
#[derive(Debug, Clone)]
pub struct GPUMetrics {
    /// GPU utilization percentage (0-100)
    pub utilization: f32,
    /// GPU memory information
    pub memory: GPUMemoryInfo,
    /// GPU temperature in Celsius
    pub temperature: Option<f32>,
    /// GPU power usage in watts
    pub power_usage: Option<f32>,
    /// GPU name/model
    pub name: String,
}

/// GPU information collector
pub struct GPU {
    device: MTLDeviceRef,
    iokit: Box<dyn IOKit>,
}

impl GPU {
    /// Create a new GPU metrics collector
    pub fn new() -> Result<Self> {
        unsafe {
            let device = MTLCreateSystemDefaultDevice();
            if device.is_null() {
                return Err(Error::not_available("No Metal-capable GPU found"));
            }

            let iokit = Box::<IOKitImpl>::default();

            Ok(Self { device, iokit })
        }
    }

    /// Get the name of the GPU
    fn get_name(&self) -> Result<String> {
        unsafe {
            if self.device.is_null() {
                return Err(Error::not_available("No Metal device available"));
            }

            // Convert the MTLDevice pointer to an AnyObject
            let device: &AnyObject = &*(self.device as *const AnyObject);

            // Get the name using objc2's safe interface
            let name: *mut AnyObject = msg_send![device, name];
            if name.is_null() {
                return Err(Error::not_available("Could not get GPU name"));
            }

            let ns_string: &NSString = &*(name.cast());
            Ok(ns_string.to_string())
        }
    }

    /// Get current GPU metrics
    pub fn get_metrics(&self) -> Result<GPUMetrics> {
        let name = self.get_name()?;

        // Get GPU memory info using IOKit
        let memory = self.get_memory_info()?;

        // Get GPU utilization
        let utilization = self.get_utilization()?;

        // Get temperature if available
        let temperature = self.get_temperature().ok();

        // Get power usage if available
        let power_usage = self.get_power_usage().ok();

        Ok(GPUMetrics {
            utilization,
            memory,
            temperature,
            power_usage,
            name,
        })
    }

    /// Get GPU memory information
    fn get_memory_info(&self) -> Result<GPUMemoryInfo> {
        // Get GPU memory info using IOKit
        let matching = self.iokit.io_service_matching("IOAccelerator");
        let service = self.iokit.io_service_get_matching_service(&matching);

        let Some(service) = service else {
            return Err(Error::not_available("Could not find GPU service"));
        };

        let _properties = self.iokit.io_registry_entry_create_cf_properties(&service)?;

        // For now return placeholder values until we implement the full memory info collection
        // TODO: Parse the properties dictionary to get actual memory values
        Ok(GPUMemoryInfo {
            total: 0,
            used: 0,
            free: 0,
        })
    }

    /// Get GPU utilization percentage
    fn get_utilization(&self) -> Result<f32> {
        // Implementation using IOKit to get GPU utilization
        Ok(0.0) // TODO: Implement actual utilization collection
    }

    /// Get GPU temperature
    fn get_temperature(&self) -> Result<f32> {
        // Implementation using IOKit to get temperature
        Err(Error::not_implemented("GPU temperature monitoring"))
    }

    /// Get GPU power usage
    fn get_power_usage(&self) -> Result<f32> {
        // Implementation using IOKit to get power usage
        Err(Error::not_implemented("GPU power usage monitoring"))
    }
}

impl Drop for GPU {
    fn drop(&mut self) {
        // No explicit cleanup needed for Metal device
        // It's managed by the Metal framework
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;
    use objc2_foundation::{NSObject, NSDictionary};
    use objc2::{msg_send, class};
    use objc2::rc::Retained;

    #[test]
    fn test_gpu_creation() -> Result<()> {
        let gpu = GPU::new()?;
        assert!(!gpu.device.is_null());
        Ok(())
    }

    #[test]
    fn test_metrics_collection() -> Result<()> {
        let mut mock_iokit = crate::iokit::MockIOKit::new();
        
        // Setup mock for service matching
        mock_iokit.expect_io_service_matching()
            .returning(|_| unsafe {
                let dict: *mut AnyObject = msg_send![class!(NSDictionary), new];
                Retained::from_raw(dict.cast()).unwrap()
            });

        // Setup mock for getting service
        mock_iokit.expect_io_service_get_matching_service()
            .returning(|_| unsafe {
                let obj: *mut AnyObject = msg_send![class!(NSObject), new];
                Some(Retained::from_raw(obj).unwrap())
            });

        // Setup mock for getting properties
        mock_iokit.expect_io_registry_entry_create_cf_properties()
            .returning(|_| unsafe {
                let dict: *mut AnyObject = msg_send![class!(NSDictionary), new];
                Ok(Retained::from_raw(dict.cast()).unwrap())
            });

        let gpu = unsafe {
            GPU {
                device: MTLCreateSystemDefaultDevice(),
                iokit: Box::new(mock_iokit),
            }
        };

        let metrics = gpu.get_metrics()?;
        assert!(!metrics.name.is_empty());
        Ok(())
    }

    #[test]
    fn test_gpu_memory_info() -> Result<()> {
        let mut mock_iokit = crate::iokit::MockIOKit::new();
        
        // Setup mock for service matching
        mock_iokit.expect_io_service_matching()
            .returning(|_| unsafe {
                let dict: *mut AnyObject = msg_send![class!(NSDictionary), new];
                Retained::from_raw(dict.cast()).unwrap()
            });

        // Setup mock for getting service
        mock_iokit.expect_io_service_get_matching_service()
            .returning(|_| unsafe {
                let obj: *mut AnyObject = msg_send![class!(NSObject), new];
                Some(Retained::from_raw(obj).unwrap())
            });

        // Setup mock for getting properties
        mock_iokit.expect_io_registry_entry_create_cf_properties()
            .returning(|_| unsafe {
                let dict: *mut AnyObject = msg_send![class!(NSDictionary), new];
                Ok(Retained::from_raw(dict.cast()).unwrap())
            });

        let gpu = unsafe {
            GPU {
                device: MTLCreateSystemDefaultDevice(),
                iokit: Box::new(mock_iokit),
            }
        };

        let memory = gpu.get_memory_info()?;
        // Since we're returning placeholder values for now
        assert_eq!(memory.total, 0);
        assert_eq!(memory.used, 0);
        assert_eq!(memory.free, 0);
        Ok(())
    }
}
