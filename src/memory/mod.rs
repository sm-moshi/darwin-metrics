//! Memory analysis and monitoring module
//!
//! This module provides comprehensive memory metrics and monitoring capabilities for macOS systems. It tracks system
//! memory usage, page states, memory pressure, and swap activity.
//!
//! # Features
//!
//! - System memory metrics (total, available, used, wired)
//! - Detailed page states (active, inactive, wired, free, compressed)
//! - Memory pressure monitoring with configurable thresholds
//! - Swap usage tracking with activity rates
//! - Memory pressure callbacks for event-driven monitoring
//!
//! # Examples
//!
//! Basic memory information:
//!
//! ```no_run
//! use darwin_metrics::memory::{Memory, MemoryMonitor};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let memory = Memory::new()?;
//!     let info = memory.memory_info().await?;
//!     
//!     println!("Total Memory: {} bytes", info.total);
//!     println!("Available Memory: {} bytes", info.free);
//!     println!("Used Memory: {} bytes", info.used);
//!     println!("Memory Usage: {:.1}%", memory.usage_percentage().await?);
//!     println!("Memory Pressure: {:.1}%", memory.pressure_percentage().await?);
//!     
//!     Ok(())
//! }
//! ```
//!
//! Monitoring memory pressure changes:
//!
//! ```no_run
//! use darwin_metrics::memory::{Memory, MemoryMonitor, PressureLevel};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let memory = Memory::new()?;
//!     
//!     loop {
//!         let level = memory.pressure_level().await?;
//!         match level {
//!             PressureLevel::Normal => println!("Memory pressure is NORMAL"),
//!             PressureLevel::Warning => println!("Memory pressure is HIGH"),
//!             PressureLevel::Critical => println!("Memory pressure is CRITICAL"),
//!             // Handle any future variants
//!             _ => println!("Memory pressure is in an UNKNOWN state"),
//!         }
//!         
//!         tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
//!     }
//! }
//! ```
//!
//! For more examples, see the `examples/memory_monitor.rs` file.

/// Memory monitoring constants
pub mod constants;

/// Memory monitoring implementation
pub mod monitors;

/// Memory data types
pub mod types;

pub use monitors::*;
pub use types::*;

use crate::error::Result;
use crate::hardware::iokit::{IOKit, IOKitImpl};
use crate::traits;
use async_trait::async_trait;

use std::sync::Arc;

/// Memory monitor trait that extends the base MemoryMonitor trait
/// with additional memory-specific functionality
#[async_trait]
pub trait MemoryMonitor: traits::MemoryMonitor {
    /// Get detailed memory information
    async fn memory_info(&self) -> Result<MemoryInfo>;

    /// Get current memory pressure level
    async fn pressure_level(&self) -> Result<PressureLevel>;

    /// Get memory pressure percentage
    async fn pressure_percentage(&self) -> Result<f64>;

    /// Get detailed page states
    async fn page_states(&self) -> Result<PageStates>;

    /// Get swap usage information
    async fn swap_usage(&self) -> Result<SwapUsage>;
}

/// Memory monitoring implementation
///
/// This struct provides access to memory monitoring capabilities through separate monitor instances
/// for memory usage, pressure, and swap metrics.
#[derive(Debug, Clone)]
pub struct Memory {
    iokit: Arc<Box<dyn IOKit>>,
}

impl Memory {
    /// Creates a new Memory instance with current system information
    pub fn new() -> Result<Self> {
        let iokit_impl = IOKitImpl::new()?;
        Ok(Self { iokit: Arc::new(Box::new(iokit_impl)) })
    }

    /// Get a monitor for memory usage metrics
    pub fn usage_monitor(&self) -> MemoryUsageMonitor {
        MemoryUsageMonitor::new(Arc::clone(&self.iokit))
    }

    /// Get a monitor for memory pressure metrics
    pub fn pressure_monitor(&self) -> MemoryPressureMonitor {
        MemoryPressureMonitor::new(Arc::clone(&self.iokit))
    }

    /// Get a monitor for swap usage metrics
    pub fn swap_monitor(&self) -> SwapMonitor {
        SwapMonitor::new(Arc::clone(&self.iokit))
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self { iokit: Arc::new(Box::new(IOKitImpl::default())) }
    }
}

// Memory Monitor trait implementations
#[async_trait]
impl traits::MemoryMonitor for Memory {
    async fn total(&self) -> Result<u64> {
        self.usage_monitor().total().await
    }

    async fn used(&self) -> Result<u64> {
        self.usage_monitor().used().await
    }

    async fn available(&self) -> Result<u64> {
        self.usage_monitor().available().await
    }

    async fn usage_percentage(&self) -> Result<f64> {
        self.usage_monitor().usage_percentage().await
    }
}

#[async_trait]
impl MemoryMonitor for Memory {
    async fn memory_info(&self) -> Result<MemoryInfo> {
        self.usage_monitor().memory_info().await
    }

    async fn pressure_level(&self) -> Result<PressureLevel> {
        self.pressure_monitor().pressure_level().await
    }

    async fn pressure_percentage(&self) -> Result<f64> {
        self.pressure_monitor().pressure_percentage().await
    }

    async fn page_states(&self) -> Result<PageStates> {
        self.usage_monitor().page_states().await
    }

    async fn swap_usage(&self) -> Result<SwapUsage> {
        self.swap_monitor().swap_usage().await
    }
}
