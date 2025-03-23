use std::collections::HashMap;
use std::os::raw::c_char;

use crate::{
    power::{Power, PowerState, PowerConsumption},
    error::Result,
};

/// Builder for creating test power instances
pub struct TestPowerBuilder {
    package_power: f32,
    cores_power: f32,
    gpu_power: Option<f32>,
    dram_power: Option<f32>,
    neural_engine_power: Option<f32>,
    power_state: PowerState,
    battery_percentage: Option<f32>,
    power_impact: Option<f32>,
    is_throttling: bool,
    smc_key_values: HashMap<[c_char; 4], f32>,
}

impl TestPowerBuilder {
    /// Create a new TestPowerBuilder with default values
    pub fn new() -> Self {
        Self {
            package_power: 10.0,
            cores_power: 5.0,
            gpu_power: Some(2.0),
            dram_power: Some(1.0),
            neural_engine_power: Some(0.5),
            power_state: PowerState::AC,
            battery_percentage: Some(80.0),
            power_impact: Some(12.0),
            is_throttling: false,
            smc_key_values: HashMap::new(),
        }
    }
    
    /// Set the package power consumption
    pub fn package_power(mut self, power: f32) -> Self {
        self.package_power = power;
        self
    }
    
    /// Set the cores power consumption
    pub fn cores_power(mut self, power: f32) -> Self {
        self.cores_power = power;
        self
    }
    
    /// Set the GPU power consumption
    pub fn gpu_power(mut self, power: Option<f32>) -> Self {
        self.gpu_power = power;
        self
    }
    
    /// Set the DRAM power consumption
    pub fn dram_power(mut self, power: Option<f32>) -> Self {
        self.dram_power = power;
        self
    }
    
    /// Set the neural engine power consumption
    pub fn neural_engine_power(mut self, power: Option<f32>) -> Self {
        self.neural_engine_power = power;
        self
    }
    
    /// Set the power state
    pub fn power_state(mut self, state: PowerState) -> Self {
        self.power_state = state;
        self
    }
    
    /// Set the battery percentage
    pub fn battery_percentage(mut self, percentage: Option<f32>) -> Self {
        self.battery_percentage = percentage;
        self
    }
    
    /// Set the power impact
    pub fn power_impact(mut self, impact: Option<f32>) -> Self {
        self.power_impact = impact;
        self
    }
    
    /// Set whether power throttling is active
    pub fn throttling(mut self, is_throttling: bool) -> Self {
        self.is_throttling = is_throttling;
        self
    }
    
    /// Set a value for a specific SMC key
    pub fn smc_key_value(mut self, key: [c_char; 4], value: f32) -> Self {
        self.smc_key_values.insert(key, value);
        self
    }
    
    /// Build a mock Power instance
    pub fn build(self) -> Result<MockPower> {
        Ok(MockPower {
            consumption: PowerConsumption {
                package: self.package_power,
                cores: self.cores_power,
                gpu: self.gpu_power,
                dram: self.dram_power,
                neural_engine: self.neural_engine_power,
                power_state: self.power_state,
                battery_percentage: self.battery_percentage,
                power_impact: self.power_impact,
            },
            is_throttling: self.is_throttling,
            smc_key_values: self.smc_key_values,
        })
    }
}

/// A mock implementation of Power for testing
pub struct MockPower {
    consumption: PowerConsumption,
    is_throttling: bool,
    smc_key_values: HashMap<[c_char; 4], f32>,
}

impl MockPower {
    /// Get the power consumption
    pub fn get_power_consumption(&self) -> Result<PowerConsumption> {
        Ok(self.consumption.clone())
    }
    
    /// Check if power throttling is active
    pub fn is_power_throttling(&self) -> Result<bool> {
        Ok(self.is_throttling)
    }
    
    /// Read a value from an SMC key
    pub fn read_smc_power_key(&self, key: [c_char; 4]) -> Result<f32> {
        Ok(self.smc_key_values.get(&key).copied().unwrap_or(0.0))
    }
}