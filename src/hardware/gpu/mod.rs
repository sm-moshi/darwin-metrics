//! GPU metrics collection for macOS
//!
//! This module provides access to GPU metrics using the Metal framework.
//!
//! # Examples
//!
//! ```no_run
//! use darwin_metrics::prelude::*;
//!
//! fn main() -> darwin_metrics::Result<()> {
//!     let gpu = GPU::new()?;
//!     
//!     // Get basic GPU information
//!     println!("GPU: {}", gpu.name()?);
//!     
//!     // Get comprehensive metrics
//!     let metrics = gpu.metrics()?;
//!     println!("Utilization: {}%", metrics.utilization);
//!     println!("Memory: {} MB used of {} MB total",
//!              metrics.memory.used / 1024 / 1024,
//!              metrics.memory.total / 1024 / 1024);
//!     
//!     if let Some(temp) = metrics.temperature {
//!         println!("Temperature: {}°C", temp);
//!     }
//!     
//!     Ok(())
//! }
//! ```

use crate::hardware::iokit::{IOKit, IOKitImpl};
use crate::utils::{autorelease_pool, objc_safe_exec};
use crate::{Error, Result};
use objc2::msg_send;
use objc2::runtime::AnyObject;
use objc2_foundation::NSString;
use std::ffi::c_void;

// Metal framework types and functions
type MTLDeviceRef = *mut c_void;

#[link(name = "Metal", kind = "framework")]
extern "C" {
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
///
/// Provides access to GPU information and performance metrics using the Metal framework.
/// This struct maintains a connection to the GPU and can be used to query various metrics.
///
/// # Thread Safety
///
/// This struct implements `Send` and `Sync` and can be safely shared across threads.
#[derive(Debug)]
pub struct GPU {
    device: MTLDeviceRef,
    iokit: Box<dyn IOKit>,
}

impl GPU {
    /// Creates a new GPU instance
    ///
    /// Initializes connection to the system GPU using Metal framework
    /// and sets up IOKit access for additional properties.
    ///
    /// # Returns
    ///
    /// A Result containing the GPU instance or an error if no GPU is available
    pub fn new() -> Result<Self> {
        // Get the default Metal device (GPU)
        let device = unsafe { MTLCreateSystemDefaultDevice() };

        // Check if we found a valid device
        if device.is_null() {
            return Err(Error::NotAvailable("No GPU device found".into()));
        }

        // Create the IOKit implementation
        let iokit = Box::new(IOKitImpl::default());

        Ok(Self { device, iokit })
    }

    /// Gets the name of the GPU
    ///
    /// # Returns
    ///
    /// A Result containing the GPU name as a String or an error
    pub fn name(&self) -> Result<String> {
        // Check if we have a valid device
        if self.device.is_null() {
            return Err(Error::NotAvailable("No GPU device available".into()));
        }

        // Safely execute Objective-C code within an autorelease pool
        autorelease_pool(|| {
            objc_safe_exec(|| {
                unsafe {
                    // Cast the device pointer to AnyObject before sending messages to it
                    let device_obj: *mut AnyObject = self.device.cast();

                    // Get the name property using Metal API
                    let name_obj: *mut AnyObject = msg_send![device_obj, name];

                    // Check if we got a valid name object
                    if name_obj.is_null() {
                        return Err(Error::NotAvailable("Could not get GPU name".into()));
                    }

                    // Convert to Rust string
                    let utf8_string: *const u8 = msg_send![name_obj, UTF8String];

                    if utf8_string.is_null() {
                        return Err(Error::NotAvailable(
                            "Could not convert GPU name to string".into(),
                        ));
                    }

                    // Convert C string to Rust string
                    let c_str = std::ffi::CStr::from_ptr(utf8_string as *const i8);
                    let name = c_str.to_string_lossy().into_owned();

                    Ok(name)
                }
            })
        })
    }

    /// Get comprehensive GPU metrics
    ///
    /// This includes utilization, memory usage, and optionally temperature and power usage.
    ///
    /// # Returns
    ///
    /// A `GPUMetrics` struct containing all available metrics.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use darwin_metrics::gpu::GPU;
    ///
    /// let gpu = GPU::new().expect("Failed to initialize GPU metrics");
    /// match gpu.metrics() {
    ///     Ok(metrics) => println!("GPU Utilization: {}%", metrics.utilization),
    ///     Err(err) => eprintln!("Failed to get GPU metrics: {}", err),
    /// }
    /// ```
    pub fn metrics(&self) -> Result<GPUMetrics> {
        self.get_metrics()
    }

    /// Get GPU memory information
    ///
    /// # Returns
    ///
    /// A `GPUMemoryInfo` struct containing memory usage statistics.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use darwin_metrics::gpu::GPU;
    ///
    /// let gpu = GPU::new().expect("Failed to initialize GPU metrics");
    /// match gpu.memory_info() {
    ///     Ok(memory) => println!("GPU Memory: {} MB used of {} MB total",
    ///                          memory.used / (1024 * 1024),
    ///                          memory.total / (1024 * 1024)),
    ///     Err(err) => eprintln!("Failed to get GPU memory info: {}", err),
    /// }
    /// ```
    pub fn memory_info(&self) -> Result<GPUMemoryInfo> {
        self.get_memory_info()
    }

    /// Get GPU utilization percentage
    ///
    /// # Returns
    ///
    /// The GPU utilization as a percentage (0-100).
    pub fn utilization(&self) -> Result<f32> {
        self.get_utilization()
    }

    /// Get GPU temperature in Celsius
    ///
    /// # Returns
    ///
    /// The GPU temperature, or an error if temperature monitoring is not available.
    pub fn temperature(&self) -> Result<f32> {
        self.get_temperature()
    }

    /// Get GPU power usage in watts
    ///
    /// # Returns
    ///
    /// The GPU power usage, or an error if power monitoring is not available.
    pub fn power_usage(&self) -> Result<f32> {
        self.get_power_usage()
    }

    // Internal implementation methods
    fn get_name(&self) -> Result<String> {
        // Early validation
        if self.device.is_null() {
            return Err(Error::not_available("No Metal device available"));
        }

        // Use objc_safe_exec to catch any exceptions from Metal
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
        // Early validation
        if self.device.is_null() {
            return Err(Error::not_available("No Metal device available"));
        }

        // Get individual metrics with proper error handling
        let name = self.get_name()?;
        let memory = self.get_memory_info()?;
        let utilization = self.get_utilization()?;

        // Optional metrics with fallback
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
        // Get GPU memory info using IOKit
        let matching = self.iokit.io_service_matching("IOAccelerator");
        let service = self.iokit.io_service_get_matching_service(&matching);

        let Some(service) = service else {
            return Err(Error::not_available("Could not find GPU service"));
        };

        let _properties = self
            .iokit
            .io_registry_entry_create_cf_properties(&service)?;

        // For now return placeholder values until we implement the full memory info collection
        // TODO: Parse the properties dictionary to get actual memory values
        Ok(GPUMemoryInfo {
            total: 0,
            used: 0,
            free: 0,
        })
    }

    fn get_utilization(&self) -> Result<f32> {
        // Implementation using IOKit to get GPU utilization
        // This is a placeholder until actual implementation
        Ok(0.0)
    }

    fn get_temperature(&self) -> Result<f32> {
        // Implementation using IOKit to get temperature
        Err(Error::not_implemented("GPU temperature monitoring"))
    }

    fn get_power_usage(&self) -> Result<f32> {
        // Implementation using IOKit to get power usage
        Err(Error::not_implemented("GPU power usage monitoring"))
    }
}

// Implement resource cleanup for the GPU struct
impl Drop for GPU {
    fn drop(&mut self) {
        if !self.device.is_null() {
            // Use autorelease_pool to safely handle cleanup
            autorelease_pool(|| {
                // Clear the device pointer to prevent double-free
                // Metal devices are reference counted by the system
                self.device = std::ptr::null_mut();
            });
        }
    }
}

// Safe Send and Sync implementations
// These are safe because:
// 1. Metal devices are thread-safe
// 2. We only access the device through ObjC message sends which preserve thread safety
// 3. IOKit is already Send + Sync
unsafe impl Send for GPU {}
unsafe impl Sync for GPU {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::iokit::MockIOKit;
    use objc2::rc::{autoreleasepool, Retained};
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::{NSDictionary, NSObject, NSString};
    use std::ptr;

    // Test that GPU name returns an error when device is null
    #[test]
    fn test_gpu_name_null_device() {
        // Create a mock IOKit
        let mock_iokit = MockIOKit::new();

        // Create GPU with null device to simulate no GPU
        let gpu = GPU {
            device: ptr::null_mut(),
            iokit: Box::new(mock_iokit),
        };

        // Should return appropriate error for null device
        let result = gpu.name();
        assert!(result.is_err());
        if let Err(Error::NotAvailable(_)) = result {
            // Expected error
        } else {
            panic!(
                "Expected NotAvailable error for null device, got: {:?}",
                result
            );
        }
    }

    // Create an empty dictionary for safe testing
    fn create_empty_dictionary() -> Retained<NSDictionary<NSString, NSObject>> {
        autoreleasepool(|_| {
            unsafe {
                let dict_class = class!(NSDictionary);
                let dict_ptr: *mut AnyObject = msg_send![dict_class, dictionary];

                // Ensure we got a valid dictionary
                assert!(!dict_ptr.is_null(), "Failed to create empty dictionary");

                // Convert to retained dictionary
                match Retained::from_raw(dict_ptr.cast()) {
                    Some(dict) => dict,
                    None => panic!("Could not retain dictionary"),
                }
            }
        })
    }

    // Helper function to create a test dictionary
    fn create_test_dictionary() -> Retained<NSDictionary<NSString, NSObject>> {
        autoreleasepool(|_| unsafe {
            let dict_class = class!(NSDictionary);
            let dict_ptr: *mut AnyObject = msg_send![dict_class, dictionary];
            Retained::from_raw(dict_ptr.cast()).expect("Failed to create test dictionary")
        })
    }

    // Create a test AnyObject safely
    fn create_test_anyobject() -> Retained<AnyObject> {
        autoreleasepool(|_| unsafe {
            let obj: *mut AnyObject = msg_send![class!(NSObject), new];
            Retained::from_raw(obj).expect("Failed to create test object")
        })
    }

    // Test that GPU metrics returns an error when device is null
    #[test]
    fn test_gpu_metrics_null_device() {
        // Create a mock IOKit
        let mock_iokit = MockIOKit::new();

        // Create GPU with null device
        let gpu = GPU {
            device: ptr::null_mut(),
            iokit: Box::new(mock_iokit),
        };

        // Should return appropriate error
        let result = gpu.metrics();
        assert!(result.is_err());
    }

    #[test]
    fn test_gpu_memory_info() -> Result<()> {
        // Skip test if no Metal device is available
        if unsafe { MTLCreateSystemDefaultDevice() }.is_null() {
            return Ok(());
        }

        let mut mock_iokit = MockIOKit::new();

        // Set expectations for the mock
        mock_iokit
            .expect_io_service_matching()
            .returning(|_| create_test_dictionary());

        mock_iokit
            .expect_io_service_get_matching_service()
            .returning(|_| Some(create_test_anyobject()));

        mock_iokit
            .expect_io_registry_entry_create_cf_properties()
            .returning(|_| Ok(create_test_dictionary()));

        let device = unsafe { MTLCreateSystemDefaultDevice() };
        if device.is_null() {
            return Ok(());
        }

        let gpu = GPU {
            device,
            iokit: Box::new(mock_iokit),
        };

        let _memory = gpu.memory_info()?; // Prefixed with _ to indicate intentional unused variable
                                          // Just verify we get some kind of result without error
                                          // The total value should be populated based on actual system info

        Ok(())
    }
}
