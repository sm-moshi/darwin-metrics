use crate::hardware::iokit::{IOKit, IOKitImpl};
use crate::Result;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SensorLocation {
    Cpu,
    Gpu,
    Memory,
    Storage,
    Battery,
    Other(String),
}

/// Hardware temperature implementation for macOS
/// Uses SMC (System Management Controller) to read temperature sensors
#[derive(Debug)]
pub struct Temperature {
    pub sensors: HashMap<String, f64>,
    io_kit: IOKitImpl,
}

impl Temperature {
    /// Create a new Temperature instance
    pub fn new() -> Self {
        Self {
            sensors: HashMap::new(),
            io_kit: IOKitImpl,
        }
    }

    /// Refresh all temperature readings
    pub fn refresh(&mut self) -> Result<()> {
        // Get CPU temperature
        if let Ok(temp) = self.io_kit.get_cpu_temperature() {
            self.sensors.insert("CPU".to_string(), temp);
        }

        // Get GPU temperature if available
        if let Ok(temp) = self.io_kit.get_gpu_temperature() {
            self.sensors.insert("GPU".to_string(), temp);
        }

        Ok(())
    }

    /// Get CPU temperature
    pub fn cpu_temperature(&mut self) -> Result<f64> {
        self.refresh()?;
        self.sensors.get("CPU").cloned().ok_or_else(|| {
            crate::Error::Temperature("CPU temperature sensor not available".to_string())
        })
    }

    /// Get GPU temperature (if available)
    pub fn gpu_temperature(&mut self) -> Result<f64> {
        self.refresh()?;
        self.sensors.get("GPU").cloned().ok_or_else(|| {
            crate::Error::Temperature("GPU temperature sensor not available".to_string())
        })
    }

    /// Get a list of all available temperature sensors
    pub fn list_sensors(&mut self) -> Result<Vec<(String, SensorLocation)>> {
        self.refresh()?;

        let mut result = Vec::new();
        for name in self.sensors.keys() {
            let location = if name == "CPU" {
                SensorLocation::Cpu
            } else if name == "GPU" {
                SensorLocation::Gpu
            } else {
                SensorLocation::Other(name.to_string())
            };

            result.push((name.clone(), location));
        }

        Ok(result)
    }

    /// Get temperature for a specific sensor by name
    pub fn get_sensor_temperature(&mut self, name: &str) -> Result<f64> {
        self.refresh()?;
        self.sensors
            .get(name)
            .cloned()
            .ok_or_else(|| crate::Error::Temperature(format!("Sensor {} not found", name)))
    }

    /// Determine if the system is experiencing thermal throttling
    pub fn is_throttling(&mut self) -> Result<bool> {
        // A simple heuristic - in a real implementation, we would use proper
        // macOS APIs to detect actual throttling
        let cpu_temp = self.cpu_temperature()?;
        Ok(cpu_temp > 80.0)
    }
}

impl Default for Temperature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_new() {
        let temp = Temperature::new();
        assert!(temp.sensors.is_empty());
    }

    #[test]
    fn test_cpu_temperature() {
        // Instead of mocking, we'll manually insert the test values
        let mut temp = Temperature::new();

        // Insert the test value directly into the sensors map
        temp.sensors.insert("CPU".to_string(), 42.5);

        // Test the method using the manually inserted value
        let result = temp.cpu_temperature().unwrap();
        assert_eq!(result, 42.5);
    }

    #[test]
    fn test_gpu_temperature() {
        // Create a Temperature instance with pre-populated data
        let mut temp = Temperature::new();
        temp.sensors.insert("GPU".to_string(), 55.0);

        // Test the method
        let result = temp.gpu_temperature().unwrap();
        assert_eq!(result, 55.0);
    }

    #[test]
    fn test_list_sensors() {
        // Create a Temperature instance with pre-populated data
        let mut temp = Temperature::new();
        temp.sensors.insert("CPU".to_string(), 42.5);
        temp.sensors.insert("GPU".to_string(), 55.0);

        // Test the method
        let sensors = temp.list_sensors().unwrap();

        // Verify results
        assert_eq!(sensors.len(), 2);

        // Check that we got both CPU and GPU sensors
        let has_cpu = sensors
            .iter()
            .any(|(name, location)| name == "CPU" && matches!(location, SensorLocation::Cpu));
        let has_gpu = sensors
            .iter()
            .any(|(name, location)| name == "GPU" && matches!(location, SensorLocation::Gpu));

        assert!(has_cpu);
        assert!(has_gpu);
    }

    #[test]
    fn test_is_throttling() {
        // Test case 1: CPU temperature below threshold
        let mut temp = Temperature::new();
        temp.sensors.insert("CPU".to_string(), 75.0);

        let result = temp.is_throttling().unwrap();
        assert_eq!(result, false);

        // Test case 2: CPU temperature above threshold
        let mut temp = Temperature::new();
        temp.sensors.insert("CPU".to_string(), 85.0);

        let result = temp.is_throttling().unwrap();
        assert_eq!(result, true);
    }
}
