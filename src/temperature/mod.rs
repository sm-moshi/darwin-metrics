use crate::{Error, Result};

/// Represents temperature sensor information
#[derive(Debug, PartialEq, Clone)]
pub struct Temperature {
    /// Sensor name/location
    pub sensor: String,
    /// Temperature in Celsius
    pub celsius: f64,
    /// Temperature in Fahrenheit
    pub fahrenheit: f64,
}

impl Temperature {
    /// Get information from all temperature sensors
    ///
    /// # Returns
    /// Returns a `Result` containing a vector of temperature information or an error if the information
    /// cannot be retrieved.
    ///
    /// # Examples
    /// ```no_run
    /// use darwin_metrics::temperature::Temperature;
    ///
    /// let temps = Temperature::get_all().unwrap();
    /// for temp in temps {
    ///     println!("{}: {:.1}°C", 
    ///         temp.sensor,
    ///         temp.celsius
    ///     );
    /// }
    /// ```
    pub fn get_all() -> Result<Vec<Self>> {
        // TODO: Implement actual temperature info retrieval
        Err(Error::not_implemented("Temperature info retrieval not yet implemented"))
    }

    /// Create a new Temperature instance from Celsius
    pub fn from_celsius(sensor: impl Into<String>, celsius: f64) -> Self {
        let celsius = celsius;
        let fahrenheit = celsius * 9.0 / 5.0 + 32.0;
        Self {
            sensor: sensor.into(),
            celsius,
            fahrenheit,
        }
    }

    /// Create a new Temperature instance from Fahrenheit
    pub fn from_fahrenheit(sensor: impl Into<String>, fahrenheit: f64) -> Self {
        let fahrenheit = fahrenheit;
        let celsius = (fahrenheit - 32.0) * 5.0 / 9.0;
        Self {
            sensor: sensor.into(),
            celsius,
            fahrenheit,
        }
    }

    /// Returns true if temperature is above 80°C
    pub fn is_critical(&self) -> bool {
        self.celsius > 80.0
    }

    /// Get current temperature information
    pub fn get_info() -> Result<Self> {
        // TODO: Implement temperature info retrieval
        Err(Error::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_conversions() {
        let temp = Temperature::from_celsius("CPU", 40.0);
        assert_eq!(temp.fahrenheit, 104.0);

        let temp = Temperature::from_fahrenheit("GPU", 104.0);
        assert_eq!(temp.celsius, 40.0);
    }

    #[test]
    fn test_temperature_critical() {
        let temp = Temperature::from_celsius("CPU", 85.0);
        assert!(temp.is_critical());

        let temp = Temperature::from_celsius("CPU", 75.0);
        assert!(!temp.is_critical());
    }
} 