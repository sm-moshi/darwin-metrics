use crate::{
    core::metrics::{hardware::TemperatureMonitor, Metric},
    error::{Error, Result},
    core::types::Temperature,
};

use async_trait::async_trait;

#[async_trait]
impl HardwareMonitor for MemoryTemperatureMonitor {
    type MetricType = Temperature;

    async fn name(&self) -> Result<String> {
        Ok("Memory Temperature Monitor".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Memory".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("memory0".to_string())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let temp = self.temperature().await?;
        Ok(Metric::new(temp))
    }
}

#[async_trait]
impl TemperatureMonitor for MemoryTemperatureMonitor {
    async fn temperature(&self) -> Result<Temperature> {
        // Implementation...
        Err(Error::NotImplemented("Memory temperature retrieval".to_string()))
    }
} 