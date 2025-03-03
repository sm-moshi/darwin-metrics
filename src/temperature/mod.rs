//! Temperature sensor monitoring for macOS systems
//!
//! This module provides functionality to monitor and retrieve temperature readings
//! from various system sensors. It supports both Celsius and Fahrenheit readings
//! and provides automatic conversion between the two units.
//!
//! # Examples
//!
//! ```no_run
//! use darwin_metrics::temperature::Temperature;
//!
//! // Get temperature readings from all sensors
//! let sensors = Temperature::get_all().unwrap();
//! for sensor in sensors {
//!     println!("{}: {:.1}°C", sensor.sensor, sensor.celsius);
//! }
//! ```

use crate::{Error, Result};

/// Represents a temperature reading from a sensor
#[derive(Debug, Clone, PartialEq)]
pub struct Temperature {
    /// Name or identifier of the temperature sensor
    pub sensor: String,
    /// Temperature in Celsius
    pub celsius: f64,
    /// Temperature in Fahrenheit
    pub fahrenheit: f64,
}

impl Temperature {
    /// Create a new Temperature instance with the given sensor name and temperature in Celsius
    ///
    /// # Arguments
    /// * `sensor` - Name or identifier of the temperature sensor
    /// * `celsius` - Temperature in Celsius
    ///
    /// # Examples
    /// ```
    /// use darwin_metrics::temperature::Temperature;
    ///
    /// let temp = Temperature::from_celsius("CPU", 45.5);
    /// assert_eq!(temp.celsius, 45.5);
    /// assert_eq!(temp.fahrenheit, 113.9);
    /// ```
    pub fn from_celsius(sensor: impl Into<String>, celsius: f64) -> Self {
        Self {
            sensor: sensor.into(),
            celsius,
            fahrenheit: celsius * 9.0 / 5.0 + 32.0,
        }
    }

    /// Create a new Temperature instance with the given sensor name and temperature in Fahrenheit
    ///
    /// # Arguments
    /// * `sensor` - Name or identifier of the temperature sensor
    /// * `fahrenheit` - Temperature in Fahrenheit
    ///
    /// # Examples
    /// ```
    /// use darwin_metrics::temperature::Temperature;
    ///
    /// let temp = Temperature::from_fahrenheit("GPU", 113.9);
    /// assert_eq!(temp.celsius, 45.5);
    /// assert_eq!(temp.fahrenheit, 113.9);
    /// ```
    pub fn from_fahrenheit(sensor: impl Into<String>, fahrenheit: f64) -> Self {
        Self {
            sensor: sensor.into(),
            celsius: (fahrenheit - 32.0) * 5.0 / 9.0,
            fahrenheit,
        }
    }

    /// Get temperature readings from all available sensors
    ///
    /// # Returns
    /// A vector of Temperature instances, one for each sensor
    pub fn get_all() -> Result<Vec<Self>> {
        // TODO: Implement actual temperature sensor reading
        Err(Error::not_implemented(
            "Temperature sensor reading not yet implemented",
        ))
    }

    /// Returns true if temperature is above 80°C
    pub fn is_critical(&self) -> bool {
        self.celsius > 80.0
    }

    /// Get current temperature information
    pub fn get_info() -> Result<Self> {
        Err(Error::NotImplemented(
            "Temperature info not yet implemented".to_string(),
        ))
    }
}
