//! # Core Module
//!
//! The core module provides fundamental types and metrics collection functionality
//! for the darwin-metrics library. It serves as the foundation for system metric
//! collection on macOS systems.
//!
//! ## Organization
//!
//! * Types - Core data types for representing various metrics
//! * Metrics - Hardware monitoring and metric collection interfaces
//!
//! ## Example
//!
//! ```rust
//! use darwin_metrics::core::types::ByteSize;
//!
//! let size = ByteSize::from_bytes(1024);
//! assert_eq!(size.as_kb(), 1.0);
//! ```

// Core modules
/// Hardware monitoring and metric collection interfaces
pub mod metrics;
/// Core data types for system metrics
pub mod types;

/// Core prelude module that re-exports commonly used types and traits
pub mod prelude {
    // Re-export core metric types
    // Re-export hardware monitoring traits from the new traits module
    pub use super::metrics::{Metric, MetricSeries};
    // Re-export hardware-specific types
    pub use crate::battery::{
        monitors::BatteryCapacityMonitor, monitors::BatteryHealthMonitor, monitors::BatteryPowerMonitor,
        monitors::BatteryTemperatureMonitor, types::BatteryInfo,
    };
    // Re-export core types

    // Import these from their respective modules
    pub use crate::disk::{ByteMetrics, DiskConfig, DiskType};
    pub use crate::gpu::GpuMetrics;
    // Re-export resource management types
    pub use crate::resource::{
        Cache, ResourceManager, ResourceMonitor, ResourceMonitoring, ResourcePool, ResourceUpdate,
    };
    pub use crate::traits::{
        ByteMetricsMonitor, HardwareMonitor, MemoryMonitor, PowerConsumptionMonitor, PowerEventMonitor,
        PowerManagementMonitor, PowerStateMonitor, RateMonitor, StorageMonitor, TemperatureMonitor, UtilizationMonitor,
    };
}

// Re-export error types at the root level

// Re-export core types and metrics

// Do not re-export at the root level to avoid conflicts
// Instead, encourage users to use the prelude
