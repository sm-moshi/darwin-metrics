use crate::{Error, Result};

/// Represents system memory information
#[derive(Debug, PartialEq, Clone)]
pub struct Memory {
    /// Total physical memory in bytes
    pub total: u64,
    /// Available memory in bytes
    pub available: u64,
    /// Used memory in bytes
    pub used: u64,
    /// Memory used by wired/kernel in bytes
    pub wired: u64,
    /// Memory pressure level (0-1)
    pub pressure: f64,
}

impl Memory {
    /// Creates a new Memory instance with the given values
    pub fn new(total: u64, available: u64, used: u64, wired: u64, pressure: f64) -> Self {
        Self {
            total,
            available,
            used,
            wired,
            pressure,
        }
    }

    /// Get current memory information
    ///
    /// # Returns
    /// Returns a `Result` containing memory information or an error if the information
    /// cannot be retrieved.
    ///
    /// # Examples
    /// ```no_run
    /// use darwin_metrics::memory::Memory;
    ///
    /// let memory = Memory::get_info().unwrap();
    /// println!("Memory usage: {:.1}%", memory.usage_percentage());
    /// ```
    pub fn get_info() -> Result<Self> {
        // TODO: Implement actual memory info retrieval
        Err(Error::not_implemented(
            "Memory info retrieval not yet implemented",
        ))
    }

    /// Returns memory usage as a percentage (0-100)
    pub fn usage_percentage(&self) -> f64 {
        (self.used as f64 / self.total as f64) * 100.0
    }

    /// Returns memory pressure as a percentage (0-100)
    pub fn pressure_percentage(&self) -> f64 {
        self.pressure * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_calculations() {
        let memory = Memory {
            total: 16 * 1024 * 1024 * 1024,    // 16GB
            available: 8 * 1024 * 1024 * 1024, // 8GB
            used: 8 * 1024 * 1024 * 1024,      // 8GB
            wired: 2 * 1024 * 1024 * 1024,     // 2GB
            pressure: 0.5,
        };

        assert_eq!(memory.usage_percentage(), 50.0);
        assert_eq!(memory.pressure_percentage(), 50.0);
    }
}
