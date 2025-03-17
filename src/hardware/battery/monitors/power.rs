use crate::{
    core::metrics::{
        hardware::{PowerMonitorTrait, PowerStateMonitor},
        Metric,
    },
    error::{Error, Result},
    hardware::battery::types::BatteryPower,
    hardware::iokit::IOKit,
    power::PowerState,
};

/// Monitor for battery power metrics including charging state, power state, and remaining time
pub struct BatteryPowerMonitor {
    iokit: Box<dyn IOKit>,
}

impl BatteryPowerMonitor {
    /// Creates a new BatteryPowerMonitor with the provided IOKit implementation
    pub fn new(iokit: Box<dyn IOKit>) -> Self {
        Self { iokit }
    }

    /// Checks if the battery is currently charging
    pub async fn is_charging(&self) -> Result<bool> {
        let info = self.iokit.get_battery_info()?;
        match info.get_bool("IsCharging") {
            Some(value) => Ok(value),
            None => Err(Error::invalid_data(
                "Battery charging status",
                Some("IsCharging information not available".to_string()),
            )),
        }
    }

    /// Checks if the device is running on external power
    pub async fn is_external_power(&self) -> Result<bool> {
        let info = self.iokit.get_battery_info()?;
        match info.get_bool("ExternalConnected") {
            Some(value) => Ok(value),
            None => Err(Error::invalid_data(
                "Battery external power",
                Some("ExternalConnected information not available".to_string()),
            )),
        }
    }

    /// Gets the current power state of the battery
    pub async fn power_state(&self) -> Result<PowerState> {
        let is_charging = self.is_charging().await?;
        let is_external = self.is_external_power().await?;

        Ok(match (is_charging, is_external) {
            (true, _) => PowerState::Charging,
            (false, true) => PowerState::AC,
            (false, false) => PowerState::Battery,
        })
    }

    /// Gets the estimated time remaining in seconds
    pub async fn time_remaining(&self) -> Result<i64> {
        let info = self.iokit.get_battery_info()?;
        match info.get_i64("TimeRemaining") {
            Some(value) => Ok(value),
            None => Err(Error::invalid_data(
                "Battery time remaining",
                Some("TimeRemaining information not available".to_string()),
            )),
        }
    }

    /// Gets the current battery percentage
    pub async fn percentage(&self) -> Result<f64> {
        let info = self.iokit.get_battery_info()?;

        let current = info.get_f64("CurrentCapacity")?;
        if current < 0.0 {
            return Err(Error::invalid_data(
                "Battery current capacity",
                Some("Current capacity cannot be negative".to_string()),
            ));
        }

        let max = info.get_f64("MaxCapacity")?;
        if max <= 0.0 {
            return Err(Error::invalid_data(
                "Battery maximum capacity",
                Some("Maximum capacity must be positive".to_string()),
            ));
        }

        Ok((current / max) * 100.0)
    }
}

#[async_trait::async_trait]
impl PowerMonitorTrait for BatteryPowerMonitor {
    type MetricType = BatteryPower;

    async fn name(&self) -> Result<String> {
        Ok("Battery Power".to_string())
    }

    async fn hardware_type(&self) -> Result<String> {
        Ok("Battery".to_string())
    }

    async fn device_id(&self) -> Result<String> {
        Ok("battery_power".to_string())
    }

    async fn get_metric(&self) -> Result<Metric<Self::MetricType>> {
        let info = self.iokit.get_battery_info()?;
        let power = info.get_f64("InstantAmperage").unwrap_or(0.0) * info.get_f64("Voltage").unwrap_or(0.0) / 1000.0;
        Ok(Metric::new(BatteryPower { watts: power }))
    }

    async fn power_consumption(&self) -> Result<f64> {
        let info = self.iokit.get_battery_info()?;
        let power = info.get_f64("InstantAmperage").unwrap_or(0.0) * info.get_f64("Voltage").unwrap_or(0.0) / 1000.0;
        Ok(power)
    }

    async fn power_state(&self) -> Result<PowerState> {
        let is_charging = self.is_charging().await?;
        let is_external = self.is_external_power().await?;

        Ok(match (is_charging, is_external) {
            (true, _) => PowerState::Charging,
            (false, true) => PowerState::AC,
            (false, false) => PowerState::Battery,
        })
    }

    async fn is_charging(&self) -> Result<bool> {
        let info = self.iokit.get_battery_info()?;
        match info.get_bool("IsCharging") {
            Some(value) => Ok(value),
            None => Err(Error::invalid_data(
                "Battery charging status",
                Some("IsCharging information not available".to_string()),
            )),
        }
    }

    async fn is_external_power(&self) -> Result<bool> {
        let info = self.iokit.get_battery_info()?;
        match info.get_bool("ExternalConnected") {
            Some(value) => Ok(value),
            None => Err(Error::invalid_data(
                "Battery external power",
                Some("ExternalConnected information not available".to_string()),
            )),
        }
    }

    async fn time_remaining(&self) -> Result<i64> {
        let info = self.iokit.get_battery_info()?;
        match info.get_i64("TimeRemaining") {
            Some(value) => Ok(value),
            None => Err(Error::invalid_data(
                "Battery time remaining",
                Some("TimeRemaining information not available".to_string()),
            )),
        }
    }
}

#[async_trait::async_trait]
impl PowerStateMonitor for BatteryPowerMonitor {
    async fn power_state(&self) -> Result<PowerState> {
        let is_charging = self.is_charging().await?;
        let is_external = self.is_external_power().await?;

        Ok(match (is_charging, is_external) {
            (true, _) => PowerState::Charging,
            (false, true) => PowerState::AC,
            (false, false) => PowerState::Battery,
        })
    }

    async fn battery_percentage(&self) -> Result<Option<f32>> {
        Ok(Some(self.percentage().await? as f32))
    }

    async fn time_remaining(&self) -> Result<Option<u32>> {
        let time = self.time_remaining().await?;
        if time <= 0 {
            Ok(None)
        } else {
            Ok(Some(time as u32))
        }
    }

    async fn is_on_battery(&self) -> Result<bool> {
        let state = self.power_state().await?;
        Ok(state == PowerState::Battery)
    }

    async fn is_charging(&self) -> Result<bool> {
        let info = self.iokit.get_battery_info()?;
        match info.get_bool("IsCharging") {
            Some(value) => Ok(value),
            None => Err(Error::invalid_data(
                "Battery charging status",
                Some("IsCharging information not available".to_string()),
            )),
        }
    }
}
