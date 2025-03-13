use crate::{
    error::{Error, Result},
    hardware::iokit::{IOKit, ThreadSafeAnyObject},
    utils::dictionary_access::DictionaryAccess,
};
use std::time::Duration;
// use objc2_foundation::NSDictionary;

#[cfg(test)]
use crate::hardware::iokit::mock::MockIOKit;

const BATTERY_SERVICE: &str = "AppleSmartBattery";
const BATTERY_PRESENT: &str = "BatteryInstalled";
const BATTERY_POWER_SOURCE: &str = "ExternalConnected";
const BATTERY_CYCLE_COUNT: &str = "CycleCount";
const BATTERY_TIME_REMAINING: &str = "TimeRemaining";
const BATTERY_TEMPERATURE: &str = "Temperature";

/// Represents the current power source for the system
///
/// This enum indicates whether the system is running on battery power, AC power, or if the power source is unknown.
#[derive(Debug, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub enum PowerSource {
    /// System is running on battery power
    Battery,
    /// System is running on AC power
    AC,
    /// Power source could not be determined
    Unknown,
}

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub present: bool,
    pub percentage: i64,
    pub cycle_count: i64,
    pub is_charging: bool,
    pub is_external: bool,
    pub temperature: f64,
    pub power_draw: f64,
    pub design_capacity: i64,
    pub current_capacity: i64,
}

#[derive(Debug)]
pub struct Battery {
    iokit: Box<dyn IOKit>,
    service: Option<ThreadSafeAnyObject>,
}

impl Clone for Battery {
    fn clone(&self) -> Self {
        Self { iokit: self.iokit.clone_box(), service: self.service.clone() }
    }
}

impl Drop for Battery {
    fn drop(&mut self) {
        // No need to explicitly close the service since Retained<AnyObject> handles cleanup
    }
}

impl Battery {
    /// Creates a new Battery instance.
    ///
    /// # Errors
    ///
    /// Returns an error if battery information cannot be retrieved from the system.
    pub fn new(iokit: Box<dyn IOKit>) -> Result<Self> {
        Ok(Self { iokit, service: None })
    }

    /// Updates battery information.
    ///
    /// # Errors
    ///
    /// Returns an error if battery information cannot be retrieved from the system.
    pub fn update(&mut self) -> Result<()> {
        let service = self.iokit.get_service_matching(BATTERY_SERVICE)?;
        self.service = service;
        Ok(())
    }

    /// Gets battery information.
    ///
    /// # Errors
    ///
    /// Returns an error if battery information cannot be retrieved from the system.
    pub fn get_info(&self) -> Result<BatteryInfo> {
        let service = self.service.as_ref().ok_or_else(|| {
            Error::io_error(
                "Battery service not found",
                std::io::Error::new(std::io::ErrorKind::NotFound, "Battery service not found"),
            )
        })?;
        let props = self.iokit.get_service_properties(service)?;

        let present = props.get_bool("BatteryInstalled").unwrap_or(false);
        let percentage = props.get_number("CurrentCapacity").unwrap_or(0.0) as i64;
        let cycle_count = props.get_number("CycleCount").unwrap_or(0.0) as i64;
        let is_charging = props.get_bool("IsCharging").unwrap_or(false);
        let is_external = props.get_bool("ExternalConnected").unwrap_or(false);
        let temperature = self.iokit.get_battery_temperature()?.unwrap_or(0.0);
        let power_draw = props.get_number("InstantAmperage").unwrap_or(0.0)
            * props.get_number("Voltage").unwrap_or(0.0)
            / 1000.0;
        let design_capacity = props.get_number("DesignCapacity").unwrap_or(0.0) as i64;
        let current_capacity = props.get_number("MaxCapacity").unwrap_or(0.0) as i64;

        Ok(BatteryInfo {
            present,
            percentage,
            cycle_count,
            is_charging,
            is_external,
            temperature,
            power_draw,
            design_capacity,
            current_capacity,
        })
    }

    #[cfg(test)]
    /// Creates a new Battery instance with the specified values
    pub fn with_values(
        battery_is_present: bool,
        battery_is_charging: bool,
        battery_cycle_count: u32,
        battery_health_percentage: f64,
        battery_temperature: f64,
        battery_time_remaining: Duration,
        battery_power_draw: f64,
        battery_design_capacity: f64,
        battery_current_capacity: f64,
    ) -> Result<Self> {
        let iokit = Box::new(MockIOKit {
            battery_is_present,
            battery_is_charging,
            battery_cycle_count,
            battery_health_percentage,
            battery_temperature,
            battery_time_remaining,
            battery_power_draw,
            battery_design_capacity,
            battery_current_capacity,
            // Default values for CPU-related fields
            physical_cores: 4,
            logical_cores: 8,
            core_usage: vec![0.0; 4],
            cpu_temperature: 45.0,
        });

        Self::new(iokit)
    }

    /// Returns true if the battery level is critically low (below 5%)
    pub fn is_critically_low(&self) -> Result<bool> {
        Ok(self.get_info()?.percentage < 5)
    }

    /// Returns true if the battery level is low (below 20%)
    pub fn is_low(&self) -> Result<bool> {
        Ok(self.get_info()?.percentage < 20)
    }

    /// Returns a human-readable string describing the time remaining
    pub fn time_remaining_display(&self) -> Result<String> {
        let _info = self.get_info()?;
        if let Some(time_remaining) = self.get_time_remaining()? {
            let total_minutes = time_remaining.as_secs() as u64;
            let hours = total_minutes / 60;
            let minutes = total_minutes % 60;
            if hours > 0 {
                Ok(format!("{}h {}m", hours, minutes))
            } else {
                Ok(format!("{}m", minutes))
            }
        } else {
            Ok("0m".to_string())
        }
    }

    /// Returns true if the battery health is poor (below 80%)
    pub fn is_health_critical(&self) -> Result<bool> {
        let info = self.get_info()?;
        Ok((info.current_capacity as f64 / info.design_capacity as f64 * 100.0) < 80.0)
    }

    /// Returns true if the battery has a high cycle count (over 1000)
    pub fn is_cycle_count_critical(&self) -> Result<bool> {
        Ok(self.get_info()?.cycle_count > 1000)
    }

    /// Returns a human-readable string describing the power source
    pub fn power_source_display(&self) -> Result<&'static str> {
        let info = self.get_info()?;
        Ok(if info.is_external {
            "AC Power"
        } else if info.present {
            "Battery"
        } else {
            "Unknown"
        })
    }

    /// Returns true if the battery temperature is critically high (above 45°C)
    pub fn is_temperature_critical(&self) -> Result<bool> {
        Ok(self.get_info()?.temperature > 45.0)
    }

    /// Returns the current battery percentage
    pub fn percentage(&self) -> Result<i64> {
        Ok(self.get_info()?.percentage)
    }

    /// Returns the current battery time remaining
    pub fn get_time_remaining(&self) -> Result<Option<Duration>> {
        let info = self.get_info()?;
        if info.is_external {
            Ok(None)
        } else {
            let power_draw = info.power_draw;
            if power_draw > 0.0 {
                let time_remaining = (info.current_capacity as f64 / power_draw) * 3600.0;
                Ok(Some(Duration::from_secs(time_remaining as u64)))
            } else {
                Ok(None)
            }
        }
    }

    /// Returns the current battery cycle count
    pub fn cycle_count(&self) -> Result<i64> {
        Ok(self.get_info()?.cycle_count)
    }

    /// Returns the current battery temperature
    pub fn temperature(&self) -> Result<f64> {
        Ok(self.get_info()?.temperature)
    }

    /// Returns the current battery power source
    pub fn power_source(&self) -> Result<PowerSource> {
        let info = self.get_info()?;
        Ok(if info.is_external {
            PowerSource::AC
        } else if info.present {
            PowerSource::Battery
        } else {
            PowerSource::Unknown
        })
    }

    pub fn is_present(&self) -> Result<bool> {
        Ok(self.get_info()?.present)
    }

    pub fn is_charging(&self) -> Result<bool> {
        Ok(self.get_info()?.is_charging)
    }

    pub fn power_draw(&self) -> Result<f64> {
        Ok(self.get_info()?.power_draw)
    }

    pub fn design_capacity(&self) -> Result<i64> {
        Ok(self.get_info()?.design_capacity)
    }

    pub fn current_capacity(&self) -> Result<i64> {
        Ok(self.get_info()?.current_capacity)
    }
}

impl BatteryInfo {
    pub fn new(
        present: bool,
        percentage: i64,
        cycle_count: i64,
        is_charging: bool,
        is_external: bool,
        temperature: f64,
        power_draw: f64,
        design_capacity: i64,
        current_capacity: i64,
    ) -> Self {
        Self {
            present,
            percentage,
            cycle_count,
            is_charging,
            is_external,
            temperature,
            power_draw,
            design_capacity,
            current_capacity,
        }
    }

    pub fn cycle_count(&self) -> i64 {
        self.cycle_count
    }

    pub fn temperature(&self) -> f64 {
        self.temperature
    }

    pub fn power_source(&self) -> PowerSource {
        if self.present {
            if self.is_charging {
                PowerSource::AC
            } else {
                PowerSource::Battery
            }
        } else {
            PowerSource::Unknown
        }
    }

    pub fn health_percentage(&self) -> f64 {
        self.percentage as f64
    }
}

impl PartialEq for BatteryInfo {
    fn eq(&self, other: &Self) -> bool {
        self.present == other.present
            && self.percentage == other.percentage
            && self.cycle_count == other.cycle_count
            && self.is_charging == other.is_charging
            && self.is_external == other.is_external
            && self.power_draw == other.power_draw
            && self.design_capacity == other.design_capacity
            && self.current_capacity == other.current_capacity
            && self.temperature == other.temperature
    }
}
