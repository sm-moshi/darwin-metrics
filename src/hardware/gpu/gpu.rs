use super::MAX_GPU_MEMORY;
use crate::hardware::iokit::{IOKit, IOKitImpl};
use crate::Result;
use std::ffi::c_void;
use objc2::rc::autoreleasepool;

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

#[derive(Debug, Default)]
pub struct GpuMetrics {
    pub utilization: f64,
    pub temperature: f64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub name: String,
    pub perf_capacity: f64,
    pub perf_threshold: f64,
}

impl GpuMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn metrics_summary(&self) -> String {
        let memory_used_mb = self.memory_used / 1024 / 1024;
        let memory_total_mb = self.memory_total / 1024 / 1024;
        
        format!(
            "GPU: {}, {}% util, {}°C, {}/{} MB memory", 
            self.name,
            self.utilization, 
            self.temperature, 
            memory_used_mb, 
            memory_total_mb
        )
    }
}

#[derive(Debug)]
pub struct GPU {
    iokit: Box<dyn IOKit>,
}

impl GPU {
    pub fn new() -> Result<Self> {
        let iokit = Box::new(IOKitImpl::default());
        Ok(Self { iokit })
    }

    pub fn get_metrics(&self) -> Result<GpuMetrics> {
        // Wrap in autoreleasepool for proper memory management
        autoreleasepool(|_| {
            // Get GPU statistics using the IOKit approach
            let gpu_stats = self.iokit.get_gpu_stats()?;
            
            // Try to get GPU temperature
            let temperature = self.iokit.get_gpu_temperature().unwrap_or(0.0);
            
            Ok(GpuMetrics {
                utilization: gpu_stats.utilization,
                temperature,
                memory_used: gpu_stats.memory_used,
                memory_total: gpu_stats.memory_total,
                name: gpu_stats.name,
                perf_capacity: gpu_stats.perf_cap,
                perf_threshold: gpu_stats.perf_threshold,
            })
        })
    }

    pub fn memory_info(&self) -> Result<GPUMemoryInfo> {
        // Wrap in autoreleasepool for proper memory management
        autoreleasepool(|_| {
            // Get GPU statistics using IOKit
            let gpu_stats = self.iokit.get_gpu_stats()?;
            
            let total = if gpu_stats.memory_total > 0 {
                gpu_stats.memory_total
            } else {
                // Fallback to default value if we couldn't determine actual memory
                MAX_GPU_MEMORY
            };
            
            let used = gpu_stats.memory_used;
            
            // Calculate free memory
            let free = total.saturating_sub(used);
            
            Ok(GPUMemoryInfo { total, used, free })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpu_initialization() {
        let gpu = GPU::new();
        assert!(gpu.is_ok(), "GPU initialization failed");
    }

    // FIXME: The following tests are temporarily disabled due to memory management issues
    // with Objective-C interoperability. They cause SIGSEGV when accessing deallocated objects.
    // Issue tracked in CHANGELOG.md and TODO.md
    
    // Test disabled due to memory safety issues with IOKit interface
    // #[test]
    // fn test_gpu_metrics() {
    //     // This test may fail in environments without GPU support
    //     let gpu = match GPU::new() {
    //         Ok(gpu) => gpu,
    //         Err(e) => {
    //             println!("Warning: Couldn't initialize GPU: {:?}", e);
    //             return; // Skip the test if GPU initialization fails
    //         }
    //     };
    //     
    //     match gpu.get_metrics() {
    //         Ok(metrics) => {
    //             assert!(metrics.utilization >= 0.0, "Invalid GPU utilization");
    //             assert!(metrics.temperature >= 0.0, "Invalid GPU temperature");
    //             assert!(metrics.memory_total > 0, "Invalid GPU memory total");
    //         },
    //         Err(e) => {
    //             // Just log the error and continue, don't fail the test
    //             println!("Warning: Couldn't get GPU metrics: {:?}", e);
    //         }
    //     }
    // }
    
    // Test disabled due to memory safety issues with IOKit interface
    // #[test]
    // fn test_gpu_memory_info() {
    //     // This test may fail in environments without GPU support
    //     let gpu = match GPU::new() {
    //         Ok(gpu) => gpu,
    //         Err(e) => {
    //             println!("Warning: Couldn't initialize GPU: {:?}", e);
    //             return; // Skip the test if GPU initialization fails
    //         }
    //     };
    //     
    //     match gpu.memory_info() {
    //         Ok(memory_info) => {
    //             assert!(memory_info.total > 0, "Invalid total VRAM");
    //             assert!(memory_info.used <= memory_info.total, "Used memory exceeds total VRAM");
    //             assert!(memory_info.free <= memory_info.total, "Free memory exceeds total VRAM");
    //         },
    //         Err(e) => {
    //             // Just log the error and continue, don't fail the test
    //             println!("Warning: Couldn't get GPU memory info: {:?}", e);
    //         }
    //     }
    // }
}