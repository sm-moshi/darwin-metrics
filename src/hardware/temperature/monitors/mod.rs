use crate::error::Result;
use crate::hardware::iokit::IOKit;
use async_trait::async_trait;
use std::sync::Arc;

mod ambient;
mod battery;
mod cpu;
mod fan;
mod gpu;

pub use ambient::AmbientTemperatureMonitor;
pub use battery::BatteryTemperatureMonitor;
pub use cpu::CpuTemperatureMonitor;
pub use fan::FanMonitor;
pub use gpu::GpuTemperatureMonitor;

/// Common trait for all temperature monitors
#[async_trait]
pub trait TemperatureMonitor: Send + Sync {
    /// Get the name of the monitor
    async fn name(&self) -> Result<String>;

    /// Get the type of hardware being monitored
    async fn hardware_type(&self) -> Result<String>;

    /// Get the unique device identifier
    async fn device_id(&self) -> Result<String>;

    /// Get the current temperature in degrees Celsius
    async fn temperature(&self) -> Result<f64>;

    /// Check if temperature is at critical level
    async fn is_critical(&self) -> Result<bool>;

    /// Get the critical temperature threshold in degrees Celsius
    async fn critical_threshold(&self) -> Result<f64>;
}

/// Trait for fan monitoring
#[async_trait::async_trait]
pub trait FanMonitoring {
    /// Get the current fan speed in RPM
    async fn speed_rpm(&self) -> Result<u32>;

    /// Get the minimum fan speed in RPM
    async fn min_speed(&self) -> Result<u32>;

    /// Get the maximum fan speed in RPM
    async fn max_speed(&self) -> Result<u32>;

    /// Get the current fan utilization as a percentage (0-100%)
    async fn percentage(&self) -> Result<f64>;

    /// Get the fan name/identifier
    async fn fan_name(&self) -> Result<String>;
}

/// Creates a new temperature monitor with the given IOKit implementation
pub fn create_monitor<T: IOKit + Clone + 'static>(
    io_kit: Arc<Box<dyn IOKit>>,
    monitor_type: &str,
) -> Result<Box<dyn TemperatureMonitor>> {
    match monitor_type {
        "cpu" => Ok(Box::new(CpuTemperatureMonitor::new(io_kit))),
        "gpu" => Ok(Box::new(GpuTemperatureMonitor::new(io_kit))),
        "ambient" => Ok(Box::new(AmbientTemperatureMonitor::new(io_kit))),
        "battery" => Ok(Box::new(BatteryTemperatureMonitor::new(io_kit))),
        _ => Err(crate::error::Error::InvalidMonitorType(monitor_type.to_string())),
    }
}
