use objc2::{msg_send, rc::autoreleasepool, runtime::AnyObject};

use crate::{
    error::Result,
    utils::bindings::{MTLCreateSystemDefaultDevice, MTLDeviceRef},
};

use super::GpuMetrics;

/// GPU monitoring functionality
///
/// This struct provides access to GPU information and metrics on macOS systems. It supports both discrete and
/// integrated GPUs, including Apple Silicon GPUs.
///
/// # Examples
///
/// ```no_run
/// use darwin_metrics::hardware::gpu::Gpu;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let gpu = Gpu::new()?;
///     let metrics = gpu.metrics()?;
///     
///     println!("GPU: {}", metrics.name);
///     println!("Utilization: {:.1}%", metrics.utilization);
///     println!("Memory used: {} bytes", metrics.memory.used);
///     
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Gpu {
    metal_device: Option<MTLDeviceRef>,
    // Note: Support for multiple GPUs will be added in a future version (post-1.0.0) This field would become a
    // Vec<MTLDeviceRef> or similar to track all available GPUs
}

impl Gpu {
    /// Creates a new GPU instance
    pub fn new() -> Result<Self> {
        // Create a Metal device for GPU info
        let metal_device = autoreleasepool(|_| unsafe {
            let device = MTLCreateSystemDefaultDevice();
            if device.is_null() {
                None
            } else {
                Some(device)
            }
        });

        Ok(Self { metal_device })
    }

    /// Gets the Metal device if available
    pub fn get_metal_device(&self) -> Option<MTLDeviceRef> {
        self.metal_device
    }

    /// Gets current GPU metrics including utilization, memory usage, and temperature
    pub fn metrics(&self) -> Result<GpuMetrics> {
        // Get all metrics in a structured way
        let name = self.name()?;
        let utilization = self.estimate_utilization()? as f64; // Convert f32 to f64
        let memory = self.estimate_memory_info()?;
        let temperature = self.get_temperature().ok();
        let characteristics = self.get_characteristics();

        Ok(GpuMetrics {
            utilization,
            memory,
            temperature,
            name,
            characteristics,
        })
    }
}

// We still need to manually implement Drop for memory safety
impl Drop for Gpu {
    fn drop(&mut self) {
        // Clean up Metal device if present
        if let Some(device) = self.metal_device {
            // Release the Metal device
            unsafe {
                let _: () = msg_send![device.cast::<AnyObject>(), release];
            }
        }
    }
}

unsafe impl Send for Gpu {}
unsafe impl Sync for Gpu {}
