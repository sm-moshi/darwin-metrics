#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
