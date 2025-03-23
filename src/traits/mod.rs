//! # Traits Module
//!
//! The traits module provides core traits that define the behavior and capabilities
//! of various system components in the darwin-metrics library. These traits serve
//! as interfaces for implementing hardware monitoring and metric collection.
//!
//! ## Features
//!
//! * Hardware monitoring traits for system components
//! * Metric collection interfaces
//! * Common trait definitions for system monitoring
//!
//! ## Async Trait Implementation
//!
//! Most traits in this module use the `#[async_trait::async_trait]` macro to enable
//! async methods in traits. This approach is being gradually migrated to Rust's native
//! async trait support as it becomes more stable.
//!
//! Some traits like `RateMonitor<T>` already use native async trait support with
//! `#[allow(async_fn_in_trait)]`. When implementing these traits, you don't need
//! to use the `async_trait` macro on your implementation.
//!
//! ### Migration Notice
//!
//! In future versions, more traits will migrate to native async trait support as
//! Rust's async trait feature stabilizes.
//!
//! ## Example
//!
//! ```rust
//! use darwin_metrics::traits::hardware::{TemperatureMonitor, UtilizationMonitor};
//!
//! // Implement temperature monitoring for a custom device
//! struct MyDevice;
//!
//! impl TemperatureMonitor for MyDevice {
//!     async fn temperature(&self) -> Result<f64, Box<dyn std::error::Error>> {
//!         // Implementation details...
//!         Ok(42.0)
//!     }
//! }
//! ```

/// Hardware monitoring and metric collection traits
pub mod hardware;

// Re-export hardware monitoring traits for external use
pub use hardware::{
    BatteryCapacityMonitorTrait, BatteryHealthMonitor, ByteMetricsMonitor, CpuMonitor, DiskHealthMonitor,
    DiskMountMonitor, DiskPerformanceMonitor, FanMonitor, GpuMonitor, HardwareMonitor, MemoryMonitor,
    NetworkBandwidthMonitor, NetworkErrorMonitor, NetworkInterfaceMonitor, NetworkPacketMonitor,
    PowerConsumptionMonitor, PowerEventMonitor, PowerManagementMonitor, PowerMonitorTrait, PowerStateMonitor,
    ProcessIOMonitor, ProcessInfoMonitor, ProcessRelationshipMonitor, ProcessResourceMonitor, RateMonitor,
    StorageMonitor, SystemInfoMonitor, SystemLoadMonitor, SystemResourceMonitor, SystemUptimeMonitor,
    TemperatureMonitor, ThermalMonitor, UtilizationMonitor,
};
