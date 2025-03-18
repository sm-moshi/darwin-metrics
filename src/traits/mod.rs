// Traits module
//
// This module contains all the trait definitions used across the codebase.
// It provides a single, centralized location for all trait definitions,
// improving code organization and reducing circular dependencies.

pub mod hardware;

// Re-export hardware monitoring traits for external use
pub use hardware::{
    BatteryCapacityMonitorTrait, BatteryHealthMonitor, ByteMetricsMonitor, CpuMonitor, DiskHealthMonitor,
    DiskMountMonitor, DiskPerformanceMonitor, GpuMonitor, HardwareMonitor, MemoryMonitor, NetworkBandwidthMonitor,
    NetworkErrorMonitor, NetworkInterfaceMonitor, NetworkPacketMonitor, PowerConsumptionMonitor, PowerEventMonitor,
    PowerManagementMonitor, PowerMonitorTrait, PowerStateMonitor, ProcessIOMonitor, ProcessInfoMonitor,
    ProcessRelationshipMonitor, ProcessResourceMonitor, RateMonitor, StorageMonitor, SystemInfoMonitor,
    SystemLoadMonitor, SystemResourceMonitor, SystemUptimeMonitor, TemperatureMonitor, ThermalMonitor,
    UtilizationMonitor,
};
