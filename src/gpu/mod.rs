/// Types for GPU metrics
pub mod types;

/// Implementation of the Gpu struct
mod gpu_impl;

/// GPU monitoring functionality
pub mod monitors;

/// GPU monitoring and metrics module
///
/// This module provides GPU metrics and monitoring for macOS systems.
///
/// It includes functionality for gathering GPU information, monitoring
/// GPU temperature, and reporting utilization.
pub use gpu_impl::*;

/// GPU constants
pub mod constants;

// NOTE: Hardware monitoring traits like HardwareMonitor, TemperatureMonitor, and UtilizationMonitor
// should be imported directly from darwin_metrics::traits module

use std::ffi::CString;
use std::sync::Arc;

use crate::hardware::iokit::IOKit;
use crate::utils::bindings::{IOServiceGetMatchingService, IOServiceMatching, K_IOMASTER_PORT_DEFAULT};

/// GPU information and monitoring
#[derive(Debug)]
pub struct Gpu {
    /// IOKit service for accessing hardware information
    io_kit: Arc<Box<dyn IOKit>>,
}

impl Gpu {
    /// Create a new GPU instance
    pub fn new(io_kit: Arc<Box<dyn IOKit>>) -> Self {
        Self { io_kit }
    }

    /// Get GPU service connection
    pub(crate) fn get_gpu_service() -> Option<u32> {
        // Create service name for GPU
        let service_name = CString::new("IOGraphicsAccelerator2").ok()?;

        // Get matching service
        unsafe {
            let service =
                IOServiceGetMatchingService(K_IOMASTER_PORT_DEFAULT, IOServiceMatching(service_name.as_ptr()));

            if service == 0 { None } else { Some(service) }
        }
    }
}

// Re-export types
pub use monitors::{GpuCharacteristicsMonitor, GpuMemoryMonitor, GpuTemperatureMonitor, GpuUtilizationMonitor};
pub use types::{GpuCharacteristics, GpuInfo, GpuMemory, GpuMetrics, GpuState, GpuUtilization};
