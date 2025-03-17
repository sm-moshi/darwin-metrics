/// GPU implementation module
pub mod gpu_impl;

/// GPU monitoring modules
pub mod monitors;

/// GPU data types and structures
pub mod types;

/// GPU constants
pub mod constants;

// Re-export types
pub use types::{GpuCharacteristics, GpuMemory, GpuUtilization};

// Re-export implementation
pub use gpu_impl::Gpu;

// Re-export monitors
pub use monitors::{GpuCharacteristicsMonitor, GpuMemoryMonitor, GpuTemperatureMonitor, GpuUtilizationMonitor};

// Re-export core monitor traits
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
