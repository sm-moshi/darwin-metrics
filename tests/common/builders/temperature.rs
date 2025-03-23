use darwin_metrics::hardware::iokit::{FanInfo, ThermalInfo};
use darwin_metrics::utils::safe_dictionary::SafeDictionary;
use darwin_metrics::Error;
use std::collections::HashMap;
use darwin_metrics::{
    hardware::iokit::ThermalInfo,
    temperature::types::Fan,
};

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

/// Builder for creating Fan test instances
pub struct FanBuilder {
    name: String,
    speed_rpm: u32,
    min_speed: u32,
    max_speed: u32,
    target_speed: u32,
}

impl Default for FanBuilder {
    fn default() -> Self {
        Self {
            name: "System Fan".to_string(),
            speed_rpm: 2000,
            min_speed: 1000,
            max_speed: 4000,
            target_speed: 2000,
        }
    }
}

impl FanBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_speed_rpm(mut self, speed_rpm: u32) -> Self {
        self.speed_rpm = speed_rpm;
        self
    }

    pub fn with_min_speed(mut self, min_speed: u32) -> Self {
        self.min_speed = min_speed;
        self
    }

    pub fn with_max_speed(mut self, max_speed: u32) -> Self {
        self.max_speed = max_speed;
        self
    }

    pub fn with_target_speed(mut self, target_speed: u32) -> Self {
        self.target_speed = target_speed;
        self
    }

    pub fn build(self) -> Fan {
        Fan {
            name: self.name,
            speed_rpm: self.speed_rpm,
            min_speed: self.min_speed,
            max_speed: self.max_speed,
            target_speed: self.target_speed,
        }
    }
} 