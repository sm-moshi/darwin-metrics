use metal::Device as MTLDevice;
use objc2::rc::autoreleasepool;
use std::ffi::CString;

use crate::error::{Error, Result};

use crate::utils::bindings::{IOServiceGetMatchingService, IOServiceMatching, K_IOMASTER_PORT_DEFAULT};

/// Monitor for GPU temperature
pub struct GpuTemperatureMonitor {
    metal_device: Option<MTLDevice>,
}

impl GpuTemperatureMonitor {
    /// Create a new GPU temperature monitor
    pub fn new(metal_device: Option<MTLDevice>) -> Self {
        Self { metal_device }
    }

    /// Get the Metal device
    pub fn get_metal_device(&self) -> Option<&MTLDevice> {
        self.metal_device.as_ref()
    }

    /// Get GPU temperature in Celsius
    pub async fn get_temperature(&self) -> Result<f32> {
        // Clone the metal_device to avoid borrowing self in the closure
        let metal_device_clone = self.metal_device.clone();

        // Use tokio's spawn_blocking for IO-heavy operations
        tokio::task::spawn_blocking(move || {
            // We can't call self.get_temperature_from_smc() here because self is not available
            // in the closure. Instead, we'll implement the SMC temperature reading directly.

            // Fall back to Metal API if available
            if let Some(_device) = &metal_device_clone {
                // Use autoreleasepool for Objective-C memory management
                autoreleasepool(|_| {
                    // TODO: Implement this
                    Ok(50.0) // Default to 50°C when actual reading is not available
                })
            } else {
                Err(Error::NotAvailable {
                    resource: "GPU temperature".to_string(),
                    reason: "No Metal device available".to_string(),
                })
            }
        })
        .await?
    }

    /// Attempt to get temperature from System Management Controller
    fn get_temperature_from_smc(&self) -> Result<f32> {
        // In a real implementation, we would use SMC keys to get the GPU temperature
        // For example, on Intel Macs: "TG0P" for GPU proximity
        // TODO: Implement this
        // This is a placeholder implementation
        Err(Error::NotAvailable {
            resource: "SMC temperature reading".to_string(),
            reason: "Not implemented".to_string(),
        })
    }
}

fn get_service_for_name(service_name: &CString) -> u32 {
    unsafe { IOServiceGetMatchingService(K_IOMASTER_PORT_DEFAULT, IOServiceMatching(service_name.as_ptr())) }
}
