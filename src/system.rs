//! # System Module
//!
//! This module provides the main system interface for accessing various hardware and system metrics
//! on macOS systems. It serves as the primary entry point for collecting system-wide metrics.
//!
//! ## Features
//!
//! * Access to IOKit functionality
//! * System-wide metric collection
//! * Hardware monitoring capabilities
//!
//! ## Example
//!
//! ```rust
//! use darwin_metrics::System;
//!
//! let system = System::new().expect("Failed to initialize system");
//! let io_kit = system.io_kit();
//! ```

use std::sync::Arc;

use crate::error::Result;
use crate::hardware::iokit::{IOKit, IOKitImpl};

/// Central system information provider
#[derive(Debug)]
pub struct System {
    /// IOKit interface for hardware monitoring access
    io_kit: Arc<Box<dyn IOKit>>,
}

impl System {
    /// Creates a new System instance
    ///
    /// # Returns
    ///
    /// A Result containing the new System instance or an error if initialization fails
    ///
    /// # Errors
    ///
    /// Returns an error if system initialization fails
    pub fn new() -> Result<Self> {
        let io_kit_impl = IOKitImpl::new()?;
        Ok(Self {
            io_kit: Arc::new(Box::new(io_kit_impl)),
        })
    }

    /// Returns a reference to the IOKit interface
    ///
    /// # Returns
    ///
    /// An Arc-wrapped Box containing the IOKit implementation
    pub fn io_kit(&self) -> Arc<Box<dyn IOKit>> {
        Arc::clone(&self.io_kit)
    }
}

impl Default for System {
    fn default() -> Self {
        Self::new().expect("Failed to create default System instance")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::iokit::MockIOKit;

    #[test]
    fn test_system_creation() {
        let system = System::new();
        assert!(system.is_ok());
    }

    #[test]
    fn test_system_with_mock() {
        let mock_iokit = MockIOKit::new().expect("Failed to create mock IOKit");
        let system = System {
            io_kit: Arc::new(Box::new(mock_iokit)),
        };
        assert!(Arc::strong_count(&system.io_kit) == 1);
    }

    #[test]
    fn test_io_kit_cloning() {
        let system = System::new().expect("Failed to create system");
        let io_kit1 = system.io_kit();
        let io_kit2 = system.io_kit();
        assert!(Arc::strong_count(&io_kit1) == 3); // Original + 2 clones
    }
}
