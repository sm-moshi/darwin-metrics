//! # Core Types Module
//!
//! This module provides fundamental data types and structures used throughout the darwin-metrics library.
//! It includes implementations for common metric types, unit conversions, and data representations.
//!
//! ## Key Types
//!
//! * `ByteSize` - Represents data sizes with convenient conversion methods
//! * `Percentage` - Represents percentage values between 0.0 and 100.0
//! * `Temperature` - Represents temperature values with Celsius/Fahrenheit conversion
//! * `DiskIO` - Represents disk I/O metrics
//! * `DiskHealth` - Represents disk health information
//! * `DiskSpace` - Represents disk space information
//! * `Transfer` - Represents data transfer rates
//!
//! ## Example
//!
//! ```rust
//! use darwin_metrics::core::types::ByteSize;
//!
//! let size = ByteSize::new(1024);
//! assert_eq!(size.as_kb(), 1.0);
//! ```

use std::time::Duration;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents a percentage value between 0.0 and 100.0
///
/// # Examples
///
/// ```rust
/// use darwin_metrics::core::types::Percentage;
///
/// let p = Percentage::new(75.0).unwrap();
/// assert_eq!(p.as_f64(), 75.0);
///
/// // Values outside 0-100 range return None
/// assert!(Percentage::new(150.0).is_none());
/// ```
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

/// Represents a temperature in Celsius with conversion methods
///
/// # Examples
///
/// ```rust
/// use darwin_metrics::core::types::Temperature;
///
/// let temp = Temperature::new(25.0);
/// assert_eq!(temp.as_celsius(), 25.0);
/// assert_eq!(temp.as_fahrenheit(), 77.0);
/// ```
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

/// Represents a size in bytes with convenient conversion methods
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct ByteSize(pub u64);

impl ByteSize {
    /// Creates a new ByteSize instance from the given number of bytes
    ///
    /// # Arguments
    ///
    /// * `bytes` - The size in bytes
    ///
    /// # Returns
    ///
    /// A new `ByteSize` instance representing the specified number of bytes
    pub fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Returns the size in bytes
    ///
    /// # Returns
    ///
    /// The size in bytes as a u64
    pub fn as_bytes(&self) -> u64 {
        self.0
    }

    /// Returns the size in kilobytes
    ///
    /// # Returns
    ///
    /// The size in kilobytes as a f64
    pub fn as_kb(&self) -> f64 {
        self.0 as f64 / 1024.0
    }

    /// Returns the size in megabytes
    ///
    /// # Returns
    ///
    /// The size in megabytes as a f64
    pub fn as_mb(&self) -> f64 {
        self.as_kb() / 1024.0
    }

    /// Returns the size in gigabytes
    ///
    /// # Returns
    ///
    /// The size in gigabytes as a f64
    pub fn as_gb(&self) -> f64 {
        self.as_mb() / 1024.0
    }

    /// Create a new ByteSize from bytes (alias for new)
    pub fn from_bytes(bytes: u64) -> Self {
        Self::new(bytes)
    }
}

// Implement From for ByteSize
impl From<ByteSize> for u64 {
    fn from(size: ByteSize) -> Self {
        size.as_bytes()
    }
}

// Implement operations for ByteSize
impl std::ops::Sub for ByteSize {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let result = self.0.saturating_sub(rhs.0);
        Self(result)
    }
}

// Implement Mul<f64> for Percentage
impl std::ops::Mul<f64> for Percentage {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        self.as_f64() * rhs
    }
}

// Implement additional conversion methods for ByteSize
impl ByteSize {
    /// Converts the ByteSize to a u64 value
    pub fn to_u64(&self) -> u64 {
        self.0
    }
}

/// Format options for displaying byte sizes
///
/// Used to control how byte sizes are formatted when displayed
#[derive(Debug, Clone, Copy)]
pub enum ByteSizeFormat {
    /// Show size in bytes
    Bytes,
    /// Show size in kilobytes
    Kilobytes,
    /// Show size in megabytes
    Megabytes,
    /// Show size in gigabytes
    Gigabytes,
    /// Automatically select the most appropriate unit
    Auto,
}

/// Format options for displaying percentage values
///
/// Used to control how percentage values are formatted when displayed
#[derive(Debug, Clone, Copy)]
pub enum PercentageFormat {
    /// Show as decimal (0.0-1.0)
    Decimal,
    /// Show as percentage with symbol (0-100%)
    WithSymbol,
    /// Show as percentage without symbol (0-100)
    WithoutSymbol,
}

/// Represents disk I/O metrics including read/write operations and timings
///
/// This struct provides a comprehensive view of disk I/O activity, including
/// the number of operations, bytes transferred, and time spent on I/O operations.
#[derive(Debug, Clone)]
pub struct DiskIO {
    /// Number of read operations performed
    pub reads: u64,
    /// Number of write operations performed
    pub writes: u64,
    /// Total number of bytes read
    pub read_bytes: ByteSize,
    /// Total number of bytes written
    pub write_bytes: ByteSize,
    /// Total time spent on read operations
    pub read_time: Duration,
    /// Total time spent on write operations
    pub write_time: Duration,
}

/// Represents disk health information including SMART status
///
/// This struct provides information about the health status of a disk,
/// including its SMART status and any detected issues.
#[derive(Debug, Clone)]
pub struct DiskHealth {
    /// Whether the disk's SMART status is OK
    pub smart_status: bool,
    /// List of detected disk health issues
    pub issues: Vec<String>,
}

/// Represents disk space information including total, used, and available space
///
/// This struct provides a snapshot of disk space usage at a point in time.
#[derive(Debug, Clone, Copy)]
pub struct DiskSpace {
    /// Total disk space capacity
    pub total: ByteSize,
    /// Currently used disk space
    pub used: ByteSize,
    /// Available disk space for use
    pub available: ByteSize,
}

/// Represents data transfer rates for read and write operations
///
/// This struct tracks the rate of data transfer for both read and write operations.
#[derive(Debug, Clone, Copy)]
pub struct Transfer {
    /// Data transfer rate for read operations
    pub read: ByteSize,
    /// Data transfer rate for write operations
    pub write: ByteSize,
}
