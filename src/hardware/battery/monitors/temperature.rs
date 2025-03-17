use crate::{
    core::metrics::{
        hardware::{HardwareMonitor, TemperatureMonitor as TemperatureMonitorTrait},
        Metric,
    },
    core::types::Temperature,
    error::{Error, Result},
    hardware::iokit::IOKit,
};
use async_trait::async_trait;

/// Monitor for battery temperature metrics
pub struct BatteryTemperatureMonitor {
    iokit: Box<dyn IOKit>,
    device_id: String,
}

impl BatteryTemperatureMonitor {
    /// Creates a new BatteryTemperatureMonitor with the provided IOKit implementation
    pub fn new(iokit: Box<dyn IOKit>) -> Self {
        Self { iokit, device_id: "battery0".to_string() }
    }

    // Private method to fetch battery temperature
    async fn fetch_battery_temperature(&self) -> Result<f64> {
        let temp = self.iokit.get_battery_temperature()?.ok_or_else(|| Error::NotAvailable {
            resource: "Battery temperature".to_string(),
            reason: "No battery temperature sensor available".to_string(),
        })?;
        Ok(temp)
    }
}

#[async_trait]
impl HardwareMonitor for BatteryTemperatureMonitor {
    type MetricType = Temperature;

    async fn name(&self) -> Result<String> {
        Ok("Battery Temperature".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("battery".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(self.device_id.clone())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let temp = self.temperature().await?;
        Ok(Metric::new(Temperature::new(temp)))
    }
}

#[async_trait]
impl TemperatureMonitorTrait for BatteryTemperatureMonitor {
    async fn temperature(&self) -> Result<f64> {
        self.fetch_battery_temperature().await
    }
}
