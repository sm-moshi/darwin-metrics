use async_trait::async_trait;

use crate::battery::types::{BatteryCapacity, BatteryPower};
use crate::core::metrics::Metric;
use crate::core::types::Temperature;
use crate::error::Result;
use crate::power::PowerState;
use crate::traits::{
    BatteryCapacityMonitorTrait, BatteryHealthMonitor as BatteryHealthMonitorTrait, HardwareMonitor, PowerMonitorTrait,
    PowerStateMonitor, TemperatureMonitor,
};

//=============================================================================
// Battery Capacity Monitor
//=============================================================================

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
        let capacity = BatteryCapacity {
            current: 80.0,
            maximum: 100.0,
            design: 100.0,
            cycle_count: 100,
        };
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

//=============================================================================
// Battery Health Monitor
//=============================================================================

/// Monitor for battery health metrics
pub struct BatteryHealthMonitor {
    device_id: String,
}

impl BatteryHealthMonitor {
    /// Creates a new BatteryHealthMonitor with the provided device ID
    pub fn new(device_id: String) -> Self {
        Self { device_id }
    }
}

#[async_trait]
impl BatteryHealthMonitorTrait for BatteryHealthMonitor {
    async fn cycle_count(&self) -> Result<i64> {
        Ok(100)
    }

    async fn health_percentage(&self) -> Result<f64> {
        Ok(95.0)
    }

    async fn is_health_critical(&self) -> Result<bool> {
        let health = self.health_percentage().await?;
        Ok(health < 80.0)
    }

    async fn is_cycle_count_critical(&self) -> Result<bool> {
        let cycles = self.cycle_count().await?;
        Ok(cycles > 1000)
    }
}

//=============================================================================
// Battery Power Monitor
//=============================================================================

/// Monitor for battery power consumption
pub struct BatteryPowerMonitor {
    device_id: String,
}

impl BatteryPowerMonitor {
    /// Creates a new BatteryPowerMonitor with the provided device ID
    pub fn new(device_id: String) -> Self {
        Self { device_id }
    }
}

#[async_trait]
impl PowerMonitorTrait for BatteryPowerMonitor {
    type MetricType = BatteryPower;

    async fn name(&self) -> Result<String> {
        Ok("Battery Power".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Battery".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(format!("battery_power_{}", self.device_id))
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let power = BatteryPower { watts: 15.0 };
        Ok(Metric::new(power))
    }

    async fn power_consumption(&self) -> Result<f64> {
        Ok(15.0)
    }

    async fn power_state(&self) -> Result<PowerState> {
        Ok(PowerState::Battery)
    }

    async fn is_charging(&self) -> Result<bool> {
        Ok(false)
    }

    async fn is_external_power(&self) -> Result<bool> {
        Ok(false)
    }

    async fn time_remaining(&self) -> Result<i64> {
        Ok(3600) // 1 hour in seconds
    }
}

//=============================================================================
// Battery Temperature Monitor
//=============================================================================

/// Monitor for battery temperature
pub struct BatteryTemperatureMonitor {
    device_id: String,
}

impl BatteryTemperatureMonitor {
    /// Creates a new BatteryTemperatureMonitor with the provided device ID
    pub fn new(device_id: String) -> Self {
        Self { device_id }
    }
}

#[async_trait]
impl HardwareMonitor for BatteryTemperatureMonitor {
    type MetricType = Temperature;

    async fn name(&self) -> Result<String> {
        Ok("Battery Temperature".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Battery".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok(format!("battery_temperature_{}", self.device_id))
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let temp = Temperature::new_celsius(35.0);
        Ok(Metric::new(temp))
    }
}

#[async_trait]
impl TemperatureMonitor for BatteryTemperatureMonitor {}
