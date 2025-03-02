use crate::{Error, Result};

/// Represents GPU information and metrics
#[derive(Debug, Clone, PartialEq)]
pub struct GPU {
    /// Name of the GPU
    pub name: String,
    /// GPU utilization percentage (0-100)
    pub utilization: f64,
    /// GPU memory usage in bytes
    pub memory_used: u64,
    /// Total GPU memory in bytes
    pub memory_total: u64,
    /// GPU temperature in Celsius
    pub temperature: f64,
}

impl GPU {
    /// Creates a new GPU instance with the given values
    pub fn new(name: String, utilization: f64, memory_used: u64, memory_total: u64, temperature: f64) -> Self {
        Self {
            name,
            utilization,
            memory_used,
            memory_total,
            temperature,
        }
    }

    /// Get current GPU information
    ///
    /// # Returns
    /// Returns a `Result` containing GPU information or an error if the information
    /// cannot be retrieved.
    ///
    /// # Examples
    /// ```no_run
    /// use darwin_metrics::gpu::GPU;
    ///
    /// let gpu = GPU::get_info().unwrap();
    /// println!("GPU: {}, Usage: {:.1}%", 
    ///     gpu.name,
    ///     gpu.utilization
    /// );
    /// ```
    pub fn get_info() -> Result<Self> {
        // TODO: Implement actual GPU info retrieval
        Err(Error::not_implemented("GPU info retrieval not yet implemented"))
    }

    /// Returns GPU memory usage as a percentage (0-100)
    pub fn memory_usage_percentage(&self) -> f64 {
        (self.memory_used as f64 / self.memory_total as f64) * 100.0
    }

    /// Returns true if GPU temperature is above 80°C
    pub fn is_temperature_critical(&self) -> bool {
        self.temperature > 80.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_calculations() {
        let gpu = GPU {
            name: "Test GPU".to_string(),
            utilization: 75.0,
            memory_used: 4 * 1024 * 1024 * 1024, // 4GB
            memory_total: 8 * 1024 * 1024 * 1024, // 8GB
            temperature: 70.0,
        };

        assert_eq!(gpu.memory_usage_percentage(), 50.0);
        assert!(!gpu.is_temperature_critical());
    }
} 