use crate::hardware::cpu::CPU;
use crate::{
    core::metrics::{
        hardware::{HardwareMonitor, TemperatureMonitor},
        Metric,
    },
    core::types::Temperature,
    error::Result,
};
use async_trait::async_trait;

/// Monitor for CPU temperature metrics
pub struct CpuTemperatureMonitor {
    cpu: CPU,
    device_id: String,
}

impl CpuTemperatureMonitor {
    /// Creates a new CpuTemperatureMonitor with the provided CPU and device ID
    pub fn new(cpu: CPU, device_id: String) -> Self {
        Self { cpu, device_id }
    }
}

#[async_trait]
impl HardwareMonitor for CpuTemperatureMonitor {
    type MetricType = Temperature;

    async fn name(&self) -> Result<String> {
        Ok("CPU Temperature".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("CPU".to_string())
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
impl TemperatureMonitor for CpuTemperatureMonitor {
    async fn temperature(&self) -> Result<f64> {
        self.cpu.temperature_monitor().temperature().await
    }
}

// ... existing code ...
