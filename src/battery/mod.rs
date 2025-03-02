use crate::{Error, Result};
use std::time::Duration;

/// Represents the current state of the system's battery
#[derive(Debug, PartialEq, Clone)]
pub struct Battery {
    /// Whether a battery is present in the system
    pub is_present: bool,
    /// Whether the battery is currently charging
    pub is_charging: bool,
    /// Battery charge percentage (0-100)
    pub percentage: f64,
    /// Estimated time remaining in minutes (when discharging)
    pub time_remaining: Duration,
}

impl Battery {
    /// Create a new Battery instance with the given parameters
    ///
    /// # Arguments
    /// * `is_present` - Whether a battery is present in the system
    /// * `is_charging` - Whether the battery is currently charging
    /// * `percentage` - Battery charge percentage (0-100)
    /// * `time_remaining` - Time remaining in minutes
    pub fn new(is_present: bool, is_charging: bool, percentage: f64, time_remaining: i32) -> Self {
        Self {
            is_present,
            is_charging,
            percentage,
            time_remaining: Duration::from_secs((time_remaining * 60) as u64),
        }
    }

    /// Get current battery information
    ///
    /// # Returns
    /// Returns a `Result` containing battery information or an error if the information
    /// cannot be retrieved.
    ///
    /// # Examples
    /// ```no_run
    /// use darwin_metrics::battery::Battery;
    ///
    /// let battery = Battery::get_info().unwrap();
    /// println!("Battery at {}%, {}", 
    ///     battery.percentage,
    ///     if battery.is_charging { "charging" } else { "discharging" }
    /// );
    /// ```
    pub fn get_info() -> Result<Self> {
        #[cfg(target_os = "macos")]
        {
            // Implementation will be added later
            Ok(Battery::new(true, false, 0.0, 0))
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(Error::not_implemented("Battery info is only available on macOS"))
        }
    }

    /// Returns true if the battery level is critically low (below 10%)
    pub fn is_critical(&self) -> bool {
        self.percentage < 10.0
    }

    /// Returns true if the battery level is low (below 20%)
    pub fn is_low(&self) -> bool {
        self.percentage < 20.0
    }

    /// Returns the estimated time remaining as a human-readable string
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_constructor() {
        let battery = Battery::new(true, false, 75.5, 90);
        assert_eq!(battery.is_present, true);
        assert_eq!(battery.is_charging, false);
        assert_eq!(battery.percentage, 75.5);
        assert_eq!(battery.time_remaining.as_secs(), 5400); // 90 minutes
    }

    #[test]
    fn test_battery_status_display() {
        let battery = Battery::new(true, false, 75.5, 90);
        assert_eq!(battery.time_remaining_display(), "1 hours 30 minutes");
        assert!(!battery.is_low());
        assert!(!battery.is_critical());
    }

    #[test]
    fn test_battery_low_critical() {
        let battery = Battery::new(true, false, 5.0, 10);
        assert!(battery.is_critical());
        assert!(battery.is_low());
    }
} 