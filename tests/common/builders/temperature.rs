use darwin_metrics::hardware::iokit::{FanInfo, ThermalInfo};
use darwin_metrics::utils::safe_dictionary::SafeDictionary;
use darwin_metrics::Error;
use std::collections::HashMap;

/// Builder for creating ThermalInfo test instances
pub struct ThermalInfoBuilder {
    temperatures: HashMap<String, f64>,
}

impl ThermalInfoBuilder {
    pub fn new() -> Self {
        Self {
            temperatures: HashMap::new(),
        }
    }

    pub fn with_cpu_temp(mut self, temp: f64) -> Self {
        self.temperatures.insert("CPU_0_DIE_TEMP".to_string(), temp);
        self
    }

    pub fn with_gpu_temp(mut self, temp: f64) -> Self {
        self.temperatures.insert("GPU_0_DIE_TEMP".to_string(), temp);
        self
    }

    pub fn with_ambient_temp(mut self, temp: f64) -> Self {
        self.temperatures.insert("AMBIENT_TEMP".to_string(), temp);
        self
    }

    pub fn with_battery_temp(mut self, temp: f64) -> Self {
        self.temperatures.insert("BATTERY_TEMP".to_string(), temp);
        self
    }

    pub fn with_heatsink_temp(mut self, temp: f64) -> Self {
        self.temperatures.insert("HS_0_TEMP".to_string(), temp);
        self
    }

    pub fn with_fan_speed(mut self, speed: f64) -> Self {
        self.temperatures.insert("FAN_0_SPEED".to_string(), speed);
        self
    }

    pub fn with_throttling(mut self, throttling: bool) -> Self {
        self.temperatures.insert("THERMAL_THROTTLING".to_string(), if throttling { 1.0 } else { 0.0 });
        self
    }

    pub fn build(self) -> ThermalInfo {
        let entries: Vec<(&str, f64)> = self
            .temperatures
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();
        
        let dict = darwin_metrics::utils::tests::test_utils::create_test_dictionary_with_entries(&entries);
        ThermalInfo::new(SafeDictionary::from(dict.into()))
    }
}

/// Builder for creating FanInfo test instances
pub struct FanInfoBuilder {
    current_speed: u32,
    target_speed: u32,
    min_speed: Option<u32>,
    max_speed: Option<u32>,
    index: u32,
    speed_rpm: u32,
    percentage: f64,
}

impl FanInfoBuilder {
    pub fn new() -> Self {
        Self {
            current_speed: 2000,
            target_speed: 2000,
            min_speed: Some(1000),
            max_speed: Some(4000),
            index: 0,
            speed_rpm: 2000,
            percentage: 33.3,
        }
    }

    pub fn with_current_speed(mut self, speed: u32) -> Self {
        self.current_speed = speed;
        self
    }

    pub fn with_target_speed(mut self, speed: u32) -> Self {
        self.target_speed = speed;
        self
    }

    pub fn with_speed(mut self, speed: u32) -> Self {
        self.speed_rpm = speed;
        self.current_speed = speed;
        self
    }

    pub fn with_min_speed(mut self, min: u32) -> Self {
        self.min_speed = Some(min);
        self
    }

    pub fn with_max_speed(mut self, max: u32) -> Self {
        self.max_speed = Some(max);
        self
    }

    pub fn with_index(mut self, index: u32) -> Self {
        self.index = index;
        self
    }

    pub fn with_percentage(mut self, percentage: f64) -> Self {
        self.percentage = percentage;
        self
    }

    pub fn build(self) -> FanInfo {
        FanInfo {
            current_speed: self.current_speed,
            target_speed: self.target_speed,
            min_speed: self.min_speed,
            max_speed: self.max_speed,
            index: self.index,
            speed_rpm: self.speed_rpm,
            percentage: self.percentage,
        }
    }
} 