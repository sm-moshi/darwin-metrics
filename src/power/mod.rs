use thiserror::Error;

#[derive(Debug, Error)]
pub enum PowerError {
    #[error("System call failed")]
    SystemCallFailed,
    #[error("Invalid power data")]
    InvalidData,
}

#[derive(Debug)]
pub struct PowerConsumption {
    pub package: f32,
    pub cores: f32,
    pub gpu: Option<f32>,
    pub dram: f32,
    pub system_agent: f32,
}

pub async fn get_power_consumption() -> Result<PowerConsumption, PowerError> {
    // Implementation of power consumption collection
    // ...
    Ok(PowerConsumption {
        package: 10.0,
        cores: 5.0,
        gpu: Some(3.0),
        dram: 2.0,
        system_agent: 1.0,
    })
}
