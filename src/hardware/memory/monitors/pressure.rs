use super::BaseMemoryMonitor;
use crate::{
    core::metrics::{hardware::HardwareMonitor, Metric},
    core::types::Percentage,
    error::{Error, Result},
    hardware::iokit::IOKit,
    hardware::memory::constants::{DEFAULT_CRITICAL_THRESHOLD, DEFAULT_WARNING_THRESHOLD},
    hardware::memory::types::MemoryInfo,
    hardware::memory::{PageStates, PressureLevel, SwapUsage},
};
use async_trait::async_trait;
use std::sync::Arc;

/// Memory pressure monitor for tracking system memory pressure
pub struct MemoryPressureMonitor {
    base: BaseMemoryMonitor,
    warning_threshold: f64,
    critical_threshold: f64,
}

impl MemoryPressureMonitor {
    /// Create a new memory pressure monitor
    pub fn new(iokit: Arc<Box<dyn IOKit>>) -> Self {
        Self {
            base: BaseMemoryMonitor::new(iokit),
            warning_threshold: DEFAULT_WARNING_THRESHOLD,
            critical_threshold: DEFAULT_CRITICAL_THRESHOLD,
        }
    }

    /// Set custom pressure thresholds
    pub fn set_thresholds(&mut self, warning: f64, critical: f64) -> Result<()> {
        if !(0.0..=100.0).contains(&warning) || !(0.0..=100.0).contains(&critical) {
            return Err(Error::InvalidArgument {
                context: "Thresholds must be between 0 and 100".into(),
                value: format!("warning: {}, critical: {}", warning, critical),
            });
        }
        if warning >= critical {
            return Err(Error::InvalidArgument {
                context: "Warning threshold must be less than critical threshold".into(),
                value: format!("warning: {}, critical: {}", warning, critical),
            });
        }
        self.warning_threshold = warning;
        self.critical_threshold = critical;
        Ok(())
    }
}

#[async_trait]
impl HardwareMonitor for MemoryPressureMonitor {
    type MetricType = Percentage;

    async fn name(&self) -> Result<String> {
        Ok("Memory Pressure".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Memory".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("memory_pressure".to_string())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let info = self.base.memory_info().await?;
        let pressure = info.pressure * 100.0;
        Ok(Metric::new(Percentage::from_f64(pressure)))
    }
}

#[async_trait]
impl super::MemoryMonitor for MemoryPressureMonitor {
    async fn memory_info(&self) -> Result<MemoryInfo> {
        let total = self.base.total_memory().await?;
        let page_states = self.base.vm_statistics().await?;
        let swap_usage = self.base.swap_usage().await?;
        let page_size = self.base.page_size().await?;

        let free = page_states.free * page_size;
        let used = total - free;
        let pressure = if total > 0 { used as f64 / total as f64 } else { 0.0 };

        Ok(MemoryInfo {
            total,
            free,
            used,
            active: page_states.active * page_size,
            inactive: page_states.inactive * page_size,
            wired: page_states.wired * page_size,
            compressed: page_states.compressed * page_size,
            pressure,
            page_size,
            page_states,
            swap_usage,
        })
    }

    async fn pressure_level(&self) -> Result<PressureLevel> {
        let pressure = self.get_metric().await?.value;

        if pressure.as_f64() >= self.critical_threshold {
            Ok(PressureLevel::Critical)
        } else if pressure.as_f64() >= self.warning_threshold {
            Ok(PressureLevel::Warning)
        } else {
            Ok(PressureLevel::Normal)
        }
    }

    async fn usage_percentage(&self) -> Result<f64> {
        let info = self.memory_info().await?;
        Ok(info.pressure * 100.0)
    }

    async fn pressure_percentage(&self) -> Result<f64> {
        let info = self.memory_info().await?;
        Ok(info.pressure * 100.0)
    }

    async fn page_states(&self) -> Result<PageStates> {
        self.base.vm_statistics().await
    }

    async fn swap_usage(&self) -> Result<SwapUsage> {
        self.base.swap_usage().await
    }
}
