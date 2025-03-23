use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::memory::{Memory, MemoryMonitor};

lazy_static! {
    /// Global mutex for memory tests to prevent concurrent access to system memory APIs
    pub static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

pub mod monitors;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let _lock = TEST_MUTEX.lock();
        let memory = Memory::new();
        assert!(memory.is_ok(), "Should be able to initialize Memory");
    }

    #[test]
    fn test_memory_monitors() {
        let _lock = TEST_MUTEX.lock();
        let memory = Memory::new().unwrap();

        // Test monitor creation
        let usage_monitor = memory.usage_monitor();
        let pressure_monitor = memory.pressure_monitor();
        let swap_monitor = memory.swap_monitor();

        // Verify monitor names
        assert!(usage_monitor.name().is_ok());
        assert!(pressure_monitor.name().is_ok());
        assert!(swap_monitor.name().is_ok());
    }
}
