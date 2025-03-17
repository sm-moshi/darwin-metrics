use crate::{
    error::{Error, Result},
    hardware::iokit::IOKit,
};

/// Monitor for battery health metrics including cycle count and capacity
pub struct BatteryHealthMonitor {
    iokit: Box<dyn IOKit>,
}

impl BatteryHealthMonitor {
    /// Creates a new BatteryHealthMonitor with the provided IOKit implementation
    pub fn new(iokit: Box<dyn IOKit>) -> Self {
        Self { iokit }
    }

    /// Gets the battery cycle count
    pub async fn cycle_count(&self) -> Result<i64> {
        let info = self.iokit.get_battery_info()?;
        match info.get_i64("CycleCount") {
            Some(value) => Ok(value),
            None => Err(Error::invalid_data(
                "Battery cycle count",
                Some("CycleCount information not available".to_string()),
            )),
        }
    }

    /// Gets the maximum capacity of the battery
    pub async fn max_capacity(&self) -> Result<f64> {
        let info = self.iokit.get_battery_info()?;
        info.get_f64("MaxCapacity")
    }

    /// Gets the current capacity of the battery
    pub async fn current_capacity(&self) -> Result<f64> {
        let info = self.iokit.get_battery_info()?;
        info.get_f64("CurrentCapacity")
    }

    /// Gets the health percentage of the battery (current capacity / design capacity)
    pub async fn health_percentage(&self) -> Result<f64> {
        let info = self.iokit.get_battery_info()?;

        let design = info.get_f64("DesignCapacity")?;
        if design <= 0.0 {
            return Err(Error::invalid_data(
                "Battery design capacity",
                Some("Design capacity must be positive".to_string()),
            ));
        }

        let max = info.get_f64("MaxCapacity")?;
        if max < 0.0 {
            return Err(Error::invalid_data(
                "Battery maximum capacity",
                Some("Maximum capacity cannot be negative".to_string()),
            ));
        }

        Ok((max / design) * 100.0)
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
