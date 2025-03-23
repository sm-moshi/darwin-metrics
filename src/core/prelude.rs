// Re-export hardware monitoring traits from the new traits module
pub use crate::traits::{
    ByteMetricsMonitor, HardwareMonitor, MemoryMonitor, PowerConsumptionMonitor, PowerEventMonitor,
    PowerManagementMonitor, PowerStateMonitor, RateMonitor, StorageMonitor, TemperatureMonitor, UtilizationMonitor,
};
pub use crate::core::metrics::{Metric, MetricSeries};
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
pub use crate::resource::{Cache, ResourceManager, ResourceMonitor, ResourceMonitoring, ResourcePool, ResourceUpdate}; 