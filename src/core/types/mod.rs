#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::time::Duration;

/// Represents a percentage value between 0.0 and 100.0
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Percentage(pub f64);

impl Percentage {
    /// Creates a new Percentage from a value between 0 and 100
    /// Returns None if the value is outside the valid range
    pub fn new(value: f64) -> Option<Self> {
        if (0.0..=100.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the percentage value as a float
    pub fn as_f64(&self) -> f64 {
        self.0
    }

    /// Create a new percentage value, clamping it to the range 0.0-100.0
    pub fn from_f64(value: f64) -> Self {
        Self(value.clamp(0.0, 100.0))
    }
}

/// Represents a temperature in Celsius
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Temperature(pub f64);

impl Temperature {
    /// Creates a new Temperature from a value in Celsius
    pub fn new(celsius: f64) -> Self {
        Self(celsius)
    }

    /// Returns the temperature in Celsius
    pub fn as_celsius(&self) -> f64 {
        self.0
    }

    /// Returns the temperature in Fahrenheit
    pub fn as_fahrenheit(&self) -> f64 {
        (self.0 * 9.0 / 5.0) + 32.0
    }

    /// Create a new temperature in Celsius (alias for new)
    pub fn new_celsius(celsius: f64) -> Self {
        Self::new(celsius)
    }
}

impl Default for Temperature {
    fn default() -> Self {
        Self(0.0)
    }
}

/// Represents a size in bytes
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct ByteSize(pub u64);

impl ByteSize {
    pub fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> u64 {
        self.0
    }

    pub fn as_kb(&self) -> f64 {
        self.0 as f64 / 1024.0
    }

    pub fn as_mb(&self) -> f64 {
        self.as_kb() / 1024.0
    }

    pub fn as_gb(&self) -> f64 {
        self.as_mb() / 1024.0
    }

    /// Create a new ByteSize from bytes (alias for new)
    pub fn from_bytes(bytes: u64) -> Self {
        Self::new(bytes)
    }
}

/// Format options for byte sizes
pub enum ByteSizeFormat {
    /// Show bytes
    Bytes,
    /// Show kilobytes
    Kilobytes,
    /// Show megabytes
    Megabytes,
    /// Show gigabytes
    Gigabytes,
    /// Automatically select the best unit
    Auto,
}

/// Format options for percentages
pub enum PercentageFormat {
    /// Show as decimal (0.0-1.0)
    Decimal,
    /// Show as percentage with symbol (0-100%)
    WithSymbol,
    /// Show as percentage without symbol (0-100)
    WithoutSymbol,
}

/// Represents disk I/O metrics
#[derive(Debug, Clone)]
pub struct DiskIO {
    /// Number of read operations
    pub reads: u64,
    /// Number of write operations
    pub writes: u64,
    /// Total bytes read
    pub read_bytes: ByteSize,
    /// Total bytes written
    pub write_bytes: ByteSize,
    /// Total time spent on read operations
    pub read_time: Duration,
    /// Total time spent on write operations
    pub write_time: Duration,
}

/// Represents disk health information
#[derive(Debug, Clone)]
pub struct DiskHealth {
    /// Whether SMART status is OK
    pub smart_status: bool,
    /// List of detected issues
    pub issues: Vec<String>,
}

/// Represents disk space information
#[derive(Debug, Clone, Copy)]
pub struct DiskSpace {
    /// Total disk space
    pub total: ByteSize,
    /// Used disk space
    pub used: ByteSize,
    /// Available disk space
    pub available: ByteSize,
}

/// Represents data transfer rates
#[derive(Debug, Clone, Copy)]
pub struct Transfer {
    /// Read transfer rate
    pub read: ByteSize,
    /// Write transfer rate
    pub write: ByteSize,
}
