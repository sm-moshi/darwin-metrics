use super::TemperatureMonitor;
use crate::error::Result;
use crate::hardware::iokit::IOKit;
use crate::hardware::temperature::constants::*;
use std::sync::Arc;

/// Monitor for battery temperature
pub struct BatteryTemperatureMonitor {
    io_kit: Arc<Box<dyn IOKit>>,
}

impl BatteryTemperatureMonitor {
    /// Create a new battery temperature monitor
    pub fn new(io_kit: Arc<Box<dyn IOKit>>) -> Self {
        Self { io_kit }
    }
}

#[async_trait::async_trait]
impl TemperatureMonitor for BatteryTemperatureMonitor {
    async fn name(&self) -> Result<String> {
        Ok("Battery Temperature Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Battery".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("battery0".to_string())
    }

    async fn temperature(&self) -> Result<f64> {
        let thermal_info = self.io_kit.get_thermal_info()?;
        thermal_info.battery_temp.ok_or_else(|| crate::error::Error::NotAvailable {
            resource: "Battery temperature".to_string(),
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
