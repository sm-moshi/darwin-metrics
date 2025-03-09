use std::time::Duration;

use crate::{
    error::{Error, Result},
    hardware::iokit::{IOKit, IOKitImpl},
};

const BATTERY_IS_PRESENT: &str = "BatteryInstalled";
const BATTERY_IS_CHARGING: &str = "IsCharging";
const BATTERY_CURRENT_CAPACITY: &str = "CurrentCapacity";
const BATTERY_MAX_CAPACITY: &str = "MaxCapacity";
const BATTERY_DESIGN_CAPACITY: &str = "DesignCapacity";
const BATTERY_CYCLE_COUNT: &str = "CycleCount";
const BATTERY_TEMPERATURE: &str = "Temperature";
const BATTERY_TIME_REMAINING: &str = "TimeRemaining";
const BATTERY_POWER_SOURCE: &str = "ExternalConnected";

#[derive(Debug, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub enum PowerSource {
    Battery,
    AC,
    Unknown,
}

#[derive(Debug)]
pub struct Battery {
    pub is_present: bool,
    pub is_charging: bool,
    pub percentage: f64,
    pub time_remaining: Duration,
    pub power_source: PowerSource,
    pub cycle_count: u32,
    pub health_percentage: f64,
    pub temperature: f64,

    #[cfg(not(test))]
    iokit: Box<dyn IOKit>,
    #[cfg(test)]
    pub iokit: Box<dyn IOKit>,
}

impl Default for Battery {
    fn default() -> Self {
        Self::with_values(false, false, 0.0, 0, PowerSource::Unknown, 0, 0.0, 0.0)
    }
}

impl Battery {
    pub fn new() -> Result<Self> {
        let mut battery = Self::default();
        battery.update()?;
        Ok(battery)
    }

    pub fn update(&mut self) -> Result<()> {
        let matching = self.iokit.io_service_matching("AppleSmartBattery");
        let service = self.iokit.io_service_get_matching_service(&matching);

        let Some(service) = service else {
            return Err(Error::service_not_found("Battery service not found".to_string()));
        };

        let properties = self.iokit.io_registry_entry_create_cf_properties(&service)?;

        self.is_present =
            self.iokit.get_bool_property(&properties, BATTERY_IS_PRESENT).unwrap_or(false);

        if !self.is_present {
            self.is_charging = false;
            self.percentage = 0.0;
            self.time_remaining = Duration::from_secs(0);
            self.power_source = PowerSource::Unknown;
            self.cycle_count = 0;
            self.health_percentage = 0.0;
            self.temperature = 0.0;
            return Ok(());
        }

        self.is_charging =
            self.iokit.get_bool_property(&properties, BATTERY_IS_CHARGING).unwrap_or(false);
        let is_external =
            self.iokit.get_bool_property(&properties, BATTERY_POWER_SOURCE).unwrap_or(false);
        self.power_source = if is_external { PowerSource::AC } else { PowerSource::Battery };

        let current =
            self.iokit.get_number_property(&properties, BATTERY_CURRENT_CAPACITY).unwrap_or(0)
                as f64;
        let max =
            self.iokit.get_number_property(&properties, BATTERY_MAX_CAPACITY).unwrap_or(100) as f64;
        self.percentage = if max > 0.0 { (current / max * 100.0).clamp(0.0, 100.0) } else { 0.0 };

        let design = self
            .iokit
            .get_number_property(&properties, BATTERY_DESIGN_CAPACITY)
            .unwrap_or(max as i64) as f64;
        self.health_percentage =
            if design > 0.0 { (max / design * 100.0).clamp(0.0, 100.0) } else { 0.0 };

        self.cycle_count =
            self.iokit.get_number_property(&properties, BATTERY_CYCLE_COUNT).unwrap_or(0) as u32;

        let time = self.iokit.get_number_property(&properties, BATTERY_TIME_REMAINING).unwrap_or(0);
        self.time_remaining = Duration::from_secs((time.max(0) * 60) as u64);

        let temp =
            self.iokit.get_number_property(&properties, BATTERY_TEMPERATURE).unwrap_or(0) as f64;
        self.temperature = temp / 100.0;

        Ok(())
    }

    pub fn get_info(&self) -> Result<Self> {
        let matching = self.iokit.io_service_matching("AppleSmartBattery");
        let service = self.iokit.io_service_get_matching_service(&matching);

        let Some(service) = service else {
            return Err(Error::ServiceNotFound("Battery service not found".to_string()));
        };

        match self.iokit.io_registry_entry_create_cf_properties(&service) {
            Ok(_) => Ok(self.clone()),
            Err(e) => Err(e),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_values(
        is_present: bool,
        is_charging: bool,
        percentage: f64,
        time_remaining: i32,
        power_source: PowerSource,
        cycle_count: u32,
        health_percentage: f64,
        temperature: f64,
    ) -> Self {
        Self {
            is_present,
            is_charging,
            percentage: percentage.clamp(0.0, 100.0),
            time_remaining: Duration::from_secs((time_remaining * 60) as u64),
            power_source,
            cycle_count,
            health_percentage: health_percentage.clamp(0.0, 100.0),
            temperature,
            iokit: Box::new(IOKitImpl),
        }
    }

    pub fn is_critical(&self) -> bool {
        self.percentage < 10.0
    }

    pub fn is_low(&self) -> bool {
        self.percentage < 20.0
    }

    pub fn time_remaining_display(&self) -> String {
        let minutes = self.time_remaining.as_secs() / 60;
        if minutes < 60 {
            format!("{} minutes", minutes)
        } else {
            let hours = minutes / 60;
            let remaining_minutes = minutes % 60;
            format!("{} hours {} minutes", hours, remaining_minutes)
        }
    }

    pub fn is_health_poor(&self) -> bool {
        self.health_percentage < 70.0
    }

    pub fn has_high_cycle_count(&self) -> bool {
        self.cycle_count > 1000
    }

    pub fn power_source_display(&self) -> &'static str {
        match self.power_source {
            PowerSource::Battery => "Battery Power",
            PowerSource::AC => "AC Power",
            PowerSource::Unknown => "Unknown Power Source",
        }
    }

    pub fn is_temperature_critical(&self) -> bool {
        self.temperature < -10.0 || self.temperature > 40.0
    }
}

impl Clone for Battery {
    fn clone(&self) -> Self {
        Self {
            is_present: self.is_present,
            is_charging: self.is_charging,
            percentage: self.percentage,
            time_remaining: self.time_remaining,
            power_source: self.power_source,
            cycle_count: self.cycle_count,
            health_percentage: self.health_percentage,
            temperature: self.temperature,
            iokit: Box::new(IOKitImpl),
        }
    }
}

impl PartialEq for Battery {
    fn eq(&self, other: &Self) -> bool {
        self.is_present == other.is_present
            && self.is_charging == other.is_charging
            && self.percentage == other.percentage
            && self.time_remaining == other.time_remaining
            && self.power_source == other.power_source
            && self.cycle_count == other.cycle_count
            && self.health_percentage == other.health_percentage
            && self.temperature == other.temperature
    }
}
