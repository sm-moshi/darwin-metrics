use crate::{
    core::metrics::{
        hardware::{BatteryCapacityMonitorTrait, PowerStateMonitor},
        Metric,
    },
    error::Result,
    hardware::battery::types::BatteryCapacity,
    power::PowerState,
};

/// Monitor for battery capacity metrics including current, maximum, and design capacity
pub struct BatteryCapacityMonitor {
    device_id: String,
}

impl BatteryCapacityMonitor {
    /// Creates a new BatteryCapacityMonitor with the provided device ID
    pub fn new(device_id: String) -> Self {
        Self { device_id }
    }
}

#[async_trait::async_trait]
impl BatteryCapacityMonitorTrait for BatteryCapacityMonitor {
    type MetricType = BatteryCapacity;

    async fn name(&self) -> Result<String> {
        Ok("Battery Capacity".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Battery".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(format!("battery_capacity_{}", self.device_id))
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let capacity = BatteryCapacity { current: 80.0, maximum: 100.0, design: 100.0, cycle_count: 100 };
        Ok(Metric::new(capacity))
    }

    async fn current_capacity(&self) -> Result<f32> {
        Ok(80.0)
    }

    async fn maximum_capacity(&self) -> Result<f32> {
        Ok(100.0)
    }

    async fn design_capacity(&self) -> Result<f32> {
        Ok(100.0)
    }

    async fn cycle_count(&self) -> Result<u32> {
        Ok(100)
    }
}

#[async_trait::async_trait]
impl PowerStateMonitor for BatteryCapacityMonitor {
    async fn power_state(&self) -> Result<PowerState> {
        Ok(PowerState::Battery)
    }

    async fn battery_percentage(&self) -> Result<Option<f32>> {
        let current = self.current_capacity().await?;
        let maximum = self.maximum_capacity().await?;
        Ok(Some((current / maximum) * 100.0))
    }

    async fn time_remaining(&self) -> Result<Option<u32>> {
        // Return 60 minutes remaining
        Ok(Some(60))
    }

    async fn is_on_battery(&self) -> Result<bool> {
        Ok(true)
    }

    async fn is_charging(&self) -> Result<bool> {
        Ok(false)
    }
}
