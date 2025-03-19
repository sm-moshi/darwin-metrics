use std::time::SystemTime;

use crate::core::types::{ByteSize, Percentage};

/// Holds information about GPU characteristics
#[derive(Debug, Clone, Default)]
pub struct GpuCharacteristics {
    /// Is this an integrated GPU (vs discrete)
    pub is_integrated: bool,
    /// Is this an Apple Silicon GPU
    pub is_apple_silicon: bool,
    /// Does this GPU have hardware raytracing support
    pub has_raytracing: bool,
    /// Core/execution unit count (if available)
    pub core_count: Option<u32>,
    /// Clock speed in MHz (if available)
    pub clock_speed_mhz: Option<u32>,
    /// GPU chip information
    pub chip_info: String,
    pub name: String,
    pub vendor: String,
    pub device_id: String,
    pub total_memory: u64,
    pub supports_metal: bool,
}

impl GpuCharacteristics {
    pub fn new(is_apple_silicon: bool, is_integrated: bool, chip_info: String) -> Self {
        Self {
            is_apple_silicon,
            is_integrated,
            chip_info,
            has_raytracing: false,
            core_count: None,
            clock_speed_mhz: None,
            name: String::new(),
            vendor: String::new(),
            device_id: String::new(),
            total_memory: 0,
            supports_metal: false,
        }
    }
}

/// Represents GPU memory information
#[derive(Debug, Clone)]
pub struct GpuMemory {
    /// Total memory in bytes
    pub total: u64,
    /// Used memory in bytes
    pub used: u64,
    /// Free memory in bytes
    pub free: u64,
    /// Timestamp when the measurement was taken
    pub timestamp: SystemTime,
}

impl Default for GpuMemory {
    fn default() -> Self {
        Self {
            total: 0,
            used: 0,
            free: 0,
            timestamp: SystemTime::now(),
        }
    }
}

impl GpuMemory {
    /// Creates a new GpuMemory instance with the specified total, used, and free memory values
    pub fn new(total: u64, used: u64, free: u64) -> Self {
        Self {
            total,
            used,
            free,
            timestamp: SystemTime::now(),
        }
    }
}

/// Represents GPU utilization metrics
#[derive(Debug, Clone)]
pub struct GpuUtilization {
    /// GPU core utilization percentage (0-100)
    pub value: f64,
}

impl Default for GpuUtilization {
    fn default() -> Self {
        Self { value: 0.0 }
    }
}

impl GpuUtilization {
    /// Creates a new GpuUtilization instance with the specified utilization value
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

/// GPU temperature information
#[derive(Debug, Clone)]
pub struct GpuTemperature {
    /// Temperature in Celsius
    pub temperature: f64,
    /// Timestamp when temperature was measured
    pub timestamp: SystemTime,
}

impl GpuTemperature {
    /// Creates a new GpuTemperature instance
    pub fn new(temperature: f64) -> Self {
        Self {
            temperature,
            timestamp: SystemTime::now(),
        }
    }
}

// Implement From<GpuTemperature> for f64 to allow explicit conversion
impl From<GpuTemperature> for f64 {
    fn from(temp: GpuTemperature) -> Self {
        temp.temperature
    }
}

// Implement From<&GpuTemperature> for f64 to allow conversion from references
impl From<&GpuTemperature> for f64 {
    fn from(temp: &GpuTemperature) -> Self {
        temp.temperature
    }
}

// Implement From<f32> for GpuTemperature to allow conversion from f32
impl From<f32> for GpuTemperature {
    fn from(temp: f32) -> Self {
        Self::new(temp as f64)
    }
}

/// GPU metrics
#[derive(Debug, Clone)]
pub struct GpuMetrics {
    /// GPU utilization percentage (0-100)
    pub utilization: Percentage,
    /// Used GPU memory in bytes
    pub memory_used: ByteSize,
    /// Total GPU memory in bytes
    pub memory_total: ByteSize,
    /// GPU temperature in Celsius
    pub temperature: f64,
    /// GPU power usage in watts
    pub power_usage: Option<f64>,
    /// Timestamp when metrics were collected
    pub timestamp: SystemTime,
}

/// GPU state information
#[derive(Debug, Clone)]
pub struct GpuState {
    /// GPU utilization metrics
    pub utilization: GpuUtilization,
    /// GPU memory metrics
    pub memory: GpuMemory,
    /// GPU temperature in Celsius
    pub temperature: f64,
    /// Timestamp when the state was captured
    pub timestamp: SystemTime,
}

impl GpuState {
    /// Create a new GPU state
    pub fn new(utilization: GpuUtilization, memory: GpuMemory, temperature: f64, timestamp: SystemTime) -> Self {
        Self {
            utilization,
            memory,
            temperature,
            timestamp,
        }
    }
}

impl Default for GpuState {
    fn default() -> Self {
        Self {
            utilization: GpuUtilization::default(),
            memory: GpuMemory::default(),
            temperature: 0.0,
            timestamp: SystemTime::now(),
        }
    }
}

/// Comprehensive GPU information including characteristics and current state
#[derive(Debug, Clone)]
pub struct GpuInfo {
    /// Static GPU characteristics
    pub characteristics: GpuCharacteristics,
    /// Current GPU state
    pub state: GpuState,
}

impl GpuInfo {
    /// Create a new GPU info object
    pub fn new(characteristics: GpuCharacteristics, state: GpuState) -> Self {
        Self { characteristics, state }
    }
}

impl Default for GpuInfo {
    fn default() -> Self {
        Self {
            characteristics: GpuCharacteristics::default(),
            state: GpuState::default(),
        }
    }
}
