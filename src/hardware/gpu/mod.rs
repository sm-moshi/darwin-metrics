// Re-export GPU module components
mod characteristics;
mod gpu_impl;
mod memory;
mod temperature;
mod utilization;

// Re-export the main types and structs
pub use characteristics::GpuCharacteristics;
pub use gpu_impl::Gpu;
pub use memory::GpuMemoryInfo;

/// GPU performance metrics and characteristics
#[derive(Debug, Clone, Default)]
pub struct GpuMetrics {
    /// GPU utilization percentage (0-100)
    pub utilization: f64,
    /// Memory usage information
    pub memory: GpuMemoryInfo,
    /// GPU temperature in Celsius (if available)
    pub temperature: Option<f32>,
    /// GPU model name
    pub name: String,
    /// GPU characteristics and capabilities
    pub characteristics: GpuCharacteristics,
}

#[cfg(test)]
mod tests;
