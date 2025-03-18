use crate::core::metrics::Metric;
use crate::core::types::{ByteSize, DiskIO, Percentage, Temperature, Transfer};
use crate::error::Result;
use crate::{
    hardware::temperature::{Fan, ThermalMetrics},
    power::PowerState,
};
use async_trait::async_trait;
use std::time::Duration;
use std::time::SystemTime;

/// Trait for monitoring hardware components
///
/// This trait defines the core functionality for hardware monitoring.
/// Implementors should provide a way to get metrics for their specific hardware component.
///
/// # Examples
///
/// ```rust
/// use darwin_metrics::traits::HardwareMonitor;
/// use darwin_metrics::core::types::Temperature;
///
/// struct MyTemperatureMonitor;
///
/// #[async_trait::async_trait]
/// impl HardwareMonitor for MyTemperatureMonitor {
///     type MetricType = Temperature;
///
///     async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
///         // Implement your metric gathering logic here
///         todo!()
///     }
/// }
/// ```
#[async_trait]
pub trait HardwareMonitor: Send + Sync {
    /// The type of metric this monitor produces
    type MetricType: Clone + Send + Sync + 'static;

    /// Get the current metric value
    async fn get_metric(&self) -> Result<Metric<Self::MetricType>>;

    /// Get the name of the hardware component
    async fn name(&self) -> Result<String>;

    /// Get the type of hardware component
    async fn hardware_type(&self) -> Result<String>;

    /// Get the unique device identifier
    async fn device_id(&self) -> Result<String>;
}

/// Trait for temperature monitoring
///
/// Provides temperature-specific monitoring functionality.
#[async_trait]
pub trait TemperatureMonitor: HardwareMonitor<MetricType = Temperature> {
    /// Get the current temperature in Celsius
    async fn temperature(&self) -> Result<f64> {
        Ok(self.get_metric().await?.value.as_celsius())
    }
}

/// Trait for utilization monitoring
///
/// Provides utilization-specific monitoring functionality.
#[async_trait]
pub trait UtilizationMonitor: HardwareMonitor<MetricType = Percentage> {
    /// Get the current utilization percentage
    async fn utilization(&self) -> Result<f64> {
        Ok(self.get_metric().await?.value.as_f64())
    }
}

/// Trait for memory monitoring
///
/// Provides memory-specific monitoring functionality.
#[async_trait]
pub trait MemoryMonitor: Send + Sync {
    /// Get total memory in bytes
    async fn total(&self) -> Result<u64>;

    /// Get used memory in bytes
    async fn used(&self) -> Result<u64>;

    /// Get available memory in bytes
    async fn available(&self) -> Result<u64>;

    /// Get memory usage as a percentage
    async fn usage_percentage(&self) -> Result<f64> {
        let total = self.total().await?;
        let used = self.used().await?;
        Ok((used as f64 / total as f64) * 100.0)
    }
}

/// Monitor for metrics related to byte-based measurements (memory, storage, etc.)
#[async_trait]
pub trait ByteMetricsMonitor: Send + Sync {
    /// Get total bytes
    async fn total_bytes(&self) -> Result<u64>;
    /// Get used bytes
    async fn used_bytes(&self) -> Result<u64>;
    /// Get free bytes
    async fn free_bytes(&self) -> Result<u64>;
}

/// Monitor for rate-based metrics (network, disk I/O, etc.)
#[allow(async_fn_in_trait)]
pub trait RateMonitor<T>: Send + Sync {
    /// Get the current rate
    async fn rate(&self) -> Result<T>;
    /// Get the average rate over a period
    async fn average_rate(&self, seconds: u64) -> Result<T>;
}

/// Monitor for storage-related metrics
#[async_trait]
pub trait StorageMonitor: Send + Sync {
    /// Get total storage capacity
    async fn total_capacity(&self) -> Result<u64>;
    /// Get available storage capacity
    async fn available_capacity(&self) -> Result<u64>;
    /// Get used storage capacity
    async fn used_capacity(&self) -> Result<u64>;
}

/// Trait for monitoring power consumption metrics
#[async_trait]
pub trait PowerConsumptionMonitor: Send + Sync {
    /// Get total package power consumption in watts
    async fn package_power(&self) -> Result<f32>;
    /// Get CPU cores power consumption in watts
    async fn cores_power(&self) -> Result<f32>;
    /// Get GPU power consumption in watts
    async fn gpu_power(&self) -> Result<Option<f32>>;
    /// Get memory subsystem power consumption in watts
    async fn dram_power(&self) -> Result<Option<f32>>;
    /// Get Neural Engine power consumption in watts (Apple Silicon only)
    async fn neural_engine_power(&self) -> Result<Option<f32>>;
    /// Get total system power consumption in watts
    async fn total_power(&self) -> Result<f32>;
}

/// Trait for monitoring power state
#[async_trait]
pub trait PowerStateMonitor: Send + Sync {
    /// Get current power state (Battery, AC, Charging)
    async fn power_state(&self) -> Result<PowerState>;
    /// Get battery percentage if available
    async fn battery_percentage(&self) -> Result<Option<f32>>;
    /// Get estimated time remaining on battery in minutes
    async fn time_remaining(&self) -> Result<Option<u32>>;
    /// Check if system is running on battery power
    async fn is_on_battery(&self) -> Result<bool>;
    /// Check if battery is charging
    async fn is_charging(&self) -> Result<bool>;
}

/// Trait for monitoring power management
#[async_trait]
pub trait PowerManagementMonitor: Send + Sync {
    /// Check if system is currently thermal throttling
    async fn is_thermal_throttling(&self) -> Result<bool>;
    /// Get current power impact score (higher means more power drain)
    async fn power_impact(&self) -> Result<Option<f32>>;
    /// Get current thermal pressure level (0-100)
    async fn thermal_pressure(&self) -> Result<u32>;
    /// Get current performance mode
    async fn performance_mode(&self) -> Result<String>;
}

/// Trait for monitoring power events
#[async_trait]
pub trait PowerEventMonitor: Send + Sync {
    /// Get time since last wake from sleep
    async fn time_since_wake(&self) -> Result<Duration>;
    /// Get number of thermal throttling events
    async fn thermal_event_count(&self) -> Result<u32>;
    /// Get time until next scheduled sleep
    async fn time_until_sleep(&self) -> Result<Option<Duration>>;
    /// Check if system is preventing sleep
    async fn is_sleep_prevented(&self) -> Result<bool>;
}

/// Trait for monitoring battery health
#[async_trait]
pub trait BatteryHealthMonitor: Send + Sync {
    /// Get the battery cycle count
    async fn cycle_count(&self) -> Result<i64>;
    /// Get the battery health percentage (current capacity / design capacity)
    async fn health_percentage(&self) -> Result<f64>;
    /// Get whether the battery health is critical (< 80% of design capacity)
    async fn is_health_critical(&self) -> Result<bool>;
    /// Get whether the cycle count is critical (> 1000 cycles)
    async fn is_cycle_count_critical(&self) -> Result<bool>;
}

/// Power monitor trait for battery power monitoring
#[async_trait]
pub trait PowerMonitorTrait: Send + Sync {
    /// The type of metric this monitor produces
    type MetricType: Clone + Send + Sync + 'static;

    /// Get the current metric value
    async fn get_metric(&self) -> Result<Metric<Self::MetricType>>;

    /// Get the name of the hardware component
    async fn name(&self) -> Result<String>;

    /// Get the type of hardware component
    async fn hardware_type(&self) -> Result<String>;

    /// Get the unique device identifier
    async fn device_id(&self) -> Result<String>;

    /// Get the current power consumption in watts
    async fn power_consumption(&self) -> Result<f64> {
        Ok(0.0) // Default implementation
    }

    /// Get the current power state (charging, discharging, etc.)
    async fn power_state(&self) -> Result<PowerState> {
        Ok(PowerState::Unknown) // Default implementation
    }

    /// Get whether the device is currently charging
    async fn is_charging(&self) -> Result<bool> {
        Ok(false) // Default implementation
    }

    /// Get whether the device is on external power
    async fn is_external_power(&self) -> Result<bool> {
        Ok(false) // Default implementation
    }

    /// Get the estimated time remaining in seconds
    async fn time_remaining(&self) -> Result<i64> {
        Ok(0) // Default implementation
    }
}

/// Battery capacity monitor trait
#[async_trait]
pub trait BatteryCapacityMonitorTrait: Send + Sync {
    /// The type of metric this monitor produces
    type MetricType: Clone + Send + Sync + 'static;

    /// Get the current metric value
    async fn get_metric(&self) -> Result<Metric<Self::MetricType>>;

    /// Get the name of the hardware component
    async fn name(&self) -> Result<String>;

    /// Get the type of hardware component
    async fn hardware_type(&self) -> Result<String>;

    /// Get the unique device identifier
    async fn device_id(&self) -> Result<String>;

    /// Get the current capacity percentage
    async fn current_capacity(&self) -> Result<f32>;

    /// Get the maximum capacity percentage
    async fn maximum_capacity(&self) -> Result<f32>;

    /// Get the design capacity percentage
    async fn design_capacity(&self) -> Result<f32>;

    /// Get the battery cycle count
    async fn cycle_count(&self) -> Result<u32>;
}

/// Trait for monitoring network interfaces
#[async_trait]
pub trait NetworkInterfaceMonitor: Send + Sync {
    /// Get whether the interface is active (up and running)
    async fn is_active(&self) -> Result<bool>;
    /// Get whether the interface supports broadcast
    async fn supports_broadcast(&self) -> Result<bool>;
    /// Get whether the interface supports multicast
    async fn supports_multicast(&self) -> Result<bool>;
    /// Get whether this is a loopback interface
    async fn is_loopback(&self) -> Result<bool>;
    /// Get whether this is a wireless interface
    async fn is_wireless(&self) -> Result<bool>;
    /// Get the interface type (Ethernet, WiFi, etc.)
    async fn interface_type(&self) -> Result<String>;
    /// Get the interface MAC address if available
    async fn mac_address(&self) -> Result<Option<String>>;
}

/// Trait for monitoring network bandwidth
#[async_trait]
pub trait NetworkBandwidthMonitor: Send + Sync {
    /// Get total bytes received
    async fn bytes_received(&self) -> Result<u64>;
    /// Get total bytes sent
    async fn bytes_sent(&self) -> Result<u64>;
    /// Get current download speed in bytes per second
    async fn download_speed(&self) -> Result<f64>;
    /// Get current upload speed in bytes per second
    async fn upload_speed(&self) -> Result<f64>;
}

/// Trait for monitoring network packets
#[async_trait]
pub trait NetworkPacketMonitor: Send + Sync {
    /// Get total packets received
    async fn packets_received(&self) -> Result<u64>;
    /// Get total packets sent
    async fn packets_sent(&self) -> Result<u64>;
    /// Get packet receive rate (packets per second)
    async fn packet_receive_rate(&self) -> Result<f64>;
    /// Get packet send rate (packets per second)
    async fn packet_send_rate(&self) -> Result<f64>;
}

/// Trait for monitoring network errors
#[async_trait]
pub trait NetworkErrorMonitor: Send + Sync {
    /// Get total receive errors
    async fn receive_errors(&self) -> Result<u64>;
    /// Get total send errors
    async fn send_errors(&self) -> Result<u64>;
    /// Get total collisions
    async fn collisions(&self) -> Result<u64>;
    /// Get receive error rate (errors per packet)
    async fn receive_error_rate(&self) -> Result<f64>;
    /// Get send error rate (errors per packet)
    async fn send_error_rate(&self) -> Result<f64>;
}

/// Trait for monitoring system information
#[async_trait]
pub trait SystemInfoMonitor: Send + Sync {
    /// Get the system hostname
    async fn hostname(&self) -> Result<String>;
    /// Get the system architecture
    async fn architecture(&self) -> Result<String>;
    /// Get the operating system version
    async fn os_version(&self) -> Result<String>;
    /// Get the kernel version
    async fn kernel_version(&self) -> Result<String>;
}

/// Trait for monitoring system load
#[async_trait]
pub trait SystemLoadMonitor: Send + Sync {
    /// Get the system load average for 1 minute
    async fn load_average_1(&self) -> Result<f64>;
    /// Get the system load average for 5 minutes
    async fn load_average_5(&self) -> Result<f64>;
    /// Get the system load average for 15 minutes
    async fn load_average_15(&self) -> Result<f64>;
    /// Get the number of processes
    async fn process_count(&self) -> Result<u32>;
    /// Get the number of threads
    async fn thread_count(&self) -> Result<u32>;
}

/// Trait for monitoring system uptime
#[async_trait]
pub trait SystemUptimeMonitor: Send + Sync {
    /// Get the system uptime in seconds
    async fn uptime_seconds(&self) -> Result<u64>;
    /// Get the system boot time as Unix timestamp
    async fn boot_time(&self) -> Result<u64>;
}

/// Trait for monitoring system resources
#[async_trait]
pub trait SystemResourceMonitor: Send + Sync {
    /// Get the number of physical CPU cores
    async fn physical_cpu_count(&self) -> Result<u32>;
    /// Get the number of logical CPU cores
    async fn logical_cpu_count(&self) -> Result<u32>;
    /// Get the total physical memory in bytes
    async fn total_memory(&self) -> Result<u64>;
    /// Get the total swap space in bytes
    async fn total_swap(&self) -> Result<u64>;
}

/// Trait for monitoring disk performance
#[async_trait]
pub trait DiskPerformanceMonitor: Send + Sync {
    /// Get read operations per second
    async fn read_ops_per_second(&self) -> Result<f64>;
    /// Get write operations per second
    async fn write_ops_per_second(&self) -> Result<f64>;
    /// Get read latency in milliseconds
    async fn read_latency_ms(&self) -> Result<f64>;
    /// Get write latency in milliseconds
    async fn write_latency_ms(&self) -> Result<f64>;
    /// Get disk queue depth
    async fn queue_depth(&self) -> Result<f64>;
}

/// Trait for monitoring disk health
#[async_trait]
pub trait DiskHealthMonitor: HardwareMonitor {
    /// Get disk type (SSD, HDD, etc.)
    async fn disk_type(&self) -> Result<String>;
    /// Get disk name/label
    async fn disk_name(&self) -> Result<String>;
    /// Get filesystem type
    async fn filesystem_type(&self) -> Result<String>;
    /// Check if disk is nearly full (>90% usage)
    async fn is_nearly_full(&self) -> Result<bool>;
    /// Check if this is a boot volume
    async fn is_boot_volume(&self) -> Result<bool>;
    /// Get SMART status
    async fn smart_status(&self) -> Result<bool>;
    /// Get temperature in Celsius
    async fn temperature(&self) -> Result<f32>;
    /// Get power-on hours
    async fn power_on_hours(&self) -> Result<u32>;
    /// Get reallocated sectors
    async fn reallocated_sectors(&self) -> Result<u32>;
    /// Get pending sectors
    async fn pending_sectors(&self) -> Result<u32>;
    /// Get uncorrectable sectors
    async fn uncorrectable_sectors(&self) -> Result<u32>;
    /// Get last check time
    async fn last_check(&self) -> Result<SystemTime>;
}

/// Trait for monitoring disk mounts
#[async_trait]
pub trait DiskMountMonitor: HardwareMonitor {
    /// Get mount point path
    async fn mount_point(&self) -> Result<String>;
    /// Get filesystem type
    async fn filesystem_type(&self) -> Result<String>;
    /// Check if disk is mounted
    async fn is_mounted(&self) -> Result<bool>;
    /// Get mount options
    async fn mount_options(&self) -> Result<Vec<String>>;
    /// Check if this is a boot volume
    async fn is_boot_volume(&self) -> Result<bool>;
    /// Check if this is a readonly volume
    async fn is_readonly(&self) -> Result<bool>;
    /// Check if this is a removable volume
    async fn is_removable(&self) -> Result<bool>;
    /// Check if this is a network volume
    async fn is_network(&self) -> Result<bool>;
}

/// Trait for monitoring process information
#[async_trait]
pub trait ProcessInfoMonitor: Send + Sync {
    /// Get process ID
    async fn pid(&self) -> Result<u32>;
    /// Get process name
    async fn name(&self) -> Result<String>;
    /// Get parent process ID
    async fn parent_pid(&self) -> Result<Option<u32>>;
    /// Get process start time
    async fn start_time(&self) -> Result<SystemTime>;
    /// Check if this is a system process
    async fn is_system_process(&self) -> Result<bool>;
}

/// Trait for monitoring process resources
#[async_trait]
pub trait ProcessResourceMonitor: Send + Sync {
    /// Get CPU usage percentage (0-100)
    async fn cpu_usage(&self) -> Result<f64>;
    /// Get memory usage in bytes
    async fn memory_usage(&self) -> Result<u64>;
    /// Get number of threads
    async fn thread_count(&self) -> Result<u32>;
    /// Check if process is suspended
    async fn is_suspended(&self) -> Result<bool>;
}

/// Trait for monitoring process I/O
#[async_trait]
pub trait ProcessIOMonitor: Send + Sync {
    /// Get total bytes read from disk
    async fn bytes_read(&self) -> Result<u64>;
    /// Get total bytes written to disk
    async fn bytes_written(&self) -> Result<u64>;
    /// Get total read operations
    async fn read_operations(&self) -> Result<u64>;
    /// Get total write operations
    async fn write_operations(&self) -> Result<u64>;
    /// Get read rate in bytes per second
    async fn read_rate(&self) -> Result<f64>;
    /// Get write rate in bytes per second
    async fn write_rate(&self) -> Result<f64>;
}

/// Trait for monitoring process relationships
#[async_trait]
pub trait ProcessRelationshipMonitor: Send + Sync {
    /// Get child process IDs
    async fn child_pids(&self) -> Result<Vec<u32>>;
    /// Get sibling process IDs (processes with same parent)
    async fn sibling_pids(&self) -> Result<Vec<u32>>;
    /// Get process tree depth (distance from init process)
    async fn tree_depth(&self) -> Result<u32>;
    /// Get process group ID
    async fn process_group_id(&self) -> Result<u32>;
}

/// Trait for monitoring CPU metrics
#[async_trait]
pub trait CpuMonitor {
    /// Get the current CPU frequency in MHz
    async fn frequency(&self) -> Result<f64>;

    /// Get the minimum CPU frequency in MHz
    async fn min_frequency(&self) -> Result<f64>;

    /// Get the maximum CPU frequency in MHz
    async fn max_frequency(&self) -> Result<f64>;

    /// Get the available CPU frequency steps in MHz
    async fn available_frequencies(&self) -> Result<Vec<f64>>;

    /// Get the number of physical CPU cores
    async fn physical_cores(&self) -> Result<u32>;

    /// Get the number of logical CPU cores
    async fn logical_cores(&self) -> Result<u32>;

    /// Get the CPU model name
    async fn model_name(&self) -> Result<String>;

    /// Get the current temperature in Celsius
    async fn temperature(&self) -> Result<Option<f64>>;

    /// Get the current power consumption in watts
    async fn power_consumption(&self) -> Result<Option<f64>>;

    /// Get per-core usage percentages
    async fn core_usage(&self) -> Result<Vec<f64>>;

    /// Get overall CPU usage percentage
    async fn total_usage(&self) -> Result<f64>;
}

/// Trait for monitoring GPU metrics
#[async_trait]
pub trait GpuMonitor {
    /// Get the GPU model name
    async fn name(&self) -> Result<String>;

    /// Get the current GPU utilization percentage
    async fn utilization(&self) -> Result<f64>;

    /// Get the current GPU temperature in Celsius
    async fn temperature(&self) -> Result<Option<f32>>;

    /// Get the total GPU memory in bytes
    async fn total_memory(&self) -> Result<u64>;

    /// Get the used GPU memory in bytes
    async fn used_memory(&self) -> Result<u64>;

    /// Get the free GPU memory in bytes
    async fn free_memory(&self) -> Result<u64>;

    /// Get the memory utilization percentage
    async fn memory_utilization(&self) -> Result<f64>;

    /// Get whether the GPU supports hardware acceleration
    async fn supports_hardware_acceleration(&self) -> Result<bool>;

    /// Get the current memory bandwidth usage in bytes per second
    async fn memory_bandwidth(&self) -> Result<Option<u64>>;
}

/// Trait for monitoring thermal metrics
#[async_trait]
pub trait ThermalMonitor {
    /// Get CPU temperature in Celsius
    async fn cpu_temperature(&self) -> Result<Option<f64>>;

    /// Get GPU temperature in Celsius
    async fn gpu_temperature(&self) -> Result<Option<f64>>;

    /// Get memory temperature in Celsius
    async fn memory_temperature(&self) -> Result<Option<f64>>;

    /// Get battery temperature in Celsius
    async fn battery_temperature(&self) -> Result<Option<f64>>;

    /// Get ambient temperature in Celsius
    async fn ambient_temperature(&self) -> Result<Option<f64>>;

    /// Get whether the system is thermal throttling
    async fn is_throttling(&self) -> Result<bool>;

    /// Get fan information
    async fn get_fans(&self) -> Result<Vec<Fan>>;

    /// Get thermal metrics for all components
    async fn get_thermal_metrics(&self) -> Result<ThermalMetrics>;
}

/// Trait for monitoring disk IO metrics
#[async_trait]
pub trait DiskIOMonitor: HardwareMonitor {
    /// Get current disk I/O metrics
    async fn get_io(&self) -> Result<DiskIO>;
    /// Get current transfer rate in bytes per second
    async fn get_transfer_rate(&self) -> Result<Transfer>;
}

/// Trait for monitoring disk storage metrics
#[async_trait]
pub trait DiskStorageMonitor: StorageMonitor {
    /// Get total disk space
    async fn total_space(&self) -> Result<ByteSize>;
    /// Get used disk space
    async fn used_space(&self) -> Result<ByteSize>;
    /// Get available disk space
    async fn available_space(&self) -> Result<ByteSize>;
    /// Get disk usage percentage
    async fn usage_percentage(&self) -> Result<Percentage>;
}

/// Trait for monitoring disk utilization
#[async_trait]
pub trait DiskUtilizationMonitor: UtilizationMonitor {
    /// Get read utilization percentage
    async fn get_read_utilization(&self) -> Result<Percentage>;
    /// Get write utilization percentage
    async fn get_write_utilization(&self) -> Result<Percentage>;
} 