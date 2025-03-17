use super::TemperatureMonitor;
use crate::error::Result;
use crate::hardware::iokit::IOKit;
use crate::hardware::temperature::constants::*;
use std::sync::Arc;

/// Monitor for GPU temperature
pub struct GpuTemperatureMonitor {
    io_kit: Arc<Box<dyn IOKit>>,
}

impl GpuTemperatureMonitor {
    /// Create a new GPU temperature monitor
    pub fn new(io_kit: Arc<Box<dyn IOKit>>) -> Self {
        Self { io_kit }
    }
}

#[async_trait::async_trait]
impl TemperatureMonitor for GpuTemperatureMonitor {
    async fn name(&self) -> Result<String> {
        Ok("GPU Temperature Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("GPU".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("gpu0".to_string())
    }

    async fn temperature(&self) -> Result<f64> {
        let thermal_info = self.io_kit.get_thermal_info()?;
        thermal_info.gpu_temp.ok_or_else(|| crate::error::Error::NotAvailable {
            resource: "GPU temperature".to_string(),
            reason: "Not available".to_string(),
        })
    }

    async fn is_critical(&self) -> Result<bool> {
        let temp = self.temperature().await?;
        Ok(temp >= CRITICAL_TEMPERATURE_THRESHOLD)
    }

    async fn critical_threshold(&self) -> Result<f64> {
        Ok(CRITICAL_TEMPERATURE_THRESHOLD)
    }
}
