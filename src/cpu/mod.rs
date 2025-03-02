use crate::{Error, Result};

/// Represents CPU information and metrics
#[derive(Debug, PartialEq)]
pub struct CPU {
    /// Number of physical CPU cores
    physical_cores: u32,
    /// Number of logical CPU cores (including virtual/hyperthreaded cores)
    logical_cores: u32,
    /// Current CPU frequency in MHz
    frequency_mhz: f64,
    /// Usage percentage per core (0.0 to 100.0)
    core_usage: Vec<f64>,
}

impl CPU {
    /// Creates a new CPU instance with the given specifications
    ///
    /// # Arguments
    /// * `physical_cores` - Number of physical CPU cores
    /// * `logical_cores` - Number of logical CPU cores
    /// * `core_usage` - Vector of core usage percentages (0.0 to 100.0)
    /// * `frequency_mhz` - Current CPU frequency in MHz
    ///
    /// # Returns
    /// A new CPU instance
    pub fn new(physical_cores: u32, logical_cores: u32, core_usage: Vec<f64>, frequency_mhz: f64) -> Self {
        // Validate core usage values
        let validated_usage = core_usage.into_iter()
            .map(|usage| usage.clamp(0.0, 100.0))
            .collect();

        CPU {
            physical_cores,
            logical_cores,
            frequency_mhz: frequency_mhz.max(0.0),
            core_usage: validated_usage,
        }
    }

    /// Retrieves current CPU information from the system
    ///
    /// # Returns
    /// Result containing CPU information or an error if retrieval fails
    pub fn get_info() -> Result<Self> {
        // TODO: Implement actual system call to get CPU info
        Err(Error::not_implemented("CPU info retrieval not yet implemented"))
    }

    /// Calculates the average CPU usage across all cores
    ///
    /// # Returns
    /// Average CPU usage as a percentage (0.0 to 100.0)
    pub fn average_usage(&self) -> f64 {
        if self.core_usage.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.core_usage.iter().sum();
        (sum / self.core_usage.len() as f64).clamp(0.0, 100.0)
    }

    /// Gets the number of physical cores
    pub fn physical_cores(&self) -> u32 {
        self.physical_cores
    }

    /// Gets the number of logical cores
    pub fn logical_cores(&self) -> u32 {
        self.logical_cores
    }

    /// Gets the current CPU frequency in MHz
    pub fn frequency_mhz(&self) -> f64 {
        self.frequency_mhz
    }

    /// Gets a slice of core usage percentages
    pub fn core_usage(&self) -> &[f64] {
        &self.core_usage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cpu() {
        let cpu = CPU::new(4, 8, vec![50.0, 75.0, 25.0, 100.0], 2400.0);
        assert_eq!(cpu.physical_cores(), 4);
        assert_eq!(cpu.logical_cores(), 8);
        assert_eq!(cpu.frequency_mhz(), 2400.0);
    }

    #[test]
    fn test_average_usage() {
        let cpu = CPU::new(2, 4, vec![50.0, 75.0, 25.0, 100.0], 2400.0);
        assert_eq!(cpu.average_usage(), 62.5);
    }

    #[test]
    fn test_value_clamping() {
        let cpu = CPU::new(
            2, 
            4, 
            vec![-10.0, 50.0, 150.0, 75.0], 
            -100.0
        );
        assert_eq!(cpu.core_usage(), &[0.0, 50.0, 100.0, 75.0]);
        assert_eq!(cpu.frequency_mhz(), 0.0);
    }

    #[test]
    fn test_empty_usage() {
        let cpu = CPU::new(2, 4, vec![], 2400.0);
        assert_eq!(cpu.average_usage(), 0.0);
    }
} 