use crate::core::types::{Percentage, Temperature};
use std::time::Instant;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents a frequency in MHz
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Frequency(pub f64);

impl Frequency {
    pub fn new(mhz: f64) -> Self {
        Self(mhz)
    }

    pub fn as_mhz(&self) -> f64 {
        self.0
    }

    pub fn as_ghz(&self) -> f64 {
        self.0 / 1000.0
    }
}

/// Represents CPU frequency information
#[derive(Debug, Clone)]
pub struct CpuFrequency {
    /// Current frequency in MHz
    pub current: f64,
    /// Minimum frequency in MHz
    pub min: f64,
    /// Maximum frequency in MHz
    pub max: f64,
}

/// Represents CPU utilization information
#[derive(Debug, Clone)]
pub struct CpuUtilization {
    /// User space utilization percentage
    pub user: f64,
    /// System space utilization percentage
    pub system: f64,
    /// Idle percentage
    pub idle: f64,
    /// Nice percentage (Unix-like systems)
    pub nice: f64,
    /// Timestamp of the measurement
    pub timestamp: Instant,
}

/// Represents CPU temperature information
#[derive(Debug, Clone)]
pub struct CpuTemperature {
    /// Temperature in Celsius
    pub celsius: f64,
    /// Critical temperature threshold in Celsius
    pub critical: Option<f64>,
    /// Timestamp of the measurement
    pub timestamp: Instant,
}

impl Default for CpuUtilization {
    fn default() -> Self {
        Self { user: 0.0, system: 0.0, idle: 100.0, nice: 0.0, timestamp: Instant::now() }
    }
}

impl Default for CpuFrequency {
    fn default() -> Self {
        Self { current: 0.0, min: 0.0, max: 0.0 }
    }
}

impl Default for CpuTemperature {
    fn default() -> Self {
        Self { celsius: 0.0, critical: None, timestamp: Instant::now() }
    }
}

/// CPU core information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CoreInfo {
    /// Core ID
    pub id: usize,
    /// Current frequency in MHz
    pub frequency: Frequency,
    /// Current utilization percentage
    pub utilization: Percentage,
    /// Core temperature in Celsius
    pub temperature: Temperature,
}

/// CPU power information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PowerInfo {
    /// Current power consumption in watts
    pub current_watts: f64,
    /// Maximum power consumption in watts
    pub max_watts: f64,
}

/// CPU model information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModelInfo {
    /// CPU model name
    pub name: String,
    /// Number of physical cores
    pub physical_cores: usize,
    /// Number of logical cores (including hyperthreading)
    pub logical_cores: usize,
    /// Base frequency in MHz
    pub base_frequency: Frequency,
    /// Maximum turbo frequency in MHz
    pub max_frequency: Frequency,
}

/// CPU temperature information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TemperatureInfo {
    /// Current CPU temperature in Celsius
    pub current: Temperature,
    /// Critical temperature threshold in Celsius
    pub critical: Temperature,
    /// Whether the CPU is in a critical temperature state
    pub is_critical: bool,
}

/// CPU utilization information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UtilizationInfo {
    /// Overall CPU utilization percentage
    pub total: Percentage,
    /// Per-core utilization percentages
    pub per_core: Vec<Percentage>,
    /// System load average (1, 5, 15 minutes)
    pub load_average: [f64; 3],
} 