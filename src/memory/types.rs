use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Memory pressure level indicator
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub enum PressureLevel {
    /// Normal memory pressure - sufficient memory available
    Normal,
    /// Warning level memory pressure - memory is becoming constrained
    Warning,
    /// Critical memory pressure - system is under severe memory constraints
    Critical,
}

impl fmt::Display for PressureLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal => write!(f, "Normal"),
            Self::Warning => write!(f, "Warning"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Detailed memory page states
#[derive(Debug, PartialEq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PageStates {
    /// Memory pages actively in use
    pub active: u64,
    /// Memory pages that haven't been accessed recently but still in RAM
    pub inactive: u64,
    /// Memory pages that cannot be paged out (kernel and other critical components)
    pub wired: u64,
    /// Memory pages immediately available for allocation
    pub free: u64,
    /// Memory pages that have been compressed to save physical RAM
    pub compressed: u64,
}

/// Swap file usage and activity metrics
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SwapUsage {
    /// Total swap space in bytes
    pub total: u64,
    /// Used swap space in bytes
    pub used: u64,
    /// Available swap space in bytes
    pub free: u64,
    /// Rate of data being swapped in (pages/sec)
    pub ins: f64,
    /// Rate of data being swapped out (pages/sec)
    pub outs: f64,
    /// Swap utilization as a percentage (0.0-1.0)
    pub pressure: f64,
}

impl Default for SwapUsage {
    fn default() -> Self {
        Self {
            total: 0,
            used: 0,
            free: 0,
            ins: 0.0,
            outs: 0.0,
            pressure: 0.0,
        }
    }
}

/// Memory metrics information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MemoryInfo {
    /// Total physical memory in bytes
    pub total: u64,
    /// Free memory in bytes
    pub free: u64,
    /// Used memory in bytes
    pub used: u64,
    /// Active memory in bytes
    pub active: u64,
    /// Inactive memory in bytes
    pub inactive: u64,
    /// Wired memory in bytes
    pub wired: u64,
    /// Compressed memory in bytes
    pub compressed: u64,
    /// Memory pressure (0.0-1.0)
    pub pressure: f64,
    /// System page size in bytes
    pub page_size: u64,
    /// Detailed page states
    pub page_states: PageStates,
    /// Swap usage information
    pub swap_usage: SwapUsage,
}

impl Default for MemoryInfo {
    fn default() -> Self {
        Self {
            total: 0,
            free: 0,
            used: 0,
            active: 0,
            inactive: 0,
            wired: 0,
            compressed: 0,
            pressure: 0.0,
            page_size: 0,
            page_states: PageStates::default(),
            swap_usage: SwapUsage::default(),
        }
    }
}

/// Type definition for memory pressure callback functions
pub type PressureCallback = Box<dyn Fn(PressureLevel) + Send + Sync>;
