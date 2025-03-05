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

use crate::iokit::{IOKit, IOKitImpl};
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
    /// Create a new GPU metrics collector
    ///
    /// # Returns
    ///
    /// A new GPU instance if a Metal-capable GPU is available, or an error otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use darwin_metrics::gpu::GPU;
    ///
    /// let gpu = GPU::new().expect("Failed to initialize GPU metrics");
    /// ```
    pub fn new() -> Result<Self> {
        let device = unsafe { MTLCreateSystemDefaultDevice() };
        if device.is_null() {
            return Err(Error::not_available("No Metal-capable GPU found"));
        }

        let iokit = Box::<IOKitImpl>::default();

        Ok(Self { device, iokit })
    }

    /// Get the GPU name/model
    ///
    /// # Returns
    ///
    /// The name of the GPU, or an error if it cannot be determined.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use darwin_metrics::gpu::GPU;
    ///
    /// let gpu = GPU::new().expect("Failed to initialize GPU metrics");
    /// println!("GPU: {}", gpu.name().unwrap_or_else(|_| "Unknown".to_string()));
    /// ```
    pub fn name(&self) -> Result<String> {
        self.get_name()
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
        objc_safe_exec(|| {
            unsafe {
                let device = self.device as *const AnyObject;
                
                if device.is_null() {
                    return Err(Error::not_available("Invalid Metal device"));
                }
                
                let device_ref = device.as_ref()
                    .ok_or_else(|| Error::not_available("Invalid Metal device reference"))?;

                let name: *mut AnyObject = msg_send![device_ref, name];
                if name.is_null() {
                    return Err(Error::not_available("Could not get GPU name"));
                }

                let ns_string = (name as *const NSString).as_ref()
                    .ok_or_else(|| Error::not_available("Invalid name string"))?;
                    
                Ok(ns_string.to_string())
            }
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

        let _properties = self.iokit.io_registry_entry_create_cf_properties(&service)?;

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
    use objc2::{class, msg_send};
    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, NSObject};
    use objc2_foundation::{NSDictionary, NSString};

    // Wrap setup_test in an autorelease pool
    fn setup_test() {
        autorelease_pool(|| {
            // No need for explicit msg_send - this is handled by autorelease_pool
        });
    }

    // Create a safe dictionary for testing
    fn create_safe_dictionary() -> Retained<NSDictionary<NSString, NSObject>> {
        unsafe {
            let dict: *mut AnyObject = msg_send![class!(NSDictionary), new];
            Retained::from_raw(dict.cast()).unwrap_or_else(|| {
                panic!("Failed to create test dictionary");
            })
        }
    }

    // Create a safe test object
    fn create_test_object() -> Retained<NSObject> {
        unsafe {
            let obj: *mut AnyObject = msg_send![class!(NSObject), new];
            Retained::from_raw(obj.cast()).unwrap_or_else(|| {
                panic!("Failed to create test object");
            })
        }
    }

    // Helper function to create a safe test device
    fn create_test_device() -> MTLDeviceRef {
        unsafe { MTLCreateSystemDefaultDevice() }
    }

    // Add this helper function
    fn with_safe_gpu<F>(f: F) -> Result<()> 
    where F: FnOnce(&GPU) -> Result<()> {
        let device = create_test_device();
        if device.is_null() {
            return Ok(()); // Skip test if no GPU
        }
        
        let mock_iokit = crate::iokit::MockIOKit::new();
        let mut gpu = GPU {
            device,
            iokit: Box::new(mock_iokit),
        };
        
        // Run the test function
        let result = f(&gpu);
        
        // Explicitly clear the device before dropping
        gpu.device = std::ptr::null_mut();
        
        result
    }
    
    // Use it in tests
    #[test]
    fn test_a_gpu_name_null_device() {
        // Ensure this runs first with alphabetical ordering
        setup_test();
        let mock_iokit = crate::iokit::MockIOKit::new();
        let gpu = GPU {
            device: std::ptr::null_mut(),
            iokit: Box::new(mock_iokit),
        };
        
        let result = gpu.name();
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, Error::NotAvailable(_)));
        }
    }

    #[test]
    fn test_gpu_creation() -> Result<()> {
        setup_test();
        // Skip test if no Metal device is available
        let device = create_test_device();
        if device.is_null() {
            return Ok(());
        }
        
        let gpu = GPU::new()?;
        assert!(!gpu.device.is_null());
        Ok(())
    }

    #[test]
    fn test_metrics_collection() -> Result<()> {
        setup_test();
        
        // Run test inside autorelease pool
        autorelease_pool(|| {
            let mut mock_iokit = crate::iokit::MockIOKit::new();
            
            mock_iokit.expect_io_service_matching()
                .returning(|_| create_safe_dictionary());
    
            mock_iokit.expect_io_service_get_matching_service()
                .returning(|_| Some(create_test_anyobject()));
    
            mock_iokit.expect_io_registry_entry_create_cf_properties()
                .returning(|_| Ok(create_safe_dictionary()));
    
            // Create GPU with test device - skip if no device
            let device = create_test_device();
            if device.is_null() {
                return Ok(());
            }
    
            let gpu = GPU {
                device,
                iokit: Box::new(mock_iokit),
            };
            
            // Get metrics safely
            let result = gpu.metrics();
            
            // Now check result
            if let Ok(metrics) = &result {
                assert!(!metrics.name.is_empty());
            }
            
            Ok(())
        })
    }

    #[test]
    fn test_gpu_metrics_failure_modes() {
        setup_test();
        // Test no Metal device available
        let mock_iokit = crate::iokit::MockIOKit::new();
        let gpu = GPU {
            device: std::ptr::null_mut(),
            iokit: Box::new(mock_iokit),
        };
        
        assert!(matches!(gpu.metrics(), Err(Error::NotAvailable(_))));

        // Test GPU without service
        let mut mock_iokit = crate::iokit::MockIOKit::new();
        mock_iokit.expect_io_service_matching()
            .returning(|_| create_safe_dictionary());
            
        mock_iokit.expect_io_service_get_matching_service()
            .returning(|_| Some(create_test_anyobject()));

        let device = create_test_device();
        if device.is_null() {
            return; // Skip test if no GPU available
        }

        let gpu = GPU {
            device,
            iokit: Box::new(mock_iokit),
        };
        
        assert!(matches!(gpu.memory_info(), Err(Error::NotAvailable(_))));
    }

    #[test]
    fn test_unimplemented_methods() {
        setup_test();
        let device = create_test_device();
        if device.is_null() {
            return; // Skip test if no GPU available
        }

        let gpu = GPU {
            device,
            iokit: Box::<IOKitImpl>::default(),
        };
        
        assert!(matches!(gpu.temperature(), Err(Error::NotImplemented(_))));
        assert!(matches!(gpu.power_usage(), Err(Error::NotImplemented(_))));
        assert!(matches!(gpu.utilization(), Ok(_)));
    }

    #[test]
    fn test_gpu_memory_info() -> Result<()> {
        // Wrap in autorelease pool
        autorelease_pool(|| {
            setup_test();
            let mut mock_iokit = crate::iokit::MockIOKit::new();
            
            // Simplify test expectations
            mock_iokit.expect_io_service_matching()
                .returning(|_| create_safe_dictionary());
    
            mock_iokit.expect_io_service_get_matching_service()
                .returning(|_| Some(create_test_anyobject()));
    
            mock_iokit.expect_io_registry_entry_create_cf_properties()
                .returning(|_| Ok(create_safe_dictionary()));
    
            let device = create_test_device();
            if device.is_null() {
                return Ok(()); 
            }
    
            let gpu = GPU {
                device,
                iokit: Box::new(mock_iokit),
            };
    
            let memory = gpu.memory_info()?;
            assert_eq!(memory.total, 0);
            assert_eq!(memory.used, 0);
            assert_eq!(memory.free, 0);
            
            Ok(())
        })
    }

    // Add a final cleanup test
    #[test]
    fn test_z_final_cleanup() {
        // Add this test to run last (alphabetical order)
        autorelease_pool(|| {
            // This ensures any remaining autoreleased objects are drained
        });
    }

    // Helper function to create a test object with the correct type
    fn create_test_anyobject() -> Retained<AnyObject> {
        unsafe { 
            let obj: *mut AnyObject = msg_send![class!(NSObject), new];
            Retained::from_raw(obj).unwrap()
        }
    }
}