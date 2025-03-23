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
/// Type alias for GPU monitor trait re-exports
// Re-export types from the monitors module
pub use monitors::{GpuCharacteristicsMonitor, GpuMemoryMonitor, GpuTemperatureMonitor, GpuUtilizationMonitor};
// Re-export types from the types module
pub use types::{GpuCharacteristics, GpuInfo, GpuMemory, GpuMetrics, GpuState, GpuUtilization};

/// GPU constants
pub mod constants;
