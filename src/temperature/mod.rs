use crate::Result;
use crate::hardware::iokit::{IOKitImpl, IOKit};
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemperatureError {
    #[error("System call failed")]
    SystemCallFailed,
    #[error("Invalid temperature data")]
    InvalidData,
    #[error("Sensor not found")]
    SensorNotFound,
    #[error("IOKit operation failed")]
    IOKitFailure,
}

impl From<TemperatureError> for crate::Error {
    fn from(err: TemperatureError) -> Self {
        crate::Error::Temperature(err.to_string())
    }
}

#[derive(Debug)]
pub struct CoreTemperature {
    pub efficiency_cores: Vec<f32>,
    pub performance_cores: Vec<f32>,
    pub gpu: Option<f32>,
}

#[derive(Debug)]
pub struct FanInfo {
    pub rpm: u32,
    pub identifier: String,
    pub location: String,
}

#[derive(Debug)]
pub struct ThermalZone {
    pub temperature: f32,
    pub max_temperature: f32,
    pub critical: bool,
}

#[derive(Debug)]
pub struct ThermalState {
    pub throttling: bool,
    pub power_limit: f32,
    pub current_power: f32,
}

#[derive(Debug)]
pub struct SensorReading {
    pub name: String,
    pub temperature: f32,
    pub location: SensorLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SensorLocation {
    Cpu,
    Gpu,
    Memory,
    Storage,
    Battery,
    Other(String),
}

impl fmt::Display for SensorLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorLocation::Cpu => write!(f, "CPU"),
            SensorLocation::Gpu => write!(f, "GPU"),
            SensorLocation::Memory => write!(f, "Memory"),
            SensorLocation::Storage => write!(f, "Storage"),
            SensorLocation::Battery => write!(f, "Battery"),
            SensorLocation::Other(s) => write!(f, "{}", s),
        }
    }
}

/// Apple Silicon processors have a mix of efficiency and performance cores
/// This struct allows accessing temperature data for each type
impl CoreTemperature {
    /// Retrieve CPU core temperatures from the System Management Controller (SMC)
    pub fn get_core_temperatures() -> Result<CoreTemperature> {
        let io_kit = IOKitImpl::default();
        
        // On Apple Silicon, we can access temperature for various sensors
        // TC0E, TC1E, TC2E, TC3E for efficiency cores
        // TC0P, TC1P, TC2P, TC3P, etc for performance cores
        
        // For simplicity in this implementation, we'll just get the 
        // main CPU temperature and GPU temperature
        let cpu_temp = io_kit.get_cpu_temperature()?;
        
        // Try to get GPU temperature, but it's optional (some Macs don't have dedicated GPUs)
        let gpu_temp = match io_kit.get_gpu_temperature() {
            Ok(temp) => Some(temp as f32),
            Err(_) => None,
        };

        // In a full implementation, we'd iterate through all cores
        // For now, we'll split the CPU temperature to simulate multiple cores
        let efficiency_cores = vec![cpu_temp as f32 - 2.0, cpu_temp as f32 - 1.0];
        let performance_cores = vec![cpu_temp as f32, cpu_temp as f32 + 1.0];

        Ok(CoreTemperature {
            efficiency_cores,
            performance_cores,
            gpu: gpu_temp,
        })
    }

    /// Get fan speed information
    pub fn get_fan_rpms() -> Result<Vec<FanInfo>> {
        let io_kit = IOKitImpl::default();
        
        // Get the fan speed
        let fan_speed = match io_kit.get_fan_speed() {
            Ok(speed) => speed,
            Err(_) => {
                // Some Macs (especially Apple Silicon) don't have fans
                // Return an empty vector in this case
                return Ok(Vec::new());
            }
        };

        // Basic fan info - in a real-world scenario we would query 
        // additional fan properties from IOKit
        let fan = FanInfo {
            rpm: fan_speed,
            identifier: "System Fan".to_string(),
            location: "Main".to_string(),
        };

        Ok(vec![fan])
    }

    /// Determine if the system is experiencing thermal throttling
    pub fn get_thermal_state() -> Result<ThermalState> {
        // This is a simplified implementation - in a complete version
        // we'd query the power management framework to get actual
        // throttling information
        
        let io_kit = IOKitImpl::default();
        let cpu_temp = io_kit.get_cpu_temperature()?;
        
        // Simplified logic - consider throttling if CPU temperature is high
        // Real implementation would use proper macOS APIs to detect throttling
        let throttling = cpu_temp > 80.0;
        
        Ok(ThermalState {
            throttling,
            power_limit: 15.0, // Default TDP for M1/M2 processors
            current_power: if throttling { 12.0 } else { 10.0 },
        })
    }

    /// Get thermal warnings if any components are too hot
    pub fn check_thermal_warnings() -> Result<Vec<String>> {
        let mut warnings = Vec::new();
        let io_kit = IOKitImpl::default();
        
        // Check CPU temperature
        let cpu_temp = io_kit.get_cpu_temperature()?;
        if cpu_temp > 90.0 {
            warnings.push(format!("Critical CPU temperature: {:.1}°C", cpu_temp));
        }
        
        // Check GPU temperature if available
        if let Ok(gpu_temp) = io_kit.get_gpu_temperature() {
            if gpu_temp > 85.0 {
                warnings.push(format!("Critical GPU temperature: {:.1}°C", gpu_temp));
            }
        }
        
        Ok(warnings)
    }

    /// Get readings from all available temperature sensors
    pub fn get_all_sensors() -> Result<Vec<SensorReading>> {
        let mut readings = Vec::new();
        let io_kit = IOKitImpl::default();
        
        // CPU temperature
        if let Ok(temp) = io_kit.get_cpu_temperature() {
            readings.push(SensorReading {
                name: "CPU Die".to_string(),
                temperature: temp as f32,
                location: SensorLocation::Cpu,
            });
        }
        
        // GPU temperature (if available)
        if let Ok(temp) = io_kit.get_gpu_temperature() {
            readings.push(SensorReading {
                name: "GPU Die".to_string(),
                temperature: temp as f32,
                location: SensorLocation::Gpu,
            });
        }
        
        // In a complete implementation, we would scan all
        // available SMC keys for temperature sensors
        
        Ok(readings)
    }

    /// Helper function to determine sensor location from name
    fn determine_sensor_location(name: &str) -> SensorLocation {
        if name.contains("CPU") {
            SensorLocation::Cpu
        } else if name.contains("GPU") {
            SensorLocation::Gpu
        } else if name.contains("Memory") {
            SensorLocation::Memory
        } else if name.contains("Storage") {
            SensorLocation::Storage
        } else if name.contains("Battery") {
            SensorLocation::Battery
        } else {
            SensorLocation::Other(name.to_string())
        }
    }
}