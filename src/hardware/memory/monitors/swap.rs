use super::BaseMemoryMonitor;
use crate::hardware::iokit::IOKit;
use crate::hardware::memory::types::{MemoryInfo, PageStates, PressureLevel, SwapUsage};
use crate::{
    core::metrics::{
        hardware::{HardwareMonitor, MemoryMonitor},
        Metric,
    },
    core::types::Percentage,
    error::{Error, Result},
};

use async_trait::async_trait;
use std::sync::Arc;

/// Swap usage monitor for tracking system swap activity
pub struct SwapMonitor {
    base: BaseMemoryMonitor,
}

impl SwapMonitor {
    /// Create a new swap monitor
    pub fn new(iokit: Arc<Box<dyn IOKit>>) -> Self {
        Self { base: BaseMemoryMonitor::new(iokit) }
    }
}

#[async_trait]
impl HardwareMonitor for SwapMonitor {
    type MetricType = Percentage;

    async fn name(&self) -> Result<String> {
        Ok("Swap Usage Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Memory".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("swap0".to_string())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let percentage = self.usage_percentage().await?;
        Ok(Metric::new(Percentage::from_f64(percentage)))
    }
}

#[async_trait]
impl super::MemoryMonitor for SwapMonitor {
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
        let swap = self.base.swap_usage().await?;
        Ok(if swap.pressure >= 0.85 {
            PressureLevel::Critical
        } else if swap.pressure >= 0.75 {
            PressureLevel::Warning
        } else {
            PressureLevel::Normal
        })
    }

    async fn usage_percentage(&self) -> Result<f64> {
        let swap = self.base.swap_usage().await?;
        Ok(if swap.total > 0 { (swap.used as f64 / swap.total as f64) * 100.0 } else { 0.0 })
    }

    async fn pressure_percentage(&self) -> Result<f64> {
        let swap = self.base.swap_usage().await?;
        Ok(swap.pressure * 100.0)
    }

    async fn page_states(&self) -> Result<PageStates> {
        self.base.vm_statistics().await
    }

    async fn swap_usage(&self) -> Result<SwapUsage> {
        self.base.swap_usage().await
    }
}

#[async_trait]
impl MemoryMonitor for SwapMonitor {
    async fn total(&self) -> Result<u64> {
        // Implementation...
        Err(Error::NotImplemented { feature: "Total swap memory retrieval".to_string() })
    }

    async fn used(&self) -> Result<u64> {
        // Implementation...
        Err(Error::NotImplemented { feature: "Used swap memory retrieval".to_string() })
    }

    async fn available(&self) -> Result<u64> {
        // Implementation...
        Err(Error::NotImplemented { feature: "Available swap memory retrieval".to_string() })
    }

    async fn usage_percentage(&self) -> Result<f64> {
        let swap = self.base.swap_usage().await?;
        Ok(if swap.total > 0 { (swap.used as f64 / swap.total as f64) * 100.0 } else { 0.0 })
    }
}
