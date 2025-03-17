use super::TemperatureMonitor;
use crate::error::Result;
use crate::hardware::iokit::IOKit;
use crate::hardware::temperature::constants::*;
use std::sync::Arc;

/// Monitor for CPU temperature
pub struct CpuTemperatureMonitor {
    io_kit: Arc<Box<dyn IOKit>>,
}

impl CpuTemperatureMonitor {
    /// Create a new CPU temperature monitor
    pub fn new(io_kit: Arc<Box<dyn IOKit>>) -> Self {
        Self { io_kit }
    }
}

#[async_trait::async_trait]
impl TemperatureMonitor for CpuTemperatureMonitor {
    async fn name(&self) -> Result<String> {
        Ok("CPU Temperature Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("CPU".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("cpu0".to_string())
    }

    async fn temperature(&self) -> Result<f64> {
        let thermal_info = self.io_kit.get_thermal_info()?;
        Ok(thermal_info.cpu_temp)
    }

    async fn is_critical(&self) -> Result<bool> {
        let temp = self.temperature().await?;
        Ok(temp >= CRITICAL_TEMPERATURE_THRESHOLD)
    }

    async fn critical_threshold(&self) -> Result<f64> {
        Ok(CRITICAL_TEMPERATURE_THRESHOLD)
    }
}
