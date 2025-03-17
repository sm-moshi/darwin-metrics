/// GPU implementation module
pub mod gpu_impl;

/// GPU monitoring module
pub mod monitors;

/// GPU data types and structures
pub mod types;

/// GPU constants
pub mod constants;

// Re-export types
pub use types::{GpuCharacteristics, GpuMemory, GpuUtilization};

// Re-export implementation
pub use gpu_impl::Gpu;

// Re-export monitors directly with a shorter path
pub use monitors::{GpuCharacteristicsMonitor, GpuMemoryMonitor, GpuTemperatureMonitor, GpuUtilizationMonitor};

// Re-export core monitor traits (deprecated)
#[deprecated(
    since = "0.2.0-alpha.1",
    note = "Hardware traits have been moved to the traits module. Use darwin_metrics::traits instead."
)]
pub use crate::core::metrics::hardware::{HardwareMonitor, TemperatureMonitor, UtilizationMonitor};

/// GPU performance metrics and characteristics
#[derive(Debug, Clone, Default)]
pub struct GpuMetrics {
    /// GPU utilization percentage (0-100)
    pub utilization: f64,
    /// Memory usage information
    pub memory: GpuMemory,
    /// GPU temperature in Celsius (if available)
    pub temperature: Option<f32>,
    /// GPU model name
    pub name: String,
    /// GPU characteristics and capabilities
    pub characteristics: GpuCharacteristics,
}

//! This module provides GPU metrics and monitoring for macOS systems.
//!
//! It includes functionality for gathering GPU information, monitoring
//! GPU temperature, and reporting utilization.

// Re-export GPU types
pub mod types;
pub use types::*;

// Re-export monitors
pub mod monitors;
pub use monitors::{GpuCharacteristicsMonitor, GpuMemoryMonitor, GpuTemperatureMonitor, GpuUtilizationMonitor};
